import SharedTypes
import UIKit

struct ProfileSaveRefusal {
  let message: String
  let field: ProfileField?
}

extension Store {
  /// Sends the save and reads back the core's verdict: `nil` when it was
  /// accepted, so the caller may dismiss (#1595).
  func saveProfile(_ profile: Profile) -> ProfileSaveRefusal? {
    let accepted = sendAccepted(.profile(.save(profile)))
    guard
      let message = viewModel?.error ?? (accepted ? nil : "Couldn't save your profile. Try again.")
    else {
      Haptic.success.play()
      return nil
    }
    var field: ProfileField?
    if case .profile(let faulted) = viewModel?.errorTarget { field = faulted }
    send(.clearError)
    Haptic.error.play()
    UIAccessibility.post(notification: .announcement, argument: "Error: \(message)")
    return ProfileSaveRefusal(message: message, field: field)
  }
}
