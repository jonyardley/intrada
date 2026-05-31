import Sentry
import SwiftUI

@main
struct IntradaApp: App {
  @State private var store = Store()

  init() {
    // No DSN in dev/CI → Sentry stays disabled. Set SENTRY_DSN for real builds.
    if let dsn = Bundle.main.object(forInfoDictionaryKey: "SENTRY_DSN") as? String, !dsn.isEmpty {
      SentrySDK.start { options in
        options.dsn = dsn
      }
    }
  }

  var body: some Scene {
    WindowGroup {
      RootView()
        .environment(store)
    }
  }
}
