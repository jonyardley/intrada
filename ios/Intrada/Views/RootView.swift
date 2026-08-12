import SharedTypes
import SwiftUI

struct RootView: View {
  @Environment(Store.self) private var store

  private enum AppTab {
    case library, practice, routines, progress
  }

  @State private var selectedTab: AppTab = .library
  @State private var openedAltitude = false

  private let apiBaseURL = "https://intrada-api.fly.dev"

  init() {
    Self.applyTabBarAppearance()
    Self.applyNavBarAppearance()
  }

  var body: some View {
    TabView(selection: $selectedTab) {
      LibrarySplitView().screenTransaction("Library")
        .tabItem { Label("Library", systemImage: "books.vertical") }
        .tag(AppTab.library)
      NavigationStack {
        PracticeScreen().screenTransaction("Practice")
      }
      .tabItem { Label("Practice", systemImage: "timer") }
      .tag(AppTab.practice)
      RoutinesScreen().screenTransaction("Routines")
        .tabItem { Label("Routines", systemImage: "music.note.list") }
        .tag(AppTab.routines)
      AnalyticsScreen().screenTransaction("Progress")
        .tabItem { Label("Progress", systemImage: "chart.line.uptrend.xyaxis") }
        .tag(AppTab.progress)
    }
    .tint(IntradaColor.accent)
    // The session player takes over the whole screen (no tab bar) while the core
    // is Active or Summary — "the app disappears during practice". State-driven:
    // the core drives presentation and dismissal (Save/Discard → Idle), so there's
    // no interactive dismiss to honour.
    .fullScreenCover(isPresented: playerBinding) {
      PlayerHost().environment(store)
    }
    // Journey B's altitudes take over the same way, and from RootView rather
    // than from the piece they were started on: an altitude outlives the screen
    // that opened it, and a crash recovered into one has no such screen at all.
    .fullScreenCover(isPresented: altitudeBinding) {
      PlayThroughHost().environment(store)
    }
    // `initial` because a crash recovered straight into an altitude is running
    // before the first body pass, and would otherwise never be tracked.
    .onChange(of: altitudeRunning, initial: true) { _, _ in trackAltitudeCover() }
    .onChange(of: reflectionOwed) { _, _ in trackAltitudeCover() }
    // App-level surfaces below the status bar, above all tabs. Empty when there's
    // nothing to show, so it adds no inset (keeps the plain shell unchanged).
    .safeAreaInset(edge: .top, spacing: 0) {
      VStack(spacing: 0) {
        if store.degraded {
          GlobalBanner(message: "Storage unavailable — changes this session won't be saved.")
        }
        if let error = store.viewModel?.error {
          GlobalBanner(message: error) { store.send(.clearError) }
        }
      }
    }
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
        store.loadRecoverableSession()
      }
    }
  }

  private var playerBinding: Binding<Bool> {
    Binding(
      get: { store.viewModel?.activeSession != nil || store.viewModel?.summary != nil },
      set: { _ in })  // no interactive dismiss — the core owns the session phase
  }

  private var altitudeRunning: Bool {
    store.viewModel?.coach.runThrough != nil || store.viewModel?.coach.openPlay != nil
  }

  private var reflectionOwed: Bool { store.viewModel?.built.reflection ?? false }

  /// Held open past the altitude while C2's question is owed, or the close takes
  /// the question with it (#846's class) — but only for a reflection *it* opened
  /// for: two covers at once from different ancestors is a presentation dropped.
  private var altitudeBinding: Binding<Bool> {
    Binding(
      get: { altitudeRunning || (openedAltitude && reflectionOwed) },
      set: { _ in })  // no interactive dismiss — the core owns the altitude
  }

  private func trackAltitudeCover() {
    openedAltitude = altitudeRunning || (openedAltitude && reflectionOwed)
  }

  private var seedSampleData: Bool { UITestFlags.seedSampleData }

  private static func applyTabBarAppearance() {
    let appearance = UITabBarAppearance()
    appearance.configureWithOpaqueBackground()
    appearance.backgroundColor = UIColor(IntradaColor.tabBarFill)

    // iOS 26's glass tab bar styles itself and ignores these item colours; they
    // apply on iOS 25 and earlier (active tint also comes from `.tint`).
    let normal = appearance.stackedLayoutAppearance.normal
    normal.iconColor = UIColor(IntradaColor.inkSecondary)
    normal.titleTextAttributes = [.foregroundColor: UIColor(IntradaColor.inkSecondary)]

    let selected = appearance.stackedLayoutAppearance.selected
    selected.iconColor = UIColor(IntradaColor.accent)
    selected.titleTextAttributes = [.foregroundColor: UIColor(IntradaColor.accent)]

    UITabBar.appearance().standardAppearance = appearance
    UITabBar.appearance().scrollEdgeAppearance = appearance
  }

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
