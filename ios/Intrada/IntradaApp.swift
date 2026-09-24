import Sentry
import SwiftUI
import UIKit

@main
struct IntradaApp: App {
  // On-disk store for the real app; on failure we report it and Store falls
  // back to in-memory (this session won't persist). Tests/previews pass nil and
  // get the in-memory default.
  @State private var store = IntradaApp.makeStore()

  private static func makeStore() -> Store {
    // Sentry starts before the database opens, or a store-open failure is dropped (#2058).
    startSentry()
    let opened = openOnDiskStore()
    // opened == nil → on-disk failed, Store falls back to in-memory: degraded.
    return Store(store: opened, degraded: opened == nil)
  }

  private static func openOnDiskStore() -> LibraryStore? {
    do {
      return try LibraryStore.onDisk()
    } catch {
      report(error, "store-open")
      return nil
    }
  }

  private static func startSentry() { _ = sentryStarted }

  private static let sentryStarted: Void = {
    // `hasPrefix` gates out empty (CI) and an unexpanded `${…}` literal alike.
    let runningUnderXCTest =
      ProcessInfo.processInfo.environment["XCTestConfigurationFilePath"] != nil
    if !runningUnderXCTest,
      let dsn = Bundle.main.object(forInfoDictionaryKey: "SENTRY_DSN") as? String,
      dsn.hasPrefix("https://")
    {
      SentrySDK.start { options in
        options.dsn = dsn
        if let release = SentryRelease.name(for: .main) {
          options.releaseName = release
        }
        #if DEBUG
          options.environment = "development"
          options.tracesSampleRate = 1.0
        #else
          options.environment = "production"
          options.tracesSampleRate = 0.2
        #endif
        options.enablePerformanceV2 = true
        options.enableAppHangTrackingV2 = true
      }
    }
  }()

  init() {
    IntradaFonts.register()

    if UITestFlags.animationsDisabled {
      UIView.setAnimationsEnabled(false)
    }

    Self.startSentry()
  }

  var body: some Scene {
    WindowGroup {
      RootView()
        .environment(store)
    }
  }
}
