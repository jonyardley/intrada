import SwiftUI

/// The cream paper wash that sits behind every screen. Extends under the safe
/// areas so the gradient meets the status bar and tab bar edges.
struct PaperBackground: View {
  var body: some View {
    LinearGradient.paper
      .ignoresSafeArea()
  }
}

#if DEBUG
  #Preview {
    PaperBackground()
  }
#endif
