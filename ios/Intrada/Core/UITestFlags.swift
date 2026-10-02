import Foundation

enum UITestFlags {
  /// Animations off — the Practice week-strip's paging TabView otherwise defeats
  /// XCUITest's wait-for-idle (#941).
  static var animationsDisabled: Bool { has("--disable-animations") }

  static var seedSampleData: Bool { has("--seed-sample-data") }

  /// The profile outlives the app in UserDefaults, so a UI test that proves a
  /// save survives a relaunch starts by forgetting the last run's, and meets
  /// the welcome again.
  static var resetProfile: Bool { has("--reset-profile") }

  /// An unseeded relaunch on an empty store would otherwise open on the
  /// welcome, which a real install only reaches with nothing to recover.
  static var skipWelcome: Bool { has("--skip-welcome") }

  /// The welcome needs an empty library and history, and a simulator clone
  /// keeps whatever earlier tests wrote to disk (#2242).
  static var emptyStore: Bool { has("--empty-store") }

  /// Foundation Models runs in a simulator when the *host Mac* has Apple
  /// Intelligence on, putting a slow, nondeterministic call in the unit suite.
  static var onDeviceModelDisabled: Bool {
    ProcessInfo.processInfo.environment["XCTestConfigurationFilePath"] != nil
  }

  private static func has(_ flag: String) -> Bool {
    ProcessInfo.processInfo.arguments.contains(flag)
  }
}
