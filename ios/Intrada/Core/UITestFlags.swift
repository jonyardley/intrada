import Foundation

enum UITestFlags {
  /// Animations off — the Practice week-strip's paging TabView otherwise defeats
  /// XCUITest's wait-for-idle (#941).
  static var animationsDisabled: Bool { has("--disable-animations") }

  static var seedSampleData: Bool { has("--seed-sample-data") }

  /// Foundation Models runs in a simulator when the *host Mac* has Apple
  /// Intelligence on, putting a slow, nondeterministic call in the unit suite.
  static var onDeviceModelDisabled: Bool {
    ProcessInfo.processInfo.environment["XCTestConfigurationFilePath"] != nil
  }

  private static func has(_ flag: String) -> Bool {
    ProcessInfo.processInfo.arguments.contains(flag)
  }
}
