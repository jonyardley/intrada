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

  @State private var reflecting: ReflectionTarget?
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
    .sheet(item: $reflecting) { target in
      ReflectionSheet(
        itemTitle: target.title, elapsedDisplay: target.elapsedDisplay,
        tempoTarget: target.tempoTargetBpm, startingTempoBpm: target.startingTempoBpm,
        tempoUnit: target.tempoUnit, currentClick: target.reading.click, plays: target.plays,
        refusal: target.refusal,
        onSave: { result in handleReflection(target, result) },
        onSkip: { handleSkipRating(target) }
      )
      .presentationDetents([.medium, .large])
      .interactiveDismissDisabled()
    }
    .sheet(isPresented: $configuringClick) {
      if let limits = store.viewModel?.limits {
        ClickSheet(click: click, bpm: click.bpm, limits: limits)
      }
    }
    .sheet(isPresented: $switchingVariation) {
      if let active {
        VariationPickerSheet(
          itemTitle: active.currentItemTitle,
          currentVariations: active.currentVariations,
          currentVariationId: active.currentVariationId,
          onPick: { switchVariation(active, to: $0) })
      }
    }
    .task { click.reseed(target: active?.currentItemTempoBpm, metre: active?.currentItemMetre) }
    .onChange(of: active?.currentPosition) { _, _ in
      click.reseed(target: active?.currentItemTempoBpm, metre: active?.currentItemMetre)
    }
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
    "FOCUS · \(active.currentPosition + 1) OF \(active.totalItems)"
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
        .font(IntradaFont.pageTitle(34))
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.center)
      if let pieceTitle = active.currentRelatedPieceTitle {
        Label("Related to \(pieceTitle)", systemImage: "arrow.turn.down.right")
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.accent)
      }
      if let aim = active.currentItemIntention, !aim.isEmpty {
        Text("Aim: \(aim)")
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
      }
      if let notes = active.currentItemNotes, !notes.isEmpty {
        Text("Notes: \(notes)")
          .font(IntradaFont.meta)
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
          Text(active.currentVariationLabel ?? "Pick a variation")
            .font(IntradaFont.segment)
            .foregroundStyle(
              active.currentVariationLabel == nil
                ? IntradaColor.inkSecondary : IntradaColor.ink
            )
            // A variation named "2nd inversion" shrinks rather than
            // ellipsising, as the click's readout does (T19).
            .lineLimit(1)
            .minimumScaleFactor(0.7)
          Image(systemName: "chevron.down")
            .font(IntradaFont.micro.weight(.semibold))
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
      .accessibilityValue(active.currentVariationLabel ?? "none picked")
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
        .switchVariation(
          entryId: active.entries[pos].id, variationId: variationId,
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
    if let referenceDate {
      timerBody(
        elapsed: Int(referenceDate.timeIntervalSince(start)),
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
      bpm: click.bpm, unit: click.metre.unit, isRunning: click.isRunning,
      unavailable: click.unavailable,
      atSeededTempo: click.isAtSeededTempo,
      targetDisplay: declared ? active.currentItemTempoDisplay : nil,
      targetSpoken: declared ? active.currentItemTempoSpoken : nil,
      onToggle: { click.toggle() },
      onStep: { click.step(by: $0) },
      onDragChange: { click.setBpm($0) })
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
      onGotIt: { store.send(.session(.repGotIt(now: SessionClock.nowRFC3339()))) },
      onNotQuite: { store.send(.session(.repMissed(now: SessionClock.nowRFC3339()))) })
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
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
    .padding(.bottom, IntradaSpacing.card)
  }

  // ── Reflection at hand-off ───────────────────────────────────────────

  private struct ReflectionTarget: Identifiable {
    let id: String  // the current entry's ulid
    let title: String
    let elapsedDisplay: String?
    let tempoTargetBpm: UInt16?
    /// The click at the moment the item ended, read before it was stopped:
    /// `PrepareReflection` stamps from it and `NextItem` must send it again
    /// (#1761).
    let reading: TempoReading
    var startingTempoBpm: Int { Int(reading.bpm) }
    /// The unit the stepper counts in, which is the click's when the player
    /// chose one and crotchets when they did not.
    var tempoUnit: UInt8 { reading.click?.metre.unit ?? 4 }
    /// What was played, oldest first, one row per variation (#1739). The last
    /// is the play still open at the moment the item ended, which `NextItem`
    /// then closes.
    let plays: [ReflectionPlay]
    /// `PrepareReflection`'s instant: `NextItem`'s own `now` must reuse it, or the rows above stop predicting the drop (#1758).
    let now: String
    var refusal: String?
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
    let elapsed = SessionClock.parseRFC3339(active.currentItemStartedAt).map {
      max(Int((referenceDate ?? Date()).timeIntervalSince($0)), 0)
    }
    // The item is over; a click ticking through the rating is keeping time for
    // nothing.
    click.stop()
    let entry = active.entries[pos]
    let now = SessionClock.nowRFC3339()
    // Stamps the open play's real seconds and tempo ahead of the terminal
    // transition, so the rows below can tell which ones the core is about to
    // discard (#1758).
    store.send(.session(.prepareReflection(now: now, reading: reading)))
    let stamped =
      store.viewModel?.activeSession?.entries.first(where: { $0.id == entry.id })?.plays
      ?? entry.plays
    reflecting = ReflectionTarget(
      id: entry.id, title: active.currentItemTitle,
      elapsedDisplay: elapsed.map(SessionClock.clockDisplay),
      tempoTargetBpm: active.currentItemTempoBpm, reading: reading,
      plays: ReflectionPlay.rows(stamped), now: now)
  }

  private func handleReflection(_ target: ReflectionTarget, _ result: ReflectionResult) {
    // A fresh nextItemStartedAt, or the sheet's dwell reads as practice on the item after (#1758).
    let plan = ReflectionHandoff.plan(
      entryId: target.id, now: target.now, nextItemStartedAt: SessionClock.nowRFC3339(),
      reading: target.reading, plays: target.plays, result: result)
    if ReflectionHandoff.run(plan, send: store.sendAccepted) { reflecting = nil } else { refuse() }
  }

  private func handleSkipRating(_ target: ReflectionTarget) {
    let accepted = store.sendAccepted(
      .session(
        .nextItem(
          now: target.now, nextItemStartedAt: SessionClock.nowRFC3339(), reading: target.reading)))
    if accepted { reflecting = nil } else { refuse() }
  }

  /// Cleared from the core so it does not also wait on the banner behind the sheet (#2009).
  private func refuse() {
    let message = ReflectionHandoff.refusalMessage(
      halted: store.halted, error: store.viewModel?.error)
    withAnimation { reflecting?.refusal = message }
    store.send(.clearError)
    UINotificationFeedbackGenerator().notificationOccurred(.error)
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
            .font(IntradaFont.meta)
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
