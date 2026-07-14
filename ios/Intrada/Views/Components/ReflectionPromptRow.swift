import SwiftUI

/// One structured end-of-session prompt (Improved / Still rough / Next
/// target — design-principles T7, DECISIONS.md surface 3): a tinted icon
/// circle, an uppercase label, and a single-line editable value. Reused
/// three times on `SessionSummaryScreen`, hence its own component rather
/// than inline (DECISIONS.md fold-in list item 3).
struct ReflectionPromptRow: View {
  let icon: String
  let tint: Color
  let tintBg: Color
  let label: String
  let placeholder: String
  @Binding var text: String

  var body: some View {
    HStack(alignment: .top, spacing: IntradaSpacing.controlGap) {
      ZStack {
        Circle().fill(tintBg)
        Image(systemName: icon)
          .font(.system(size: 13, weight: .semibold))
          .foregroundStyle(tint)
      }
      .frame(width: 28, height: 28)

      VStack(alignment: .leading, spacing: 3) {
        Eyebrow(label)
        TextField(placeholder, text: $text)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.ink)
      }
    }
    .padding(.vertical, IntradaSpacing.controlGap)
    .accessibilityElement(children: .combine)
  }
}

#if DEBUG
  #Preview("Reflection prompt rows") {
    VStack(alignment: .leading, spacing: 0) {
      ReflectionPromptRow(
        icon: "arrow.up.right", tint: IntradaColor.repCleanFg, tintBg: IntradaColor.repCleanBg,
        label: "Improved", placeholder: "What moved forward?",
        text: .constant("Thumb-unders even at 92."))
      Rectangle().fill(IntradaColor.hairline).frame(height: 1)
      ReflectionPromptRow(
        icon: "wrench.fill", tint: IntradaColor.exerciseBadgeFg,
        tintBg: IntradaColor.exerciseBadgeBg,
        label: "Still rough", placeholder: "What's still fighting you?",
        text: .constant(""))
      Rectangle().fill(IntradaColor.hairline).frame(height: 1)
      ReflectionPromptRow(
        icon: "target", tint: IntradaColor.pieceBadgeFg, tintBg: IntradaColor.pieceBadgeBg,
        label: "Next target", placeholder: "Where does the next session start?",
        text: .constant(""))
    }
    .padding()
    .background(IntradaColor.paperTop)
  }
#endif
