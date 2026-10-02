import SwiftUI

struct RoutinesScreen: View {
  @Environment(Store.self) private var store

  var body: some View {
    ScreenScaffold(title: "Routines") {
      PlaceholderContent(
        systemImage: "music.note.list",
        message: "Build reusable routines from the library.",
        actions: [
          .init(title: "Build a session", identifier: "routines.empty.build") {
            store.send(.session(.startBuilding))
          }
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
