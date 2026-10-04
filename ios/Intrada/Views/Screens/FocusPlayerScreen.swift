import SharedTypes
import SwiftUI

/// The live, while-playing surface (the player's Focus screen). Renders the
/// core's `ActiveSessionView`; every control sends a `SessionEvent` and the core
/// drives the transition (Done on the last item → Summary). Full-screen, no
/// chrome — "the app disappears during practice".
struct FocusPlayerScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.scenePhase) private var scenePhase
  @Environment(\.screenWakeLock) private var wakeLock
  @Environment(\.accessibilityReduceMotion) private var reduceMotion

  // Snapshots inject a fixed instant so the timer is deterministic; production
  // passes nil and the timer ticks off the wall clock (mirrors PracticeScreen).
  private let referenceDate: Date?

  init(referenceDate: Date? = nil) { self.referenceDate = referenceDate }

  /// Shown in the sheet, not the banner behind it (#2009); cleared when the core closes the sheet.
  @State private var reflectionRefusal: String?
  @State private var click = ClickController()
  @State private var configuringClick = false
  @State private var switchingVariation = false

  private var active: ActiveSessionView? { store.viewModel?.activeSession }

  /// What the click is doing right now, sent with every event that closes a
  /// play so the core stamps the tempo on the play it was played on (#1761).
  private var tempoReading: TempoReading {
    TempoReading(
      bpm: UInt16(clamping: click.bpm), clickSounding: click.isRunning, click: click.clickState)
  }

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      if let active {
        content(active)
      }
    }
    // The core's draft decides whether the sheet is up, so a resume reopens it (#2137).
    .sheet(item: Binding(get: { reflectionTarget }, set: { _ in })) { target in
      if let limits = store.viewModel?.limits {
        ReflectionSheet(
          itemTitle: target.title, elapsedDisplay: target.elapsedDisplay,
          tempoTarget: target.tempoTargetBpm, startingTempoBpm: target.startingTempoBpm,
          currentClick: target.sheetClick, plays: target.plays,
          limits: limits,
          refusal: reflectionRefusal, seed: target.seed,
          onSave: { result in handleReflection(target, result) },
          onSkip: { handleSkipRating(target) },
          onDraft: { result in
            store.send(
              .session(
                .updateReflectionDraft(
                  answers: ReflectionHandoff.draft(result, plays: target.plays))))
          }
        )
        .presentationDetents([.medium, .large])
        .interactiveDismissDisabled()
      }
    }
    .sheet(isPresented: $configuringClick, onDismiss: recordTempoChange) {
      if let limits = store.viewModel?.limits {
        ClickSheet(click: click, bpm: click.bpm, limits: limits)
      }
    }
    .sheet(isPresented: $switchingVariation) {
      if let active {
        VariationPickerSheet(
          itemTitle: active.currentItemTitle,
          currentVariations: active.currentVariations,
          currentVariationId: active.currentVariationIds.first,
          onPick: { switchVariation(active, to: $0) })
      }
    }
    .task { reseedClick() }
    .onChange(of: active?.reflection == nil) { _, closed in
      if closed { reflectionRefusal = nil }
    }
    .onChange(of: active?.currentPosition) { _, _ in reseedClick() }
    // `initial: true` is what takes the hold for a session started in the
    // foreground, where the phase never changes (#1513).
    .onChange(of: scenePhase, initial: true) { _, phase in
      wakeLock.update(sessionActive: active != nil, phase: phase)
      if phase == .active { click.enteredForeground() } else { click.enteredBackground() }
    }
    .onDisappear {
      click.dispose()
      wakeLock.release()
    }
  }

  private func reseedClick() {
    guard let active, let limits = store.viewModel?.limits else {
      click.stop()
      return
    }
    click.reseed(from: active, limits: limits)
  }

  private func content(_ active: ActiveSessionView) -> some View {
    VStack(spacing: 0) {
      topChrome(active).fadeUp(0)
      Spacer(minLength: IntradaSpacing.card)
      centerInfo(active).fadeUp(1)
      timer(active).fadeUp(2).padding(.top, IntradaSpacing.section)
      clickRow(active).padding(.top, IntradaSpacing.controlGap)
      if click.isRunning {
        barLine.padding(.top, IntradaSpacing.controlGap)
      }
      repCounter(active).fadeUp(3).padding(.top, IntradaSpacing.section)
      Spacer(minLength: IntradaSpacing.card)
      controls(active).fadeUp(4)
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.card)
  }

  // ── Top: session elapsed + position label + progress + options menu ──

  @ViewBuilder private func topChrome(_ active: ActiveSessionView) -> some View {
    let start = SessionClock.parseRFC3339(active.startedAt)
    if let start, referenceDate == nil {
      // Anchored to the session start, not `.now`: `.now` re-phases the tick on
      // every body evaluation, so the two timers drift out of step on a screen
      // whose brief is to sit still.
      TimelineView(.periodic(from: start, by: 1)) { context in
        band(active, elapsed: Int(context.date.timeIntervalSince(start)))
      }
    } else {
      // A session total nobody can vouch for is worse than none: an unparsable
      // anchor would otherwise count up from screen appearance and read as fact.
      band(active, elapsed: start.map { Int((referenceDate ?? .now).timeIntervalSince($0)) })
    }
  }

  private func band(_ active: ActiveSessionView, elapsed: Int?) -> some View {
    SessionOrientationBand(
      sessionElapsed: elapsed,
      positionLabel: positionLabel(active),
      types: active.entries.map(\.itemType),
      filled: min(Int(active.currentPosition) + 1, Int(active.totalItems)),
      menu: { optionsMenu })
  }

  private func positionLabel(_ active: ActiveSessionView) -> String {
    "Focus · \(active.currentPosition + 1) of \(active.totalItems)"
  }

  private var optionsMenu: some View {
    Menu {
      Button {
        store.send(.session(.skipItem(now: SessionClock.nowRFC3339())))
      } label: {
        Label("Skip this item", systemImage: "forward.end")
      }
      Button(role: .destructive) {
        store.send(
          .session(.endSessionEarly(now: SessionClock.nowRFC3339(), reading: tempoReading)))
      } label: {
        Label("End session early", systemImage: "stop.circle")
      }
    } label: {
      Image(systemName: "ellipsis")
        .iconSize(.control)
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(width: 28, height: 28)
    }
    .accessibilityLabel("Session options")
    .accessibilityIdentifier("player.options")
  }

  // ── Centre: item identity, the live timer ──

  private func centerInfo(_ active: ActiveSessionView) -> some View {
    VStack(spacing: 8) {
      TypeBadge(kind: active.currentItemType)
      Text(active.currentItemTitle)
        .font(IntradaFont.pageTitle)
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.center)
      if let pieceTitle = active.currentRelatedPieceTitle {
        Label("Related to \(pieceTitle)", systemImage: "arrow.turn.down.right")
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.accent)
      }
      if let aim = active.currentItemIntention, !aim.isEmpty {
        Text("Aim: \(aim)")
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
      }
      if let notes = active.currentItemNotes, !notes.isEmpty {
        Text("Notes: \(notes)")
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
          .lineLimit(3)
          .truncationMode(.tail)
      }
      variationChip(active)
    }
    .padding(.horizontal, IntradaSpacing.card)
  }

  // ── The variation being practised right now (#1739 decision 6) ──

  @ViewBuilder private func variationChip(_ active: ActiveSessionView) -> some View {
    if !active.currentVariations.isEmpty {
      Button {
        switchingVariation = true
      } label: {
        HStack(spacing: 7) {
          Text(active.currentPlayLabel ?? "Pick a variation")
            .font(IntradaFont.segment)
            .foregroundStyle(
              active.currentPlayLabel == nil
                ? IntradaColor.inkSecondary : IntradaColor.ink
            )
            // A variation named "2nd inversion" shrinks rather than
            // ellipsising, as the click's readout does (T19).
            .lineLimit(1)
            .minimumScaleFactor(0.7)
          Image(systemName: "chevron.down")
            .font(IntradaFont.small.weight(.semibold))
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        .padding(.horizontal, 14)
        .frame(minHeight: 44)
        .background(IntradaColor.cardFill, in: Capsule())
        .overlay(Capsule().stroke(IntradaColor.hairline, lineWidth: 1))
        .cardShadow()
      }
      .buttonStyle(PressRebound())
      .accessibilityLabel("Variation")
      .accessibilityIdentifier("player.variation")
      .accessibilityValue(active.currentPlayLabel ?? "none picked")
      .accessibilityHint("Switches to another variation of this exercise")
    }
  }

  /// False when the core refused the switch (the per-entry cap, or a variation
  /// that has since been deleted), so the picker stays up beside the banner.
  private func switchVariation(_ active: ActiveSessionView, to variationId: String) -> Bool {
    let pos = Int(active.currentPosition)
    guard active.entries.indices.contains(pos) else { return false }
    return store.sendAccepted(
      .session(
        .switchPlay(
          entryId: active.entries[pos].id, sectionId: active.currentSectionId,
          key: active.currentKey, variationIds: [variationId],
          now: SessionClock.nowRFC3339(), reading: tempoReading)))
  }

  // An anchor that will not parse draws no ring rather than counting from when
  // the screen appeared (#1942).
  @ViewBuilder private func timer(_ active: ActiveSessionView) -> some View {
    if let start = SessionClock.parseRFC3339(active.currentItemStartedAt) {
      timerRing(active, start: start)
    }
  }

  @ViewBuilder private func timerRing(_ active: ActiveSessionView, start: Date) -> some View {
    if let held = SessionClock.heldAt(
      stoppedAt: active.reflection?.stoppedAt, reference: referenceDate)
    {
      timerBody(
        elapsed: Int(held.timeIntervalSince(start)),
        planned: active.currentPlannedDurationSecs)
    } else {
      TimelineView(.periodic(from: start, by: 1)) { context in
        timerBody(
          elapsed: Int(context.date.timeIntervalSince(start)),
          planned: active.currentPlannedDurationSecs)
      }
    }
  }

  @ViewBuilder private func timerBody(elapsed: Int, planned: UInt32?) -> some View {
    TimerRing(elapsed: elapsed, planned: planned.map(Int.init))
  }

  // A marking with no BPM, or one outside the click's range (crotchet = 240
  // plays 208), names the click instead (#1942).
  private func clickRow(_ active: ActiveSessionView) -> some View {
    let declared = click.soundsTarget
    return ClickControl(
      bpm: click.bpm, unit: click.metre.unit, step: click.tempoStep, band: click.band,
      isRunning: click.isRunning,
      unavailable: click.unavailable,
      atSeededTempo: click.isAtSeededTempo,
      targetDisplay: declared ? active.currentItemTempoDisplay : nil,
      targetSpoken: declared ? active.currentItemTempoSpoken : nil,
      onToggle: {
        click.toggle()
        recordTempoChange()
      },
      onStep: {
        click.step(by: $0)
        recordTempoChange()
      },
      onDragChange: {
        click.setBpm($0)
        recordTempoChange()
      })
  }

  /// The core keeps the tempo once it settles, so a drag sends every step (#2107).
  private func recordTempoChange() {
    store.send(.session(.tempoChanged(now: SessionClock.nowRFC3339(), reading: tempoReading)))
  }

  // The indicator reads the audio's clock through the engine on every frame,
  // so it cannot drift against the click the way a view timer would (T19).
  // Snapshots and Reduce Motion get a settled frame with no ring.
  @ViewBuilder private var barLine: some View {
    if referenceDate != nil || reduceMotion {
      barLineBody(currentBeat: nil)
    } else {
      TimelineView(.animation(minimumInterval: 1.0 / 30)) { _ in
        barLineBody(currentBeat: click.currentBeat())
      }
    }
  }

  private func barLineBody(currentBeat: Int?) -> some View {
    ClickBarLine(
      metre: click.metre, sounding: click.sounding, currentBeat: currentBeat,
      onTap: { configuringClick = true })
  }

  // ── Repetitions (resident; the core records nothing until the first tap) ──

  private func repCounter(_ active: ActiveSessionView) -> some View {
    RepCounter(
      count: Int(active.currentRepCount ?? 0),
      slots: Int(active.currentRepSlots),
      touched: active.currentRepCount != nil,
      reached: active.currentRepTargetReached ?? false,
      extra: Int(active.currentRepsPastTarget), canUndo: active.currentCanUndo,
      onGotIt: {
        store.send(.session(.repGotIt(now: SessionClock.nowRFC3339(), reading: tempoReading)))
      },
      onNotQuite: {
        store.send(.session(.repMissed(now: SessionClock.nowRFC3339(), reading: tempoReading)))
      },
      onUndo: {
        store.send(.session(.repUndo(now: SessionClock.nowRFC3339(), reading: tempoReading)))
      })
  }

  // ── Bottom: transport (advance + skip-forward) + next-item hint ──

  private func controls(_ active: ActiveSessionView) -> some View {
    VStack(spacing: 14) {
      HStack(spacing: 32) {
        TransportButton(
          systemImage: "play.fill", prominence: .primary,
          label: active.nextItemTitle == nil ? "Finish session" : "Next item"
        ) {
          presentReflection(active)
        }
        .accessibilityIdentifier("player.advance")

        TransportButton(
          systemImage: "forward.end", prominence: .secondary, label: "Skip this item"
        ) {
          store.send(.session(.skipItem(now: SessionClock.nowRFC3339())))
        }
        .accessibilityIdentifier("player.skip")
      }
      if let next = active.nextItemTitle {
        Text("Next · \(next)")
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
    .padding(.bottom, IntradaSpacing.card)
  }

  // ── Reflection at hand-off ───────────────────────────────────────────

  private struct ReflectionTarget: Identifiable {
    let id: String  // the current entry's ulid
    let title: String
    let elapsedDisplay: String
    let tempoTargetBpm: UInt16?
    /// The click at the stamp, kept in the draft so a resume still seeds the
    /// unstamped rows from it with `sheetClick`; `NextItem` sends it again (#1761, #2137).
    let reading: TempoReading
    var startingTempoBpm: Int { Int(reading.bpm) }
    let sheetClick: ClickState
    /// What was played, oldest first, one row per variation (#1739). The last
    /// is the play still open at the moment the item ended, which `NextItem`
    /// then closes.
    let plays: [ReflectionPlay]
    let seed: ReflectionResult
  }

  private var reflectionTarget: ReflectionTarget? {
    guard let active, let draft = active.reflection,
      active.entries.indices.contains(Int(active.currentPosition))
    else { return nil }
    let entry = active.entries[Int(active.currentPosition)]
    // The stamped plays tile the item, and recovery backdates its start by
    // their sum, so this reads the same before and after a resume (#2137).
    let seconds = entry.plays.reduce(0) { $0 + Int($1.seconds) }
    return ReflectionTarget(
      id: entry.id, title: active.currentItemTitle,
      elapsedDisplay: SessionClock.clockDisplay(seconds),
      tempoTargetBpm: active.currentItemTempoBpm, reading: draft.reading,
      sheetClick: ReflectionHandoff.sheetClick(draft.reading, active: active),
      plays: ReflectionPlay.rows(entry.plays),
      seed: ReflectionHandoff.seed(draft.answers))
  }

  private func presentReflection(_ active: ActiveSessionView) {
    let pos = Int(active.currentPosition)
    // Read before stop() below, or every hand-off would read as silent.
    let reading = tempoReading
    guard active.entries.indices.contains(pos) else {
      let now = SessionClock.nowRFC3339()
      store.send(.session(.nextItem(now: now, nextItemStartedAt: now, reading: reading)))
      return
    }
    // The item is over; a click ticking through the rating is keeping time for
    // nothing.
    click.stop()
    // Stamps the open play's real seconds and tempo ahead of the terminal
    // transition, so the sheet's rows can tell which ones the core is about to
    // discard (#1758), and opens the draft that puts the sheet up.
    store.send(.session(.prepareReflection(now: SessionClock.nowRFC3339(), reading: reading)))
  }

  // `now` is ignored while the draft is open: the core closes the entry at
  // the draft's own instant (#2137). A fresh nextItemStartedAt, or the
  // sheet's dwell reads as practice on the item after (#1758).
  private func handleReflection(_ target: ReflectionTarget, _ result: ReflectionResult) {
    let now = SessionClock.nowRFC3339()
    let plan = ReflectionHandoff.plan(
      entryId: target.id, now: now, nextItemStartedAt: now,
      reading: target.reading, plays: target.plays, result: result)
    if !ReflectionHandoff.run(plan, send: store.sendAccepted) { refuse() }
  }

  private func handleSkipRating(_ target: ReflectionTarget) {
    let now = SessionClock.nowRFC3339()
    let accepted = store.sendAccepted(
      .session(.nextItem(now: now, nextItemStartedAt: now, reading: target.reading)))
    if !accepted { refuse() }
  }

  /// Cleared from the core so it does not also wait on the banner behind the sheet (#2009).
  private func refuse() {
    let message = ReflectionHandoff.refusalMessage(
      halted: store.halted, error: store.viewModel?.error)
    withAnimation { reflectionRefusal = message }
    store.send(.clearError)
    Haptic.error.play()
    UIAccessibility.post(notification: .announcement, argument: "Error: \(message)")
  }
}

/// Calm circular timer ring — elapsed time centred, planned arc swept clockwise.
/// Static (no pulse/glow): the player surface should sit still while practice runs.
private struct TimerRing: View {
  let elapsed: Int
  let planned: Int?

  private var fraction: Double {
    guard let planned, planned > 0 else { return 0 }
    return min(Double(elapsed) / Double(planned), 1)
  }

  var body: some View {
    ZStack {
      // The 200 box is the price of the resident counter (T19); the ring had
      // the most slack. The time stays centred at full size.
      ZStack {
        Circle().stroke(IntradaColor.timerTrack, lineWidth: 10)
        if planned != nil {
          Circle()
            .trim(from: 0, to: fraction)
            .stroke(
              LinearGradient.ringSweep,
              style: StrokeStyle(lineWidth: 10, lineCap: .round)
            )
            .rotationEffect(.degrees(-90))
        }
      }
      .padding(18)
      VStack(spacing: 4) {
        Text(SessionClock.clockDisplay(elapsed))
          .font(IntradaFont.timer(48))
          .monospacedDigit()
          .foregroundStyle(IntradaColor.ink)
        if let planned {
          Text("of \(SessionClock.clockDisplay(planned))")
            .font(IntradaFont.secondary)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
    }
    .frame(width: 200, height: 200)
    .accessibilityElement(children: .ignore)
    // Named for the item rather than just "Elapsed": the orientation band now
    // carries a session timer too, so an unqualified label reads as either (T19).
    .accessibilityLabel("This item")
    .accessibilityValue(
      planned == nil
        ? SessionClock.clockDisplay(elapsed)
        : "\(SessionClock.clockDisplay(elapsed)) of \(SessionClock.clockDisplay(planned ?? 0))"
    )
  }
}

#if DEBUG
  #Preview("Untouched") {
    FocusPlayerScreen().environment(Store.previewActive)
  }

  #Preview("Reps") {
    FocusPlayerScreen().environment(Store.previewActiveReps)
  }

  #Preview("Variations") {
    FocusPlayerScreen().environment(Store.previewActiveVariations)
  }
#endif
