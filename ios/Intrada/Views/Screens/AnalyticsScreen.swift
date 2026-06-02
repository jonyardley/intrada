import SwiftUI

struct AnalyticsScreen: View {
  var body: some View {
    ScreenScaffold(title: "Progress") {
      PlaceholderContent(
        systemImage: "chart.line.uptrend.xyaxis",
        message: "Track your progress and insights here.")
    }
  }
}

#if DEBUG
  #Preview {
    AnalyticsScreen()
  }
#endif
