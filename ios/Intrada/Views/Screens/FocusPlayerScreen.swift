import SharedTypes
import SwiftUI

/// The live, while-playing surface (the player's Focus screen). Renders the
/// core's `ActiveSessionView`; every control sends a `SessionEvent` and the core
/// drives the transition (Done on the last item → Summary). Full-screen, no
/// chrome — "the app disappears during practice".
struct FocusPlayerScreen: View {
  @Environment(Store.self) private var store

  // Snapshots inject a fixed instant so the timer is deterministic; production
  // passes nil and the timer ticks off the wall clock (mirrors PracticeScreen).
  private let referenceDate: Date?

  init(referenceDate: Date? = nil) { self.referenceDate = referenceDate }

  @State private var reflecting: ReflectionTarget?

  private var active: ActiveSessionView? { store.viewModel?.activeSession }

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
        tempoTarget: target.tempoTargetBpm,
        onSave: { score, note, achievedTempo in
          handleReflection(target, score: score, note: note, achievedTempo: achievedTempo)
        },
        onSkip: { handleSkipRating() }
      )
      .presentationDetents([.medium, .large])
    }
  }

  private func content(_ active: ActiveSessionView) -> some View {
    VStack(spacing: 0) {
      topChrome(active).fadeUp(0)
      Spacer(minLength: IntradaSpacing.card)
      centerInfo(active).fadeUp(1)
      timer(active).fadeUp(2).padding(.top, IntradaSpacing.section)
      if let tempo = active.currentItemTempoDisplay {
        Label(tempo, systemImage: "metronome")
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
          .padding(.top, IntradaSpacing.controlGap)
      }
      if active.currentRepTarget != nil {
        repCounter(active).fadeUp(3).padding(.top, 28)
      }
      Spacer(minLength: IntradaSpacing.card)
      controls(active).fadeUp(4)
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.card)
  }

  // ── Top: position label + segmented session progress + options menu ──

  private func topChrome(_ active: ActiveSessionView) -> some View {
    VStack(spacing: 12) {
      HStack {
        Color.clear.frame(width: 28, height: 1)  // balances the menu so the label centres
        Spacer()
        Text(positionLabel(active))
          .font(IntradaFont.badge)
          .tracking(1.5)
          .foregroundStyle(IntradaColor.inkFaint)
        Spacer()
        optionsMenu
      }
      SegmentedProgress(
        types: active.entries.map(\.itemType),
        filled: min(Int(active.currentPosition) + 1, Int(active.totalItems)))
    }
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
        store.send(.session(.endSessionEarly(now: SessionClock.nowRFC3339())))
      } label: {
        Label("End session early", systemImage: "stop.circle")
      }
    } label: {
      Image(systemName: "ellipsis")
        .font(.system(size: 20))
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(width: 28, height: 28)
    }
    .accessibilityLabel("Session options")
  }

  // ── Centre: intention echo, item identity, the live timer ──

  private func centerInfo(_ active: ActiveSessionView) -> some View {
    VStack(spacing: 28) {
      if let intention = active.sessionIntention, !intention.isEmpty {
        Text("“\(intention)”")
          .font(IntradaFont.body).italic()
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
      }
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
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
  }

  @ViewBuilder private func timer(_ active: ActiveSessionView) -> some View {
    let start = SessionClock.parseRFC3339(active.currentItemStartedAt) ?? Date()
    if let referenceDate {
      timerBody(
        elapsed: Int(referenceDate.timeIntervalSince(start)),
        planned: active.currentPlannedDurationSecs)
    } else {
      TimelineView(.periodic(from: .now, by: 1)) { context in
        timerBody(
          elapsed: Int(context.date.timeIntervalSince(start)),
          planned: active.currentPlannedDurationSecs)
      }
    }
  }

  @ViewBuilder private func timerBody(elapsed: Int, planned: UInt32?) -> some View {
    TimerRing(elapsed: elapsed, planned: planned.map(Int.init))
  }

  // ── Reps (only when the current item has a target) ──

  private func repCounter(_ active: ActiveSessionView) -> some View {
    RepCounter(
      count: Int(active.currentRepCount ?? 0),
      target: Int(active.currentRepTarget ?? 0),
      onClean: { store.send(.session(.repGotIt)) },
      onMissed: { store.send(.session(.repMissed)) })
  }

  // ── Bottom: transport (advance + skip-forward) + next-item hint ──

  private func controls(_ active: ActiveSessionView) -> some View {
    VStack(spacing: 14) {
      HStack(spacing: 32) {
        Button {
          presentReflection(active)
        } label: {
          Image(systemName: "play.fill")
            .font(.system(size: 32))
            .foregroundStyle(IntradaColor.onAccent)
            .frame(width: 78, height: 78)
            .background(LinearGradient.brandBar)
            .clipShape(Circle())
            .shadow(color: IntradaColor.ink.opacity(0.18), radius: 14, y: 6)
        }
        .buttonStyle(PressRebound())
        .accessibilityLabel(active.nextItemTitle == nil ? "Finish session" : "Next item")

        Button {
          store.send(.session(.skipItem(now: SessionClock.nowRFC3339())))
        } label: {
          Image(systemName: "forward.end")
            .font(.system(size: 22))
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(width: 48, height: 48)
        }
        .buttonStyle(PressRebound())
        .accessibilityLabel("Skip this item")
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
    let elapsedDisplay: String
    /// The item's own declared tempo (the practice target), distinct from
    /// `achievedTempo` logged after the fact. `nil` hides the tempo stepper.
    let tempoTargetBpm: UInt16?
  }

  private func presentReflection(_ active: ActiveSessionView) {
    let pos = Int(active.currentPosition)
    guard active.entries.indices.contains(pos) else {
      store.send(.session(.nextItem(now: SessionClock.nowRFC3339())))
      return
    }
    let start = SessionClock.parseRFC3339(active.currentItemStartedAt) ?? Date()
    let elapsed = max(Int((referenceDate ?? Date()).timeIntervalSince(start)), 0)
    reflecting = ReflectionTarget(
      id: active.entries[pos].id, title: active.currentItemTitle,
      elapsedDisplay: SessionClock.clockDisplay(elapsed),
      tempoTargetBpm: active.currentItemTempoBpm)
  }

  // Notes first (no status guard — surfaces a validation error before advancing);
  // then NextItem completes the entry so the score and tempo can land (both
  // need Completed). Errors surface on RootView's banner, so dismiss only on
  // success.
  private func handleReflection(
    _ target: ReflectionTarget, score: UInt8?, note: String, achievedTempo: UInt16?
  ) {
    if !note.isEmpty {
      let before = store.viewModel?.errorSeq
      store.send(.session(.updateEntryNotes(entryId: target.id, notes: note)))
      if store.viewModel?.errorSeq != before { return }
    }
    store.send(.session(.nextItem(now: SessionClock.nowRFC3339())))
    if let score {
      store.send(.session(.updateEntryScore(entryId: target.id, score: score)))
    }
    if let achievedTempo {
      store.send(.session(.updateEntryTempo(entryId: target.id, tempo: achievedTempo)))
    }
    reflecting = nil
  }

  private func handleSkipRating() {
    store.send(.session(.nextItem(now: SessionClock.nowRFC3339())))
    reflecting = nil
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
      // Inset the ring to r≈100 within the 236 box (design geometry); the time
      // stays centred at full size.
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
    .frame(width: 236, height: 236)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(
      planned == nil
        ? "Elapsed \(SessionClock.clockDisplay(elapsed))"
        : "Elapsed \(SessionClock.clockDisplay(elapsed)) of \(SessionClock.clockDisplay(planned ?? 0))"
    )
  }
}

/// Discrete session-position indicator — N filled segments of M, one per
/// setlist entry. Stepped (not a continuous fill) so it reads as "which item",
/// distinct from the timer's continuous target bar. Completed segments are
/// tinted by that entry's item type (piece vs. exercise) rather than a single
/// brand gradient, so the strip doubles as a glance-able session shape.
struct SegmentedProgress: View {
  let types: [ItemKind]
  let filled: Int

  private var total: Int { types.count }

  var body: some View {
    HStack(spacing: 5) {
      ForEach(Array(types.enumerated()), id: \.offset) { index, type in
        Capsule()
          .fill(index < filled ? AnyShapeStyle(type.accent) : AnyShapeStyle(IntradaColor.divider))
          .frame(height: 4)
      }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Item \(filled) of \(total)")
  }
}

#if DEBUG
  #Preview("No reps") {
    FocusPlayerScreen().environment(Store.previewActive)
  }

  #Preview("Reps") {
    FocusPlayerScreen().environment(Store.previewActiveReps)
  }
#endif
