import SwiftUI

/// Scrollable library item list with type filter tabs, item count,
/// loading skeleton, and empty state. Used inside NavigationSplitView
/// sidebar on iPad and as the main list on iPhone.
struct LibraryListContent: View {
    @Environment(IntradaCore.self) private var core
    @Binding var selectedItemId: String?
    @Binding var showAddSheet: Bool
    @State private var filterTab: FilterTab = .all
    @State private var searchText: String = ""

    /// Items from the ViewModel. Filtering is done by the Crux core via `dispatchQuery()`,
    /// so this is the already-filtered result set, not a client-side filter.
    private var items: [LibraryItemView] {
        core.viewModel.items
    }

    var body: some View {
        Group {
            if core.isLoading {
                ScrollView {
                    LibrarySkeletonView()
                }
                .transition(.opacity)
            } else if items.isEmpty {
                if !searchText.isEmpty || filterTab != .all {
                    EmptyStateView(
                        icon: "magnifyingglass",
                        title: "No results",
                        message: "Try a different search or filter"
                    )
                } else {
                    EmptyStateView(
                        icon: "music.note.list",
                        title: "No items yet",
                        message: "Add your first piece or exercise to get started",
                        actionTitle: "Add Item",
                        action: { showAddSheet = true }
                    )
                }
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(items, id: \.id) { item in
                            Button {
                                selectedItemId = item.id
                            } label: {
                                LibraryItemRow(item: item)
                                    .background(
                                        item.id == selectedItemId
                                            ? Color.surfaceHover
                                            : Color.clear
                                    )
                            }
                            .buttonStyle(.plain)
                            .accessibilityLabel("\(item.title), \(item.subtitle)")

                            Divider()
                                .overlay(Color.borderDefault)
                        }
                    }
                }
                .transition(.opacity)
            }
        }
        .animation(.easeInOut(duration: 0.2), value: core.isLoading)
        .searchable(text: $searchText, prompt: "Search by title or composer")
        .onChange(of: searchText) { _, newText in
            dispatchQuery(tab: filterTab, text: newText)
        }
        .safeAreaInset(edge: .top) {
            VStack(spacing: 8) {
                // Error banner
                if let error = core.viewModel.error {
                    ErrorBanner(message: error, onDismiss: {
                        core.update(.clearError)
                    })
                    .padding(.horizontal, Spacing.card)
                }

                TypeTabs(selection: $filterTab)
                    .padding(.horizontal, Spacing.card)

                if !core.isLoading {
                    Text("\(items.count) item\(items.count == 1 ? "" : "s")")
                        .font(.caption)
                        .foregroundStyle(Color.textMuted)
                }
            }
            .padding(.vertical, 8)
            .background(.ultraThinMaterial)
        }
        .onChange(of: filterTab) { _, newTab in
            dispatchQuery(tab: newTab, text: searchText)
        }
    }

    private func dispatchQuery(tab: FilterTab, text: String) {
        let query = ListQuery(
            text: text.isEmpty ? nil : text,
            itemType: tab.itemKind,
            key: nil,
            tags: []
        )
        core.update(.setQuery(query))
    }
}

#Preview {
    struct Preview: View {
        @State private var selectedId: String?
        @State private var showAdd = false
        var body: some View {
            NavigationStack {
                LibraryListContent(
                    selectedItemId: $selectedId,
                    showAddSheet: $showAdd
                )
                .navigationTitle("Library")
            }
            .environment(IntradaCore())
        }
    }
    return Preview()
        .preferredColorScheme(.dark)
}
