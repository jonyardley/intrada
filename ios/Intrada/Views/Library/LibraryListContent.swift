import SwiftUI

/// Scrollable library item list with type filter tabs, item count,
/// loading skeleton, and empty state. Used inside NavigationSplitView
/// sidebar on iPad and as the main list on iPhone.
///
/// On iPad (regular width), uses ScrollView + LazyVStack with manual selection
/// to enable custom selection indicators without fighting SwiftUI's List selection.
/// On iPhone (compact width), uses List(selection:) for automatic push navigation.
struct LibraryListContent: View {
    @Environment(IntradaCore.self) private var core
    @Binding var selectedItemId: String?
    @Binding var showAddSheet: Bool
    @State private var filterTab: FilterTab = .all
    @State private var searchText: String = ""

    /// Detect iPad vs iPhone. We can't use horizontalSizeClass because
    /// NavigationSplitView sidebar reports .compact even on iPad.
    private var isIPad: Bool {
        UIDevice.current.userInterfaceIdiom == .pad
    }

    /// Items from the ViewModel. Filtering is done by the Crux core via `dispatchQuery()`,
    /// so this is the already-filtered result set, not a client-side filter.
    private var items: [LibraryItemView] {
        core.viewModel.items
    }

    var body: some View {
        listContent
            .background(Color.backgroundApp)
            .animation(.easeInOut(duration: 0.2), value: core.isLoading)
            .searchable(text: $searchText, prompt: "Search by title or composer")
            .onChange(of: searchText) { _, newText in
                dispatchQuery(tab: filterTab, text: newText)
            }
            .onChange(of: filterTab) { _, newTab in
                dispatchQuery(tab: newTab, text: searchText)
            }
    }

    // MARK: - List Content

    @ViewBuilder
    private var listContent: some View {
        if core.isLoading {
            ScrollView {
                LibrarySkeletonView()
            }
            .transition(.opacity)
        } else if items.isEmpty {
            List {
                filterSection

                Section {
                    if !searchText.isEmpty || filterTab != .all {
                        EmptyStateView(
                            icon: "magnifyingglass",
                            title: "No results",
                            message: "Try a different search or filter"
                        )
                        .listRowBackground(Color.clear)
                        .listRowSeparator(.hidden)
                    } else {
                        EmptyStateView(
                            icon: "music.note.list",
                            title: "No items yet",
                            message: "Add your first piece or exercise to get started",
                            actionTitle: "Add Item",
                            action: { showAddSheet = true }
                        )
                        .listRowBackground(Color.clear)
                        .listRowSeparator(.hidden)
                    }
                }
            }
            .listStyle(.plain)
            .scrollContentBackground(.hidden)
        } else if isIPad {
            // iPad: ScrollView + LazyVStack for full control over selection visuals.
            // NavigationSplitView shows sidebar + detail side by side, so we just
            // set selectedItemId to update the detail pane — no push navigation needed.
            ScrollView {
                LazyVStack(spacing: 0) {
                    filterHeader
                        .padding(.horizontal, Spacing.card)
                        .padding(.vertical, 8)

                    ForEach(items, id: \.id) { item in
                        Button {
                            selectedItemId = item.id
                        } label: {
                            LibraryItemRow(item: item)
                        }
                        .buttonStyle(.plain)
                        .background(item.id == selectedItemId ? Color.accent.opacity(0.10) : Color.clear)
                        .overlay(alignment: .leading) {
                            if item.id == selectedItemId {
                                RoundedRectangle(cornerRadius: 1.5)
                                    .fill(Color.accent)
                                    .frame(width: 3)
                                    .padding(.vertical, 4)
                            }
                        }
                        .accessibilityLabel("\(item.title), \(item.subtitle)")

                        if item.id != items.last?.id {
                            Divider()
                                .foregroundStyle(Color.borderDefault)
                                .padding(.leading, Spacing.card)
                        }
                    }
                }
            }
            .transition(.opacity)
        } else {
            // iPhone: List(selection:) for automatic NavigationSplitView push navigation.
            // No selection indicator needed — tapping pushes to detail and hides the list.
            List(selection: $selectedItemId) {
                filterSection

                Section {
                    ForEach(items, id: \.id) { item in
                        LibraryItemRow(item: item)
                            .tag(item.id)
                            .listRowBackground(Color.clear)
                            .listRowInsets(EdgeInsets())
                            .accessibilityLabel("\(item.title), \(item.subtitle)")
                    }
                }
            }
            .listStyle(.plain)
            .scrollContentBackground(.hidden)
            .transition(.opacity)
        }
    }

    // MARK: - Filter Section (List version — iPhone + empty state)

    /// Filter tabs, error banner, and item count — displayed as the first section in the list.
    private var filterSection: some View {
        Section {
            VStack(spacing: 8) {
                // Error banner
                if let error = core.viewModel.error {
                    ErrorBanner(message: error, onDismiss: {
                        core.update(.clearError)
                    })
                }

                TypeTabs(selection: $filterTab)

                if !core.isLoading {
                    Text("\(items.count) item\(items.count == 1 ? "" : "s")")
                        .font(.caption)
                        .foregroundStyle(Color.textMuted)
                }
            }
            .listRowBackground(Color.clear)
            .listRowInsets(EdgeInsets(top: 8, leading: Spacing.card, bottom: 8, trailing: Spacing.card))
            .listRowSeparator(.hidden)
        }
    }

    // MARK: - Filter Header (ScrollView version — iPad)

    /// Filter tabs, error banner, and item count — displayed as a plain VStack header.
    private var filterHeader: some View {
        VStack(spacing: 8) {
            // Error banner
            if let error = core.viewModel.error {
                ErrorBanner(message: error, onDismiss: {
                    core.update(.clearError)
                })
            }

            TypeTabs(selection: $filterTab)

            if !core.isLoading {
                Text("\(items.count) item\(items.count == 1 ? "" : "s")")
                    .font(.caption)
                    .foregroundStyle(Color.textMuted)
            }
        }
    }

    // MARK: - Helpers

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
