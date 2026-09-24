import SharedTypes
import UIKit

/// The one place a haptic is built (#2013): `scripts/check-haptics.sh` fails
/// any other file that makes a feedback generator.
enum Haptic {
  case impact
  case selection
  case success
  case warning
  case error

  @MainActor
  func play() {
    switch self {
    case .impact: UIImpactFeedbackGenerator(style: .light).impactOccurred()
    case .selection: UISelectionFeedbackGenerator().selectionChanged()
    case .success: UINotificationFeedbackGenerator().notificationOccurred(.success)
    case .warning: UINotificationFeedbackGenerator().notificationOccurred(.warning)
    case .error: UINotificationFeedbackGenerator().notificationOccurred(.error)
    }
  }
}

extension Store {
  /// Plays the haptic only when the core accepted the event, so a rejected
  /// mutation never feels like it landed.
  @discardableResult
  func send(_ event: Event, onSuccess haptic: Haptic) -> Bool {
    let accepted = sendAccepted(event)
    if accepted { haptic.play() }
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
