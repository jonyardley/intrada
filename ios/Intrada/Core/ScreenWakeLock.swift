import SwiftUI
import UIKit

/// Keeps the screen awake while a session plays. `isIdleTimerDisabled` is
/// process-global and a leaked hold looks like nothing at all until the battery
/// goes (#1513), so the transitions live here rather than in the view, with the
/// flag injected so the release path is testable.
final class ScreenWakeLock: @unchecked Sendable {
  private let setFlag: @MainActor (Bool) -> Void
  private(set) var isHeld = false

  init(setFlag: @escaping @MainActor (Bool) -> Void) {
    self.setFlag = setFlag
  }

  @MainActor
  func update(sessionActive: Bool, phase: ScenePhase) {
    // Only hold while the session is active and not backgrounded.
    // Inactive (like Control Centre glances) keeps the hold.
    let shouldHold = sessionActive && phase != .background
    hold(shouldHold)
  }

  @MainActor
  func release() {
    hold(false)
  }

  @MainActor
  private func hold(_ wanted: Bool) {
    guard wanted != isHeld else { return }
    isHeld = wanted
    setFlag(wanted)
  }
}
