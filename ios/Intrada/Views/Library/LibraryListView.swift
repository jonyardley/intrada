import SwiftUI

/// Main library view showing all pieces and exercises with search and type filter.
struct LibraryListView: View {
    @Environment(IntradaCore.self) private var core

    @State private var searchText = ""
    @State private var selectedType: ItemTypeFilter = .all
    @State private var showingAddItem = false

    enum ItemTypeFilter: String, CaseIterable {
        case all = "All"
        case piece = "Pieces"
        case exercise = "Exercises"
    }

    private var filteredItems: [LibraryItemView] {
        core.viewModel.items.filter { item in
            let matchesType: Bool = switch selectedType {
            case .all: true
            case .piece: item.itemType.lowercased() == "piece"
            case .exercise: item.itemType.lowercased() == "exercise"
            }

            let matchesSearch: Bool = searchText.isEmpty
                || item.title.localizedCaseInsensitiveContains(searchText)
                || item.subtitle.localizedCaseInsensitiveContains(searchText)
                || item.tags.contains(where: { $0.localizedCaseInsensitiveContains(searchText) })

            return matchesType && matchesSearch
        }
    }

    /// Active goals for the summary card at the top.
    private var activeGoals: [GoalView] {
        Array(core.viewModel.goals.filter { $0.status == "active" }.prefix(3))
    }

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                // Active Goals Summary
                if !activeGoals.isEmpty {
                    ActiveGoalsSummaryCard(goals: activeGoals)
                }

                // Type filter
                Picker("Filter", selection: $selectedType) {
                    ForEach(ItemTypeFilter.allCases, id: \.self) { filter in
                        Text(filter.rawValue).tag(filter)
                    }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal)

                // Items
                if core.isLoading {
                    ProgressView("Loading library...")
                        .padding(.top, 40)
                } else if filteredItems.isEmpty {
                    EmptyStateView(
                        icon: "music.note.list",
                        title: searchText.isEmpty ? "No items yet" : "No results",
                        message: searchText.isEmpty
                            ? "Add your first piece or exercise to get started."
                            : "Try adjusting your search or filter.",
                        actionTitle: searchText.isEmpty ? "Add Item" : nil,
                        action: searchText.isEmpty ? { showingAddItem = true } : nil
                    )
                } else {
                    LazyVStack(spacing: 12) {
                        ForEach(filteredItems, id: \.id) { item in
                            NavigationLink(value: item.id) {
                                LibraryItemCard(item: item)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .padding(.horizontal)
                }
            }
            .padding(.vertical)
        }
        .navigationTitle("Library")
        .navigationDestination(for: String.self) { itemId in
            ItemDetailView(itemId: itemId)
        }
        .searchable(text: $searchText, prompt: "Search library")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    showingAddItem = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showingAddItem) {
            NavigationStack {
                AddItemView()
            }
        }
    }
}

// MARK: - Library Item Card

struct LibraryItemCard: View {
    let item: LibraryItemView

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(item.title)
                        .font(.headline)
                        .lineLimit(1)

                    if !item.subtitle.isEmpty {
                        Text(item.subtitle)
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                }

                Spacer()

                TypeBadge(itemType: item.itemType)
            }

            if !item.tags.isEmpty {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 6) {
                        ForEach(item.tags, id: \.self) { tag in
                            Text(tag)
                                .font(.caption2)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(.quaternary)
                                .clipShape(Capsule())
                        }
                    }
                }
            }

            if let practice = item.practice, practice.sessionCount > 0 {
                HStack(spacing: 12) {
                    Label("\(practice.sessionCount) sessions", systemImage: "music.note")
                        .font(.caption)
                        .foregroundStyle(.secondary)

                    if let score = practice.latestScore {
                        ScoreIndicator(score: score)
                    }

                    if let tempo = item.latestAchievedTempo {
                        Label("\(tempo) BPM", systemImage: "metronome")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

// MARK: - Active Goals Summary Card

struct ActiveGoalsSummaryCard: View {
    let goals: [GoalView]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Active Goals")
                    .font(.subheadline)
                    .fontWeight(.semibold)
                Spacer()
                Text("\(goals.count)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            ForEach(goals, id: \.id) { goal in
                VStack(alignment: .leading, spacing: 4) {
                    Text(goal.title)
                        .font(.caption)
                        .lineLimit(1)

                    if let progress = goal.progress {
                        ProgressView(value: progress.percentage / 100.0)
                            .tint(.indigo)
                    }
                }
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .padding(.horizontal)
    }
}
