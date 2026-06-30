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
    HStack(spacing: 5) {
      ForEach(1...10, id: \.self) { value in
        Button {
          onSelect(score == value ? nil : UInt8(value))
        } label: {
          Circle()
            .fill(score >= value ? AnyShapeStyle(IntradaColor.accent) : AnyShapeStyle(.clear))
            .frame(width: 18, height: 18)
            .overlay(
              Circle()
                .stroke(IntradaColor.divider, lineWidth: 1.5)
                .opacity(score >= value ? 0 : 1))
        }
        .buttonStyle(.plain)
      }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
    .accessibilityValue(score == 0 ? "not scored" : "\(score) of 10")
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
