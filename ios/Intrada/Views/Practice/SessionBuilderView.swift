import SwiftUI

/// Main session builder view — tap-to-queue library items into a setlist.
///
/// Layout adapts by device:
/// - **iPhone** (compact width): Library list fills the screen with a sticky
///   bottom bar. Tap the bottom bar to open a sheet with setlist details.
/// - **iPad** (regular width): Side-by-side split — library left, setlist right.
struct SessionBuilderView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.horizontalSizeClass) private var sizeClass
    @State private var searchText = ""
    @State private var isSheetPresented = false
    @State private var expandedEntryId: String?

    /// Library items filtered by search text (shell-local concern).
    private var filteredItems: [LibraryItemView] {
        let items = core.viewModel.items
        guard !searchText.isEmpty else { return items }
        let query = searchText.lowercased()
        return items.filter { (item: LibraryItemView) -> Bool in
            item.title.lowercased().contains(query)
                || item.subtitle.lowercased().contains(query)
        }
    }

    /// The current building setlist from the ViewModel.
    private var setlist: BuildingSetlistView? {
        core.viewModel.buildingSetlist
    }

    /// Set of item IDs currently in the setlist (for selection state).
    private var selectedItemIds: Set<String> {
        guard let entries = setlist?.entries else { return [] }
        return Set(entries.map { (e: SetlistEntryView) -> String in e.itemId })
    }

    /// Total planned duration in minutes.
    private var totalMinutes: Int {
        guard let entries = setlist?.entries else { return 0 }
        let totalSecs = entries.compactMap { (e: SetlistEntryView) -> UInt32? in
            e.plannedDurationSecs
        }.reduce(0, +)
        return totalSecs > 0 ? Int(totalSecs) / 60 : Int(entries.count) * 5
    }

    var body: some View {
        if sizeClass == .regular {
            iPadLayout
        } else {
            iPhoneLayout
        }
    }

    // MARK: - iPhone Layout

    private var iPhoneLayout: some View {
        VStack(spacing: 0) {
            // Navigation bar
            HStack {
                Button {
                    core.update(.session(.cancelBuilding))
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "chevron.left")
                        Text("Practice")
                    }
                    .font(.body)
                    .foregroundStyle(Color.accentText)
                }
                Spacer()
            }
            .padding(.horizontal, 16)
            .frame(height: 44)

            // Heading
            HStack {
                Text("New Session")
                    .font(.system(size: 28, weight: .bold))
                    .foregroundStyle(Color.textPrimary)
                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.bottom, 4)

            // Search bar
            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(Color.textFaint)
                TextField("Search library...", text: $searchText)
                    .foregroundStyle(Color.textPrimary)
                    .autocorrectionDisabled()
                if !searchText.isEmpty {
                    Button {
                        searchText = ""
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(Color.textFaint)
                    }
                }
            }
            .padding(.horizontal, 12)
            .frame(height: 36)
            .background(Color.surfaceInput)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color.borderInput, lineWidth: 1)
            )
            .padding(.horizontal, 16)
            .padding(.vertical, 8)

            // Library list
            SessionBuilderListContent(
                items: filteredItems,
                selectedItemIds: selectedItemIds,
                onToggle: toggleItem
            )

            // Sticky bottom bar
            let entryCount = Int(setlist?.itemCount ?? 0)
            StickyBottomBar(
                itemCount: entryCount,
                totalMinutes: totalMinutes,
                isDisabled: entryCount == 0,
                onTapCount: { isSheetPresented = true },
                onStartSession: startSession
            )
        }
        .background(Color.backgroundApp)
        .sheet(isPresented: $isSheetPresented) {
            SetlistSheetContent(
                expandedEntryId: $expandedEntryId,
                onStartSession: startSession
            )
            .presentationDetents([.medium, .large])
            .presentationDragIndicator(.visible)
            .presentationBackground(Color.backgroundApp)
        }
    }

    // MARK: - iPad Layout

    private var iPadLayout: some View {
        HStack(spacing: 0) {
            // Left column — Library
            VStack(spacing: 0) {
                // Back link
                HStack {
                    Button {
                        core.update(.session(.cancelBuilding))
                    } label: {
                        HStack(spacing: 4) {
                            Image(systemName: "chevron.left")
                            Text("Practice")
                        }
                        .font(.body)
                        .foregroundStyle(Color.accentText)
                    }
                    Spacer()
                }
                .padding(.horizontal, 16)
                .frame(height: 44)

                // Heading
                HStack {
                    Text("Library")
                        .font(.system(size: 28, weight: .bold))
                        .foregroundStyle(Color.textPrimary)
                    Spacer()
                }
                .padding(.horizontal, 16)

                // Search bar
                HStack(spacing: 8) {
                    Image(systemName: "magnifyingglass")
                        .foregroundStyle(Color.textFaint)
                    TextField("Search library...", text: $searchText)
                        .foregroundStyle(Color.textPrimary)
                        .autocorrectionDisabled()
                    if !searchText.isEmpty {
                        Button {
                            searchText = ""
                        } label: {
                            Image(systemName: "xmark.circle.fill")
                                .foregroundStyle(Color.textFaint)
                        }
                    }
                }
                .padding(.horizontal, 12)
                .frame(height: 36)
                .background(Color.surfaceInput)
                .clipShape(RoundedRectangle(cornerRadius: 8))
                .overlay(
                    RoundedRectangle(cornerRadius: 8)
                        .stroke(Color.borderInput, lineWidth: 1)
                )
                .padding(.horizontal, 16)
                .padding(.vertical, 8)

                // Library list
                SessionBuilderListContent(
                    items: filteredItems,
                    selectedItemIds: selectedItemIds,
                    onToggle: toggleItem
                )
            }
            .frame(width: 420)
            .overlay(alignment: .trailing) {
                Rectangle()
                    .fill(Color.borderDefault)
                    .frame(width: 1)
            }

            // Right column — Setlist
            SetlistSheetContent(
                expandedEntryId: $expandedEntryId,
                onStartSession: startSession
            )
        }
        .background(Color.backgroundApp)
    }

    // MARK: - Actions

    /// Toggle a library item in/out of the setlist.
    private func toggleItem(_ item: LibraryItemView) {
        if selectedItemIds.contains(item.id) {
            // Find the entry and remove it
            if let entry = setlist?.entries.first(where: { (e: SetlistEntryView) -> Bool in
                e.itemId == item.id
            }) {
                core.update(.session(.removeFromSetlist(entryId: entry.id)))
            }
        } else {
            core.update(.session(.addToSetlist(itemId: item.id)))
        }
    }

    /// Start the session with the current timestamp.
    private func startSession() {
        let now = ISO8601DateFormatter().string(from: Date())
        core.update(.session(.startSession(now: now)))
    }
}

#Preview {
    SessionBuilderView()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
