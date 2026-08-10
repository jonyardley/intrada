import SharedTypes
import SwiftUI

/// B1 — the gated run: one "Held / Broke down" tap per named chart section, in
/// reading order. A peer of the drill loop, not a block of it: no gate, no
/// ladder, no ceiling, because the piece is what bounds it.
struct RunThroughScreen: View {
  let state: RunThroughView
  /// `true` = "Yes — held", `false` = "No — broke down".
  var onVerdict: (Bool) -> Void
  /// B1's "Don't count this run": the time is still logged, the evidence is not.
  var onDiscard: () -> Void
  var onFinish: () -> Void
  var onDismiss: () -> Void

  @Environment(\.horizontalSizeClass) private var sizeClass
  @Environment(\.dynamicTypeSize) private var typeSize

  private var scale: CoachScale { sizeClass == .regular ? .regular : .compact }
  private var gutter: CGFloat { scale == .compact ? IntradaSpacing.card : IntradaSpacing.stage }
  /// The tune line gives way first; the section, the question and the pair
  /// survive at any size.
  private var showsIdentityDetail: Bool { !typeSize.isAccessibilitySize }

  private var sectionCount: Int { state.sections.count }
  private var position: Int { min(state.held.count + 1, max(sectionCount, 1)) }

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: 0) {
        OrientationStrip(
          elapsedSeconds: Int(state.elapsedSeconds),
          blockKinds: [], blockIndex: 0, altitude: .runThrough,
          onDismiss: onDismiss)
        identity
        Spacer(minLength: IntradaSpacing.card)
        centre
        Spacer(minLength: IntradaSpacing.card)
        footer
      }
      .padding(.horizontal, gutter)
      .padding(.bottom, IntradaSpacing.section)
    }
    .environment(\.coachScale, scale)
    .dynamicTypeSize(.xSmall ... .accessibility5)
  }

  // ── Identity: which piece, and where in it ──

  private var identity: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      if showsIdentityDetail {
        TypeBadge(kind: ItemKind.piece, label: "Play it through")
      }
      Text(state.title)
        .font(IntradaFont.drillTitle(scale.drillTitle))
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)
    }
    .padding(.top, IntradaSpacing.section)
  }

  // ── Centre: the section the next tap judges ──

  @ViewBuilder private var centre: some View {
    VStack(spacing: IntradaSpacing.section) {
      if let section = state.currentSection {
        Text(section)
          .font(IntradaFont.verdict(scale.question))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
          .accessibilityLabel("Section \(section)")
        Text("Did it hold?")
          .font(IntradaFont.ambient(scale == .compact ? 16 : 22))
          .foregroundStyle(IntradaColor.inkSecondary)
      } else {
        // Every section judged, and still nothing written: the exit stays an
        // explicit tap so "Don't count this run" is reachable after the last
        // verdict (B1).
        RepVerdict(outcome: .clean, fact: "Run complete")
      }
      GateDots(
        filled: state.held.count, target: sectionCount, caption: dotsCaption,
        verdicts: state.held)
    }
  }

  private var dotsCaption: String {
    guard state.currentSection != nil else { return heldSummary }
    return "Section \(position) of \(sectionCount)"
  }

  private var heldSummary: String {
    let held = state.held.filter { $0 }.count
    return "\(held) of \(sectionCount) held"
  }

  // ── Footer: the verdict pair, then the quiet way out ──

  @ViewBuilder private var footer: some View {
    VStack(spacing: IntradaSpacing.card) {
      if state.currentSection != nil {
        TapVerdict(onClean: { onVerdict(true) }, onMissed: { onVerdict(false) }, subject: .section)
      } else {
        BrandBarButton(prominent: true, action: onFinish) {
          Text("Log this run")
        }
      }
      Button("Don't count this run", action: onDiscard)
        .buttonStyle(QuietAction())
        .accessibilityHint("Keeps the time, records no verdicts against the piece")
    }
  }
}

#if DEBUG
  extension RunThroughView {
    /// The design's worked example: *Alice in Wonderland*, an AABA-ish chart. A
    /// fixture for previews and snapshots only — at runtime every field comes
    /// from the core's `CoachView`.
    static func preview(
      held: [Bool] = [true, false], elapsedSeconds: UInt32 = 252,
      sections: [String] = ["A", "B", "Bridge", "A'"]
    ) -> RunThroughView {
      RunThroughView(
        title: "Alice in Wonderland",
        sections: sections,
        held: held,
        currentSection: held.count < sections.count ? sections[held.count] : nil,
        complete: held.count >= sections.count,
        elapsedSeconds: elapsedSeconds)
    }
  }

  private struct RunThroughPreview: View {
    let state: RunThroughView
    var body: some View {
      RunThroughScreen(
        state: state, onVerdict: { _ in }, onDiscard: {}, onFinish: {}, onDismiss: {})
    }
  }

  #Preview("Mid-run") { RunThroughPreview(state: .preview()) }

  #Preview("Every section judged") {
    RunThroughPreview(state: .preview(held: [true, false, true, true], elapsedSeconds: 448))
  }

  #Preview("Largest accessibility size") {
    RunThroughPreview(state: .preview()).environment(\.dynamicTypeSize, .accessibility5)
  }
#endif
