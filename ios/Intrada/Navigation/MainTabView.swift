import SwiftUI
import ClerkKit

/// Main tab-based navigation matching the web shell's bottom navigation.
///
/// Currently shows placeholder views for each tab. Features will be
/// implemented incrementally.
struct MainTabView: View {
    @Environment(Clerk.self) private var clerk
    @Environment(IntradaCore.self) private var core
    @State private var selectedTab: Tab = .library
    @State private var showingSignOutConfirmation = false
    @State private var isSigningOut = false

    enum Tab: String, CaseIterable {
        case library = "Library"
        case practice = "Practice"
        case routines = "Routines"
        case analytics = "Analytics"

        var icon: String {
            switch self {
            case .library: "books.vertical"
            case .practice: "play.circle"
            case .routines: "list.bullet.rectangle"
            case .analytics: "chart.xyaxis.line"
            }
        }
    }

    /// Whether the Practice tab should show an active indicator dot.
    private var practiceTabHasActivity: Bool {
        let status = core.viewModel.sessionStatus
        return status == .active || status == .building
    }

    var body: some View {
        TabView(selection: $selectedTab) {
            // Library — fully implemented
            LibraryView()
                .tabItem {
                    Label(Tab.library.rawValue, systemImage: Tab.library.icon)
                }
                .tag(Tab.library)

            // Practice — state-driven router
            PracticeTabRouter()
                .tabItem {
                    Label(
                        Tab.practice.rawValue,
                        systemImage: practiceTabHasActivity
                            ? "play.circle.fill"
                            : Tab.practice.icon
                    )
                }
                .tag(Tab.practice)
                .badge(practiceTabHasActivity ? "●" : nil)

            // Routines — fully implemented
            RoutineListView()
                .tabItem {
                    Label(Tab.routines.rawValue, systemImage: Tab.routines.icon)
                }
                .tag(Tab.routines)

            // Analytics — fully implemented
            AnalyticsDashboardView()
                .tabItem {
                    Label(Tab.analytics.rawValue, systemImage: Tab.analytics.icon)
                }
            .tag(Tab.analytics)
        }
        .onChange(of: core.viewModel.sessionStatus) { _, newStatus in
            // Auto-switch to Practice tab when a routine is loaded or session starts
            if newStatus == .building || newStatus == .active {
                selectedTab = .practice
            }
        }
        .tint(Color.accent)
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
                        .foregroundStyle(Color.accentText)
                }
            }
        }
    }
}
