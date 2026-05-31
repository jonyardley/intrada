import SwiftUI

/// Track pillar — practice analytics and insights. Placeholder for now.
struct AnalyticsScreen: View {
  var body: some View {
    ScreenScaffold(title: "Analytics") {
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
