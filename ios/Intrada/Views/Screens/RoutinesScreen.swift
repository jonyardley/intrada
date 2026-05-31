import SwiftUI

struct RoutinesScreen: View {
  var body: some View {
    ScreenScaffold(title: "Routines") {
      PlaceholderContent(
        systemImage: "repeat",
        message: "Build reusable routines from your library.")
    }
  }
}

#if DEBUG
  #Preview {
    RoutinesScreen()
  }
#endif
