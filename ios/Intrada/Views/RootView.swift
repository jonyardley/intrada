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
      // Default (incl. plain DEBUG runs): local-first — the Library hydrates
      // from the on-device store so saved items survive restarts. Seeding is
      // opt-in via --seed-sample-data (ios-run-sim.sh / CI screenshots / E2E);
      // it loads demo data and skips StartApp so a late fetch can't clobber it.
      if seedSampleData {
        store.send(.loadSampleData)
      } else {
        store.send(.startApp(apiBaseUrl: apiBaseURL, localFirst: true))
        store.restorePersistedSort()
      }
    }
  }

  private var seedSampleData: Bool {
    ProcessInfo.processInfo.arguments.contains("--seed-sample-data")
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
  /// (indigo via `tint`) floats over it. Pushed screens (add, edit) render their
  /// nav title in the serif + ink tokens to match the paper design — a touch
  /// smaller than the UIKit default so it reads as a quiet header.
  private static func applyNavBarAppearance() {
    let appearance = UINavigationBarAppearance()
    appearance.configureWithTransparentBackground()
    let ink = UIColor(IntradaColor.ink)
    if let large = UIFont(name: "SourceSerif4Variable-Semibold", size: 28) {
      appearance.largeTitleTextAttributes = [.font: large, .foregroundColor: ink]
    }
    if let inline = UIFont(name: "SourceSerif4Variable-Semibold", size: 16) {
      appearance.titleTextAttributes = [.font: inline, .foregroundColor: ink]
    }
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
