import SharedTypes
import UIKit

/// Success haptic for an optimistic send: fires only when the core accepted
/// the event (`errorSeq` unchanged), so a rejected mutation never feels like
/// it landed — the surface-don't-swallow rule for feedback.
enum SuccessFeedback {
  case impact
  case selection

  fileprivate func fire() {
    switch self {
    case .impact: UIImpactFeedbackGenerator(style: .light).impactOccurred()
    case .selection: UISelectionFeedbackGenerator().selectionChanged()
    }
  }
}

extension Store {
  @discardableResult
  func send(_ event: Event, onSuccess feedback: SuccessFeedback) -> Bool {
    let before = viewModel?.errorSeq
    send(event)
    let accepted = viewModel?.errorSeq == before
    if accepted { feedback.fire() }
    return accepted
  }
}
