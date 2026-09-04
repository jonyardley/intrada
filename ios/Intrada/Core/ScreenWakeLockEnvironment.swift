import SwiftUI

private struct ScreenWakeLockKey: EnvironmentKey {
  // Inert by default: previews and snapshot tests mount the player with no host
  // to inject one, and a live default would leave a process-global flag set for
  // whatever ran next. `RootView` injects `.system()`.
  static let defaultValue = ScreenWakeLock { _ in }
}

extension EnvironmentValues {
  var screenWakeLock: ScreenWakeLock {
    get { self[ScreenWakeLockKey.self] }
    set { self[ScreenWakeLockKey.self] = newValue }
  }
}
