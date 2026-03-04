import SwiftUI

/// Edit a routine: rename, reorder entries, add/remove items.
struct RoutineEditView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.dismiss) private var dismiss

    let routineId: String

    @State private var name: String = ""
    @State private var entries: [RoutineEntry] = []
    @State private var showItemPicker = false
    @State private var hasChanges = false

    private var routine: RoutineView? {
        core.viewModel.routines.first(where: { $0.id == routineId })
    }

    var body: some View {
        Group {
            if let routine {
                List {
                    // Name section
                    Section("Name") {
                        TextField("Routine name", text: $name)
                            .onChange(of: name) { hasChanges = true }
                    }

                    // Entries section
                    Section {
                        if entries.isEmpty {
                            Text("No items in this routine")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                                .listRowBackground(Color.clear)
                        } else {
                            ForEach(entries, id: \.id) { entry in
                                HStack(spacing: 12) {
                                    TypeBadge(itemType: entry.itemType)
                                    Text(entry.itemTitle)
                                        .font(.subheadline)
                                    Spacer()
                                }
                            }
                            .onMove(perform: moveEntry)
                            .onDelete(perform: deleteEntry)
                        }
                    } header: {
                        HStack {
                            Text("Items (\(entries.count))")
                            Spacer()
                            Button {
                                showItemPicker = true
                            } label: {
                                Label("Add", systemImage: "plus.circle")
                                    .font(.caption)
                            }
                        }
                    }

                    // Quick load section
                    Section {
                        Button {
                            core.update(.routine(.loadRoutineIntoSetlist(routineId: routineId)))
                            dismiss()
                        } label: {
                            Label("Load into Session", systemImage: "play.circle")
                        }
                    }
                }
                .navigationTitle("Edit Routine")
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .primaryAction) {
                        Button("Save") {
                            saveRoutine()
                        }
                        .disabled(!hasChanges || name.isEmpty)
                        .fontWeight(.semibold)
                    }

                    ToolbarItem(placement: .topBarTrailing) {
                        EditButton()
                    }
                }
                .onAppear {
                    name = routine.name
                    entries = routine.entries.map { entry in
                        RoutineEntry(
                            id: entry.id,
                            itemId: entry.itemId,
                            itemTitle: entry.itemTitle,
                            itemType: entry.itemType,
                            position: entry.position
                        )
                    }
                }
                .sheet(isPresented: $showItemPicker) {
                    NavigationStack {
                        RoutineItemPicker(items: core.viewModel.items) { itemId in
                            addItem(itemId: itemId)
                        }
                    }
                }
            } else {
                ContentUnavailableView("Routine not found", systemImage: "list.bullet.rectangle")
            }
        }
    }

    private func moveEntry(from source: IndexSet, to destination: Int) {
        entries.move(fromOffsets: source, toOffset: destination)
        reindexPositions()
        hasChanges = true
    }

    private func deleteEntry(at offsets: IndexSet) {
        entries.remove(atOffsets: offsets)
        reindexPositions()
        hasChanges = true
    }

    private func addItem(itemId: String) {
        guard let item = core.viewModel.items.first(where: { $0.id == itemId }) else { return }
        let newEntry = RoutineEntry(
            id: UUID().uuidString,
            itemId: item.id,
            itemTitle: item.title,
            itemType: item.itemType,
            position: UInt(entries.count)
        )
        entries.append(newEntry)
        hasChanges = true
    }

    private func reindexPositions() {
        for i in entries.indices {
            entries[i].position = UInt(i)
        }
    }

    private func saveRoutine() {
        core.update(.routine(.updateRoutine(id: routineId, name: name, entries: entries)))
        hasChanges = false
        dismiss()
    }
}

// MARK: - Routine Item Picker

private struct RoutineItemPicker: View {
    let items: [LibraryItemView]
    let onSelect: (String) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var searchText = ""

    private var filtered: [LibraryItemView] {
        guard !searchText.isEmpty else { return items }
        return items.filter {
            $0.title.localizedCaseInsensitiveContains(searchText)
            || $0.subtitle.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        List(filtered, id: \.id) { item in
            Button {
                onSelect(item.id)
                dismiss()
            } label: {
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(item.title)
                            .font(.subheadline)
                        if !item.subtitle.isEmpty {
                            Text(item.subtitle)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    Spacer()
                    TypeBadge(itemType: item.itemType)
                }
            }
        }
        .navigationTitle("Add Item")
        .navigationBarTitleDisplayMode(.inline)
        .searchable(text: $searchText, prompt: "Search library")
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Done") { dismiss() }
            }
        }
    }
}
