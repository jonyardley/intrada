import SwiftUI

/// Read-only gate progress, shown *between* reps only — the dots are derived
/// from tap-verdicts, so mid-drill they would be a live score (`Drill Loop`,
/// A2). Filled dots are `masteryFill` indigo, not teal: this is progress, and
/// teal is reserved for the verdict itself. Distinct from `RepCounter`, the
/// manual ± counter in the legacy player.
struct GateDots: View {
  let filled: Int
  let target: Int
  /// Replaces the default "2 of 3" — "gate open", "3 clean at 120".
  var caption: String?

  @Environment(\.coachScale) private var scale

  private var clamped: Int { min(max(filled, 0), target) }
  private var text: String { caption ?? "\(clamped) of \(target)" }

  var body: some View {
    HStack(spacing: 10) {
      HStack(spacing: scale.dotGap) {
        ForEach(0..<max(target, 0), id: \.self) { index in
          dot(done: index < clamped)
            .popOnChange(clamped == index + 1)
        }
      }
      Text(text)
        .font(IntradaFont.ambient(scale.dotCaption))
        .monospacedDigit()
        .foregroundStyle(caption == nil ? IntradaColor.inkSecondary : IntradaColor.ink)
        .dynamicTypeSize(...(.accessibility3))
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(caption ?? "\(clamped) clean of \(target)")
  }

  private func dot(done: Bool) -> some View {
    Circle()
      .fill(done ? IntradaColor.masteryFill : Color.clear)
      .frame(width: scale.dot, height: scale.dot)
      .overlay(
        Circle().strokeBorder(done ? Color.clear : IntradaColor.slotOutline, lineWidth: 2))
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
      }
    }
  }
#endif
