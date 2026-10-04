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

  /// For an event the core may accept as a no-op (#1413): plays the haptic
  /// only when the outcome the tap was for actually happened.
  @discardableResult
  func send(_ event: Event, onSuccess haptic: Haptic, when landed: (ViewModel) -> Bool) -> Bool {
    let accepted = sendAccepted(event)
    let didLand = accepted && viewModel.map(landed) == true
    if didLand { haptic.play() }
    return didLand
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

extension Store {
  /// A sheet's save (#1595): the core's refusal comes back for the sheet to show
  /// inline and leaves the core, so the app banner does not repeat it; nothing
  /// celebrates until the core accepts.
  func sendFromSheet(_ event: Event) -> String? {
    let accepted = sendAccepted(event)
    guard let error = viewModel?.error ?? (accepted ? nil : "Couldn't save. Try again.") else {
      Haptic.success.play()
      return nil
    }
    send(.clearError)
    Haptic.error.play()
    UIAccessibility.post(notification: .announcement, argument: "Error: \(error)")
    return error
  }
}
