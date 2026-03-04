import SwiftUI
import ClerkKit

/// Main tab-based navigation matching the web shell's bottom navigation.
struct MainTabView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(Clerk.self) private var clerk
    @State private var selectedTab: Tab = .library
    @State private var showingSignOutConfirmation = false
    @State private var isSigningOut = false

    enum Tab: String, CaseIterable {
        case library = "Library"
        case practice = "Practice"
        case routines = "Routines"
        case goals = "Goals"
        case analytics = "Analytics"

        var icon: String {
            switch self {
            case .library: "books.vertical"
            case .practice: "play.circle"
            case .routines: "list.bullet.rectangle"
            case .goals: "target"
            case .analytics: "chart.xyaxis.line"
            }
        }
    }

    var body: some View {
        TabView(selection: $selectedTab) {
            ForEach(Tab.allCases, id: \.self) { tab in
                NavigationStack {
                    tabContent(for: tab)
                        .toolbar {
                            ToolbarItem(placement: .topBarTrailing) {
                                accountMenu
                            }
                        }
                }
                .tabItem {
                    Label(tab.rawValue, systemImage: tab.icon)
                }
                .tag(tab)
            }
        }
        .tint(.indigo)
        .onChange(of: core.viewModel.sessionStatus) { _, newValue in
            // Auto-navigate to Practice tab when a session becomes active
            if newValue == "active" || newValue == "summary" {
                selectedTab = .practice
            }
        }
        .confirmationDialog(
            "Sign Out",
            isPresented: $showingSignOutConfirmation,
            titleVisibility: .visible
        ) {
            Button("Sign Out", role: .destructive) {
                Task {
                    isSigningOut = true
                    defer { isSigningOut = false }
                    try? await clerk.auth.signOut()
                }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Are you sure you want to sign out?")
        }
    }

    // MARK: - Account Menu

    private var accountMenu: some View {
        Menu {
            if let user = clerk.user {
                Section {
                    let displayName = [user.firstName, user.lastName]
                        .compactMap { $0 }
                        .joined(separator: " ")
                    Label(
                        displayName.isEmpty
                            ? (user.primaryEmailAddress?.emailAddress ?? "Account")
                            : displayName,
                        systemImage: "person.fill"
                    )
                }
            }
            Button(role: .destructive) {
                showingSignOutConfirmation = true
            } label: {
                Label("Sign Out", systemImage: "rectangle.portrait.and.arrow.right")
            }
        } label: {
            Group {
                if isSigningOut {
                    ProgressView()
                        .controlSize(.small)
                } else {
                    Image(systemName: "person.circle")
                        .font(.title3)
                        .foregroundStyle(.indigo)
                }
            }
        }
    }

    @ViewBuilder
    private func tabContent(for tab: Tab) -> some View {
        switch tab {
        case .library:
            LibraryListView()
        case .practice:
            PracticeTabRoot()
        case .routines:
            RoutinesListView()
        case .goals:
            GoalsListView()
        case .analytics:
            AnalyticsDashboardView()
        }
    }
}

// MARK: - Practice Tab Root

/// Routes between session states: idle → new session, building → setlist builder,
/// active → active practice, summary → session summary.
struct PracticeTabRoot: View {
    @Environment(IntradaCore.self) private var core

    private var sessionStatus: String { core.viewModel.sessionStatus }

    var body: some View {
        Group {
            switch sessionStatus {
            case "building":
                NewSessionView()
            case "active":
                ActiveSessionView()
            case "summary":
                SessionSummaryView()
            default:
                // idle — show sessions list with "New Session" CTA
                SessionsListView()
            }
        }
    }
}
