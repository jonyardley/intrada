import SwiftUI

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
