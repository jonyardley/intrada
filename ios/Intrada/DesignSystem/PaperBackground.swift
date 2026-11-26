import SwiftUI

struct PaperBackground: View {
  var body: some View {
    IntradaColor.paperTop
      .ignoresSafeArea()
  }
}

#if DEBUG
  #Preview {
    PaperBackground()
  }
#endif
