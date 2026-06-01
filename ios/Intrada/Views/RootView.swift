import SharedTypes
import SwiftUI

/// The app shell: a four-tab bar over the three pillars — Plan (Library,
/// Routines), Practice, and Track (Analytics). Each tab is a placeholder
/// screen; the real UI lands tab-by-tab in the screen rewrite (Phase C).
/// Owns the one-time `StartApp` kick so the core bridge boots behind the tabs.
struct RootView: View {
  @Environment(Store.self) private var store

  private let apiBaseURL = "https://intrada-api.fly.dev"

  init() {
    Self.applyTabBarAppearance()
    Self.applyNavBarAppearance()
  }

  var body: some View {
    TabView {
      NavigationStack {
        LibraryScreen()
      }
      .tabItem { Label("Library", systemImage: "books.vertical") }
      PracticeScreen()
        .tabItem { Label("Practice", systemImage: "music.note") }
      RoutinesScreen()
        .tabItem { Label("Routines", systemImage: "repeat") }
      AnalyticsScreen()
        .tabItem { Label("Analytics", systemImage: "chart.line.uptrend.xyaxis") }
    }
    .tint(IntradaColor.accent)
    .task {
      // Seed mode loads demo data offline via the core; it skips StartApp so a
      // late fetch can't clobber the seed. Until there's a real backend/auth,
      // DEBUG builds seed by default (pass --no-seed-sample-data to hit the API).
      // Release seeds only via the explicit arg (CI screenshots / E2E).
      if seedSampleData {
        store.send(.loadSampleData)
      } else {
        // local-first: the Library hydrates from the on-device store, no HTTP.
        store.send(.startApp(apiBaseUrl: apiBaseURL, localFirst: true))
      }
    }
  }

  private var seedSampleData: Bool {
    let args = ProcessInfo.processInfo.arguments
    if args.contains("--no-seed-sample-data") { return false }
    #if DEBUG
      return true
    #else
      return args.contains("--seed-sample-data")
    #endif
  }

  /// Paint the UIKit tab bar with the paper theme: cream fill, faint inactive
  /// glyphs/labels, indigo for the selected tab. `tint` above colours selection
  /// at the SwiftUI layer; this carries the rest UIKit owns.
  private static func applyTabBarAppearance() {
    let appearance = UITabBarAppearance()
    appearance.configureWithOpaqueBackground()
    appearance.backgroundColor = UIColor(IntradaColor.tabBarFill)

    let normal = appearance.stackedLayoutAppearance.normal
    normal.iconColor = UIColor(IntradaColor.tabBarInactiveIcon)
    normal.titleTextAttributes = [.foregroundColor: UIColor(IntradaColor.inkFaint)]

    let selected = appearance.stackedLayoutAppearance.selected
    selected.iconColor = UIColor(IntradaColor.accent)
    selected.titleTextAttributes = [.foregroundColor: UIColor(IntradaColor.accent)]

    UITabBar.appearance().standardAppearance = appearance
    UITabBar.appearance().scrollEdgeAppearance = appearance
  }

  /// Transparent nav bar so the paper background shows through; the back chevron
  /// (indigo via `tint`) floats over it. Screens draw their own serif headers.
  private static func applyNavBarAppearance() {
    let appearance = UINavigationBarAppearance()
    appearance.configureWithTransparentBackground()
    UINavigationBar.appearance().standardAppearance = appearance
    UINavigationBar.appearance().scrollEdgeAppearance = appearance
    UINavigationBar.appearance().compactAppearance = appearance
  }
}

#if DEBUG
  #Preview {
    RootView()
      .environment(Store.previewSeeded)
  }
#endif
