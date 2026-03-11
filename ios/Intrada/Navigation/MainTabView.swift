import SwiftUI
import ClerkKit

/// Main tab-based navigation matching the web shell's bottom navigation.
///
/// Currently shows placeholder views for each tab. Features will be
/// implemented incrementally.
struct MainTabView: View {
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
                    PlaceholderView(tab: tab)
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
}

// MARK: - Placeholder View

/// Placeholder view shown for tabs that haven't been implemented yet.
private struct PlaceholderView: View {
    let tab: MainTabView.Tab

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: tab.icon)
                .font(.system(size: 48))
                .foregroundStyle(.tertiary)

            Text(tab.rawValue)
                .font(.title2)
                .fontWeight(.semibold)
                .foregroundStyle(.secondary)

            Text("Coming soon")
                .font(.subheadline)
                .foregroundStyle(.tertiary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(.systemBackground))
        .navigationTitle(tab.rawValue)
    }
}
