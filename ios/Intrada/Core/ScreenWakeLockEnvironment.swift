import SwiftUI

private struct ScreenWakeLockKey: EnvironmentKey {
  // Inert by default: previews and snapshot tests mount the player without a
  // `PlayerHost`, and `isIdleTimerDisabled` is process-global, so a live
  // default would let one snapshot leave the flag set for whatever runs next.
  // Production injects `ScreenWakeLock.system()` from `PlayerHost`.
  static let defaultValue = ScreenWakeLock { _ in }
}

extension EnvironmentValues {
  var screenWakeLock: ScreenWakeLock {
    get { self[ScreenWakeLockKey.self] }
    set { self[ScreenWakeLockKey.self] = newValue }
  }
}
