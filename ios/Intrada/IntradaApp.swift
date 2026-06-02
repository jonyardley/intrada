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
      report(error, "store-open")
      return nil
    }
  }

  init() {
    IntradaFonts.register()

    // `hasPrefix` gates out empty (CI) and an unexpanded `${…}` literal alike.
    if let dsn = Bundle.main.object(forInfoDictionaryKey: "SENTRY_DSN") as? String,
      dsn.hasPrefix("https://")
    {
      SentrySDK.start { options in
        options.dsn = dsn
        #if DEBUG
          options.environment = "development"
        #else
          options.environment = "production"
        #endif
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
