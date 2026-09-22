import SharedTypes
import UIKit

/// Success haptic for an optimistic send: fires only when the core accepted
/// the event, so a rejected mutation never feels like it landed.
enum SuccessFeedback {
  case impact
  case selection
  case success
  case warning

  @MainActor
  fileprivate func fire() {
    switch self {
    case .impact: UIImpactFeedbackGenerator(style: .light).impactOccurred()
    case .selection: UISelectionFeedbackGenerator().selectionChanged()
    case .success: UINotificationFeedbackGenerator().notificationOccurred(.success)
    case .warning: UINotificationFeedbackGenerator().notificationOccurred(.warning)
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

  /// False when the core refused the event (`errorSeq` moved), the bridge
  /// threw, or there is no ViewModel to confirm against (#1937).
  func sendAccepted(_ event: Event) -> Bool {
    confirmed { send(event) }
  }

  /// `sendAccepted` for sends a caller has already wrapped, such as a form's
  /// submit closure.
  func confirmed(_ sends: () -> Void) -> Bool {
    let failuresBefore = bridgeFailureSeq
    let before = viewModel?.errorSeq
    sends()
    guard let before, let after = viewModel?.errorSeq else { return false }
    return bridgeFailureSeq == failuresBefore && after == before
  }
}
