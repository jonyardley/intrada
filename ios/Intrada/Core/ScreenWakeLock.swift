import SwiftUI
import UIKit

/// Keeps the screen awake while a session plays. `isIdleTimerDisabled` is
/// process-global and a leaked hold looks like nothing at all until the battery
/// goes (#1513), so the transitions live here rather than in the view, with the
/// flag injected so the release path is testable.
///
/// `@unchecked Sendable` is the price of being an `EnvironmentKey.defaultValue`,
/// which Swift 6 requires to be `Sendable`: every member below is `@MainActor`,
/// so `isHeld` is only ever touched on the main actor.
final class ScreenWakeLock: @unchecked Sendable {
  private let setFlag: @MainActor (Bool) -> Void
  private(set) var isHeld = false

  init(setFlag: @escaping @MainActor (Bool) -> Void) {
    self.setFlag = setFlag
  }

  /// The production lock, holding the real idle timer.
  static func system() -> ScreenWakeLock {
    ScreenWakeLock { UIApplication.shared.isIdleTimerDisabled = $0 }
  }

  /// Held while a session is on screen and the app is not backgrounded.
  /// `.inactive` (Control Centre, the app switcher) keeps the hold, so a glance
  /// away does not let the screen lock mid-practice.
  @MainActor
  func update(sessionActive: Bool, phase: ScenePhase) {
    hold(sessionActive && phase != .background)
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
