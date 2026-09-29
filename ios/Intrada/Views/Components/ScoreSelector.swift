import SharedTypes
import SwiftUI

/// One pill per mark the core accepts (#2042), filled up to the chosen one;
/// tapping the chosen one clears it. The caller owns the score and the write.
struct ScoreSelector: View {
  /// 0 means unscored — no pills filled.
  let score: Int
  let range: ClosedRange<Int>
  let accessibilityLabel: String
  /// `nil` clears the score (tapping the current value).
  let onSelect: (UInt8?) -> Void

  var body: some View {
    HStack(spacing: 4) {
      ForEach(pillValues, id: \.self) { pill($0) }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
    .accessibilityValue(spokenValue)
    .accessibilityAdjustableAction { direction in
      switch direction {
      case .increment:
        guard let markAbove else { return }
        Haptic.selection.play()
        onSelect(markAbove)
      case .decrement where score > 0:
        Haptic.selection.play()
        onSelect(markBelow)
      default:
        break
      }
    }
  }

  var pillValues: [Int] { Array(range) }

  var spokenValue: String { score == 0 ? "not marked" : "\(score) of \(range.upperBound)" }

  /// Unmarked starts at the lowest mark, which need not be 1.
  var markAbove: UInt8? { score < range.upperBound ? UInt8(max(score + 1, range.lowerBound)) : nil }

  /// `nil` clears: at or under the lowest mark there is no lower one to send.
  var markBelow: UInt8? {
    score <= range.lowerBound ? nil : UInt8(min(score - 1, range.upperBound))
  }

  private func pill(_ value: Int) -> some View {
    let filled = score >= value
    return Button {
      Haptic.selection.play()
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

extension LimitsView {
  var scoreRange: ClosedRange<Int> { Int(scoreMin)...Int(scoreMax) }
}

#if DEBUG
  #Preview("Score selector") {
    VStack(alignment: .leading, spacing: 24) {
      ScoreSelector(score: 0, range: 1...10, accessibilityLabel: "Score") { _ in }
      ScoreSelector(score: 4, range: 1...10, accessibilityLabel: "Score") { _ in }
      ScoreSelector(score: 10, range: 1...10, accessibilityLabel: "Score") { _ in }
    }
    .padding()
    .background(IntradaColor.paperTop)
  }
#endif
