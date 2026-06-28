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

  private var active: ActiveSessionView? { store.viewModel?.activeSession }

  var body: some View {
    ZStack {
      PaperBackground()
      if let active {
        content(active)
      }
    }
  }

  private func content(_ active: ActiveSessionView) -> some View {
    VStack(spacing: 0) {
      topChrome(active)
      Spacer(minLength: IntradaSpacing.card)
      centerInfo(active)
      if active.currentRepTarget != nil {
        repCard(active).padding(.top, 28)
      }
      Spacer(minLength: IntradaSpacing.card)
      controls(active)
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
        total: Int(active.totalItems),
        filled: min(Int(active.currentPosition) + 1, Int(active.totalItems)))
    }
  }

  private func positionLabel(_ active: ActiveSessionView) -> String {
    "\(active.currentPosition + 1) OF \(active.totalItems)"
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
      VStack(spacing: 6) {
        Text(active.currentItemTitle)
          .font(IntradaFont.pageTitle(34))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
        Text(active.currentItemType.label.uppercased())
          .font(IntradaFont.badge)
          .tracking(1.5)
          .foregroundStyle(IntradaColor.inkFaint)
      }
      timer(active)
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
    VStack(spacing: 12) {
      Text(SessionClock.clockDisplay(elapsed))
        .font(IntradaFont.timer())
        .monospacedDigit()
        .foregroundStyle(IntradaColor.ink)
        .accessibilityLabel("Elapsed \(SessionClock.clockDisplay(elapsed))")
      if let planned {
        targetBar(elapsed: elapsed, planned: Int(planned))
      }
    }
  }

  private func targetBar(elapsed: Int, planned: Int) -> some View {
    let fraction = planned > 0 ? min(Double(elapsed) / Double(planned), 1) : 0
    return VStack(spacing: 8) {
      GeometryReader { geo in
        ZStack(alignment: .leading) {
          Capsule().fill(IntradaColor.divider)
          Capsule().fill(LinearGradient.brandBar).frame(width: geo.size.width * fraction)
        }
      }
      .frame(width: 180, height: 5)
      Text("of \(SessionClock.clockDisplay(planned))")
        .font(IntradaFont.meta)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Target \(SessionClock.clockDisplay(planned))")
  }

  // ── Reps (only when the current item has a target) ──

  private func repCard(_ active: ActiveSessionView) -> some View {
    let count = active.currentRepCount.map(Int.init) ?? 0
    let target = active.currentRepTarget.map(Int.init) ?? 0
    let reached = active.currentRepTargetReached ?? false
    return VStack(spacing: 12) {
      Text("CONSECUTIVE REPS")
        .font(IntradaFont.micro).fontWeight(.semibold).tracking(1.5)
        .foregroundStyle(IntradaColor.inkFaint)
      Text("\(count) / \(target)")
        .font(IntradaFont.cardTitle(20))
        .monospacedDigit()
        .foregroundStyle(IntradaColor.ink)
      HStack(spacing: IntradaSpacing.controlGap) {
        repButton("Got it", filled: true) { store.send(.session(.repGotIt)) }
        repButton("Missed", filled: false) { store.send(.session(.repMissed)) }
      }
      .disabled(reached)
      .opacity(reached ? 0.5 : 1)
    }
    .padding(IntradaSpacing.card)
    .frame(maxWidth: .infinity)
    .background(IntradaColor.cardFill)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .stroke(IntradaColor.hairline, lineWidth: 1))
  }

  private func repButton(_ title: String, filled: Bool, action: @escaping () -> Void)
    -> some View
  {
    Button(action: action) {
      Text(title)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(filled ? IntradaColor.onAccent : IntradaColor.inkSecondary)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.cardCompact)
        .background(filled ? AnyShapeStyle(LinearGradient.brandBar) : AnyShapeStyle(.clear))
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.card)
            .stroke(filled ? .clear : IntradaColor.hairline, lineWidth: 1))
    }
    .buttonStyle(.plain)
  }

  // ── Bottom: the one primary action + next-item hint ──

  private func controls(_ active: ActiveSessionView) -> some View {
    VStack(spacing: 10) {
      Button {
        store.send(.session(.nextItem(now: SessionClock.nowRFC3339())))
      } label: {
        Text(active.nextItemTitle == nil ? "Finish session" : "Done")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.onAccent)
          .frame(maxWidth: .infinity)
          .padding(.vertical, IntradaSpacing.row)
          .background(LinearGradient.brandBar)
          .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
      }
      .buttonStyle(.plain)
      if let next = active.nextItemTitle {
        Text("Next · \(next)")
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
    .padding(.bottom, IntradaSpacing.card)
  }
}

/// Discrete session-position indicator — N filled segments of M. Stepped (not a
/// continuous fill) so it reads as "which item", distinct from the timer's
/// continuous target bar.
struct SegmentedProgress: View {
  let total: Int
  let filled: Int

  var body: some View {
    HStack(spacing: 5) {
      ForEach(0..<max(total, 1), id: \.self) { index in
        Capsule()
          .fill(
            index < filled
              ? AnyShapeStyle(LinearGradient.brandBar) : AnyShapeStyle(IntradaColor.divider)
          )
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
