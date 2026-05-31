import SwiftUI

@main
struct IntradaApp: App {
  @State private var store = Store()

  var body: some Scene {
    WindowGroup {
      RootView()
        .environment(store)
    }
  }
}
