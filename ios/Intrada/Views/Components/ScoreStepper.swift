import SwiftUI

/// A score nudged with minus and plus in thin rings (T39), with a word under
/// the number saying where it came from. The caller owns the score, the range
/// from the core's limits and the write.
struct ScoreStepper: View {
  enum Origin {
    case lastTime, today, suggested

    var caption: String {
      switch self {
      case .lastTime: "last time"
      case .today: "today"
      case .suggested: "suggested"
      }
    }
  }

  let score: Int
  let origin: Origin
  let range: ClosedRange<Int>
  let accessibilityLabel: String
  let onChange: (Int) -> Void

  var body: some View {
    HStack(spacing: 0) {
      TempoStepButton(systemImage: "minus", label: "Lower", isEnabled: canLower) {
        onChange(stepped(by: -1))
      }
      VStack(spacing: 0) {
        Text("\(score)")
          .font(IntradaFont.figureLarge)
          .foregroundStyle(tint)
        Text(origin.caption)
          .font(origin == .today ? IntradaFont.smallMedium : IntradaFont.small)
          .foregroundStyle(tint)
          .lineLimit(1)
          .fixedSize()
      }
      // Keeps a caption that outgrows the column at large text clear of the rings.
      .padding(.horizontal, IntradaSpacing.controlGap)
      .frame(minWidth: StepperValue.width, minHeight: 44)
      TempoStepButton(systemImage: "plus", label: "Higher", isEnabled: canRaise) {
        onChange(stepped(by: 1))
      }
    }
    .fixedSize()
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
    .accessibilityValue(spokenValue)
    .accessibilityAdjustableAction { direction in
      switch direction {
      case .increment where canRaise:
        Haptic.selection.play()
        onChange(stepped(by: 1))
      case .decrement where canLower:
        Haptic.selection.play()
        onChange(stepped(by: -1))
      default:
        break
      }
    }
  }

  var canLower: Bool { score > range.lowerBound }
  var canRaise: Bool { score < range.upperBound }
  var spokenValue: String { "\(score), \(origin.caption)" }

  func stepped(by delta: Int) -> Int {
    min(range.upperBound, max(range.lowerBound, score + delta))
  }

  private var tint: Color { origin == .today ? IntradaColor.ink : IntradaColor.inkSecondary }
}

#if DEBUG
  #Preview("Score stepper") {
    VStack(spacing: IntradaSpacing.section) {
      ScoreStepper(
        score: 5, origin: .lastTime, range: 1...10, accessibilityLabel: "Score for D major"
      ) { _ in }
      ScoreStepper(
        score: 6, origin: .today, range: 1...10, accessibilityLabel: "Score for D major"
      ) { _ in }
      ScoreStepper(
        score: 10, origin: .suggested, range: 1...10, accessibilityLabel: "Score for D major"
      ) { _ in }
    }
    .padding(IntradaSpacing.section)
    .background(IntradaColor.paperTop)
  }
#endif
