import SwiftUI

private struct ScreenWakeLockKey: EnvironmentKey {
  static let defaultValue: ScreenWakeLock = {
    let lock = ScreenWakeLock { _ in }
    return lock
  }()
}

extension EnvironmentValues {
  var screenWakeLock: ScreenWakeLock {
    get { self[ScreenWakeLockKey.self] }
    set { self[ScreenWakeLockKey.self] = newValue }
  }
}
