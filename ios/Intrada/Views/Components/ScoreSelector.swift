import SwiftUI

/// The 0–10 score control: ten tappable pills with a cumulative fill up to the
/// chosen value. Tapping the current value clears it (`nil`). Feeds the
/// `ScoreRing` everywhere a score is set — per-item and overall at session
/// hand-off. Pure presentation; the caller owns the score and the write.
struct ScoreSelector: View {
  /// 0 means unscored — no pills filled.
  let score: Int
  let accessibilityLabel: String
  /// `nil` clears the score (tapping the current value).
  let onSelect: (UInt8?) -> Void

  var body: some View {
    HStack(spacing: 4) {
      ForEach(1...10, id: \.self) { pill($0) }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
    .accessibilityValue(score == 0 ? "not scored" : "\(score) of 10")
    .accessibilityAdjustableAction { direction in
      switch direction {
      case .increment where score < 10:
        UISelectionFeedbackGenerator().selectionChanged()
        onSelect(UInt8(score + 1))
      case .decrement where score > 0:
        UISelectionFeedbackGenerator().selectionChanged()
        onSelect(score == 1 ? nil : UInt8(score - 1))
      default:
        break
      }
    }
  }

  private func pill(_ value: Int) -> some View {
    let filled = score >= value
    return Button {
      UISelectionFeedbackGenerator().selectionChanged()
      onSelect(score == value ? nil : UInt8(value))
    } label: {
      Text("\(value)")
        .font(IntradaFont.badge)
        .foregroundStyle(filled ? IntradaColor.onAccent : IntradaColor.inkSecondary)
        .frame(maxWidth: .infinity)
        .frame(height: 32)
        .background(
          RoundedRectangle(cornerRadius: IntradaRadius.badge)
            .fill(filled ? AnyShapeStyle(IntradaColor.accent) : AnyShapeStyle(Color.clear))
        )
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.badge)
            .strokeBorder(IntradaColor.slotOutline, lineWidth: 1.5)
            .opacity(filled ? 0 : 1))
    }
    .buttonStyle(.plain)
  }
}

#if DEBUG
  #Preview("Score selector") {
    VStack(alignment: .leading, spacing: 24) {
      ScoreSelector(score: 0, accessibilityLabel: "Score") { _ in }
      ScoreSelector(score: 4, accessibilityLabel: "Score") { _ in }
      ScoreSelector(score: 10, accessibilityLabel: "Score") { _ in }
    }
    .padding()
    .background(IntradaColor.paperTop)
  }
#endif
