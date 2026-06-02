import SwiftUI

struct RoutinesScreen: View {
  var body: some View {
    ScreenScaffold(title: "Routines") {
      PlaceholderContent(
        systemImage: "music.note.list",
        message: "Build reusable routines from your library.")
    }
  }
}

#if DEBUG
  #Preview {
    RoutinesScreen()
  }
#endif
