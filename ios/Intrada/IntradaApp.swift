import Sentry
import SwiftUI

@main
struct IntradaApp: App {
  // On-disk store for the real app; on failure we report it and Store falls
  // back to in-memory (this session won't persist). Tests/previews pass nil and
  // get the in-memory default.
  @State private var store = Store(store: IntradaApp.openOnDiskStore())

  private static func openOnDiskStore() -> LibraryStore? {
    do {
      return try LibraryStore.onDisk()
    } catch {
      report(error)
      return nil
    }
  }

  init() {
    IntradaFonts.register()

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
