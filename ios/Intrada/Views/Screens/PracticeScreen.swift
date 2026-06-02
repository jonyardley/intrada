import SwiftUI

struct PracticeScreen: View {
  var body: some View {
    ScreenScaffold(title: "Practice") {
      PlaceholderContent(
        systemImage: "metronome.fill",
        message: "Start a focused practice session here.")
    }
  }
}

#if DEBUG
  #Preview {
    PracticeScreen()
  }
#endif
