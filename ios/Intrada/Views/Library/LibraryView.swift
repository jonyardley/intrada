import SwiftUI

/// Root library view using NavigationSplitView.
/// - iPad (regular width): Shows sidebar list + detail pane side by side.
/// - iPhone (compact width): Auto-collapses to NavigationStack with push navigation.
struct LibraryView: View {
    @Environment(IntradaCore.self) private var core
    @State private var selectedItemId: String?
    @State private var showAddSheet: Bool = false
    @State private var showLessonSheet: Bool = false
    @State private var columnVisibility: NavigationSplitViewVisibility = .all

    var body: some View {
        NavigationSplitView(columnVisibility: $columnVisibility) {
            LibraryListContent(
                selectedItemId: $selectedItemId,
                showAddSheet: $showAddSheet
            )
            .navigationTitle("Library")
            .navigationSplitViewColumnWidth(min: 300, ideal: 320, max: 400)
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    HStack(spacing: 12) {
                        Button {
                            showLessonSheet = true
                        } label: {
                            Image(systemName: "pencil.and.list.clipboard")
                        }
                        .tint(.accent)
                        .accessibilityLabel("Log lesson")

                        Button {
                            showAddSheet = true
                        } label: {
                            Image(systemName: "plus")
                        }
                        .tint(.accent)
                        .accessibilityLabel("Add item")
                    }
                }
            }
        } detail: {
            if let itemId = selectedItemId {
                ItemDetailView(
                    itemId: itemId,
                    selectedItemId: $selectedItemId
                )
            } else {
                EmptyStateView(
                    icon: "music.note",
                    title: "Select an Item",
                    message: "Choose an item from your library"
                )
            }
        }
        .sheet(isPresented: $showAddSheet) {
            NavigationStack {
                AddItemView()
            }
        }
        .sheet(isPresented: $showLessonSheet) {
            NavigationStack {
                LessonCaptureView()
            }
        }
        .onAppear { autoSelectFirstItem() }
        .onChange(of: core.viewModel.items.first?.id) { _, _ in
            autoSelectFirstItem()
        }
    }
    /// Auto-select the first library item on iPad when nothing is selected.
    /// No-ops on iPhone (NavigationSplitView collapses to a stack).
    private func autoSelectFirstItem() {
        guard selectedItemId == nil,
              let firstId = core.viewModel.items.first?.id else { return }
        selectedItemId = firstId
    }
}

#Preview {
    LibraryView()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
