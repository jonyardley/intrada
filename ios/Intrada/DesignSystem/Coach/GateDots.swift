import SwiftUI

/// Read-only gate progress, shown *between* reps only — the dots are derived
/// from tap-verdicts, so mid-drill they would be a live score. Indigo, not
/// teal: teal is reserved for the verdict itself. Distinct from `RepCounter`,
/// the manual ± counter in the legacy player.
struct GateDots: View {
  let filled: Int
  let target: Int
  /// Replaces the default "2 of 3" — "gate open", "3 clean at 120".
  var caption: String?
  /// A run-through's per-section verdicts, oldest first (#1256 Phase C). A gate
  /// counts clean reps, so its dots are binary; a run gives every section a
  /// verdict either way, and a broke-down section that drew as "not yet" would
  /// read as a run still to finish. Entries beyond this array are unjudged.
  var verdicts: [Bool]?

  @Environment(\.coachScale) private var scale

  private var clamped: Int { min(max(filled, 0), target) }
  private var text: String { caption ?? "\(clamped) of \(target)" }

  var body: some View {
    HStack(spacing: 10) {
      HStack(spacing: scale.dotGap) {
        ForEach(0..<max(target, 0), id: \.self) { index in
          // Keyed on this dot's own filled-ness: `clamped == index + 1` also
          // flips the dot below it, popping two. The appear-pop lands one
          // stagger after the verdict glyph, per the design's motion.
          dot(verdict(at: index))
            .popOnChange(index < clamped)
            .popOnAppear(index == clamped - 1, delay: IntradaMotion.fadeUpStagger)
        }
      }
      Text(text)
        .font(IntradaFont.ambient(scale.dotCaption))
        .monospacedDigit()
        .foregroundStyle(caption == nil ? IntradaColor.inkSecondary : IntradaColor.ink)
        .dynamicTypeSize(...(.accessibility3))
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(spoken)
  }

  private enum DotState {
    case pending
    case held
    case brokeDown
  }

  private func verdict(at index: Int) -> DotState {
    guard let verdicts else { return index < clamped ? .held : .pending }
    guard index < verdicts.count else { return .pending }
    return verdicts[index] ? .held : .brokeDown
  }

  private var spoken: String {
    if let caption { return caption }
    guard let verdicts else { return "\(clamped) clean of \(target)" }
    let held = verdicts.filter { $0 }.count
    return "\(held) of \(verdicts.count) sections held, \(target) in the piece"
  }

  private func dot(_ state: DotState) -> some View {
    Circle()
      .fill(fill(state))
      .frame(width: scale.dot, height: scale.dot)
      .overlay(
        Circle().strokeBorder(
          state == .pending ? IntradaColor.slotOutline : Color.clear, lineWidth: 2))
  }

  private func fill(_ state: DotState) -> Color {
    switch state {
    case .pending: Color.clear
    case .held: IntradaColor.masteryFill
    // Taupe, never red: a section that broke down is information, not a telling-off.
    case .brokeDown: IntradaColor.repMissedFg
    }
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
        GateDots(filled: 0, target: 3)
        GateDots(filled: 2, target: 3)
        GateDots(filled: 3, target: 3, caption: "gate open")
        GateDots(filled: 3, target: 3, caption: "3 clean at 120")
        GateDots(filled: 2, target: 3)
          .environment(\.coachScale, .regular)
        GateDots(filled: 3, target: 4, caption: "Section 4 of 4", verdicts: [true, false, true])
        GateDots(
          filled: 4, target: 4, caption: "Run complete", verdicts: [true, false, true, true])
      }
    }
  }
#endif
