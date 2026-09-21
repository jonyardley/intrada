import SharedTypes
import UIKit

/// Success haptic for an optimistic send: fires only when the core accepted
/// the event (`errorSeq` unchanged), so a rejected mutation never feels like
/// it landed — the surface-don't-swallow rule for feedback.
enum SuccessFeedback {
  case impact
  case selection

  @MainActor
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
    let accepted = sendAccepted(event)
    if accepted { feedback.fire() }
    return accepted
  }

  /// False when the core refused the event (`errorSeq` moved).
  func sendAccepted(_ event: Event) -> Bool {
    let before = viewModel?.errorSeq
    send(event)
    return viewModel?.errorSeq == before
  }
}
