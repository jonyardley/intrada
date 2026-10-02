import SwiftUI

struct RoutinesScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.openPractice) private var openPractice

  var body: some View {
    ScreenScaffold(title: "Routines") {
      PlaceholderContent(
        systemImage: "music.note.list",
        message: "Build reusable routines from the library.",
        actions: [
          .buildSession(
            identifier: "routines.empty.build", store: store, openPractice: openPractice)
        ])
    }
  }
}

#if DEBUG
  #Preview {
    RoutinesScreen()
      .environment(Store.preview)
  }
#endif
