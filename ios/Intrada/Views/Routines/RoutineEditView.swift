import SwiftUI

/// Edit routine — rename, reorder, add/remove items.
struct RoutineEditView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.dismiss) private var dismiss

    let routine: RoutineView

    @State private var name: String = ""
    @State private var entries: [RoutineEntry] = []
    @State private var showLibrary: Bool = false

    var body: some View {
        List {
            // Name section
            Section {
                TextField("Routine name", text: $name)
                    .font(.system(size: 16))
                    .foregroundStyle(Color.textPrimary)
            } header: {
                Text("Name")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundStyle(Color.textMuted)
            }
            .listRowBackground(Color.surfaceSecondary)

            // Entries section
            Section {
                ForEach(entries, id: \.id) { (entry: RoutineEntry) in
                    HStack(spacing: Spacing.cardCompact) {
                        Text(entry.itemTitle)
                            .font(.system(size: 15, weight: .medium))
                            .foregroundStyle(Color.textPrimary)
                            .lineLimit(1)

                        Spacer()

                        TypeBadge(kind: entry.itemType)
                    }
                    .padding(.vertical, 2)
                }
                .onDelete { indexSet in
                    entries.remove(atOffsets: indexSet)
                }
                .onMove { from, to in
                    entries.move(fromOffsets: from, toOffset: to)
                }
            } header: {
                HStack {
                    Text("Items (\(entries.count))")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(Color.textMuted)
                    Spacer()
                    Button {
                        showLibrary = true
                    } label: {
                        HStack(spacing: 4) {
                            Image(systemName: "plus")
                                .font(.system(size: 10, weight: .semibold))
                            Text("Add")
                                .font(.system(size: 12, weight: .semibold))
                        }
                        .foregroundStyle(Color.accentText)
                    }
                }
            }
            .listRowBackground(Color.surfaceSecondary)
        }
        .listStyle(.insetGrouped)
        .scrollContentBackground(.hidden)
        .background(Color.backgroundApp)
        .navigationTitle("Edit Routine")
        .navigationBarTitleDisplayMode(.inline)
        .navigationBarBackButtonHidden(true)
        .toolbar {
            ToolbarItem(placement: .topBarLeading) {
                Button("Cancel") {
                    dismiss()
                }
                .foregroundStyle(Color.accentText)
            }
            ToolbarItem(placement: .topBarTrailing) {
                Button("Save") {
                    saveRoutine()
                }
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(Color.accentText)
                .disabled(name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || entries.isEmpty)
            }
        }
        .environment(\.editMode, .constant(.active))
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
        .sheet(isPresented: $showLibrary) {
            addFromLibrarySheet
        }
    }

    // MARK: - Add from Library

    private var addFromLibrarySheet: some View {
        NavigationStack {
            List {
                let existingItemIds = Set(entries.map(\.itemId))
                ForEach(core.viewModel.items, id: \.id) { (item: LibraryItemView) in
                    let alreadyAdded = existingItemIds.contains(item.id)

                    Button {
                        guard !alreadyAdded else { return }
                        let newEntry = RoutineEntry(
                            id: UUID().uuidString,
                            itemId: item.id,
                            itemTitle: item.title,
                            itemType: item.itemType,
                            position: UInt64(entries.count)
                        )
                        entries.append(newEntry)
                    } label: {
                        HStack {
                            Text(item.title)
                                .font(.system(size: 15, weight: .medium))
                                .foregroundStyle(alreadyAdded ? Color.textFaint : Color.textPrimary)
                                .lineLimit(1)

                            Spacer()

                            TypeBadge(kind: item.itemType)

                            if alreadyAdded {
                                Image(systemName: "checkmark")
                                    .font(.system(size: 12, weight: .semibold))
                                    .foregroundStyle(Color.successText)
                            }
                        }
                    }
                    .disabled(alreadyAdded)
                    .listRowBackground(Color.surfaceSecondary)
                }
            }
            .listStyle(.plain)
            .scrollContentBackground(.hidden)
            .background(Color.backgroundApp)
            .navigationTitle("Add Items")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        showLibrary = false
                    }
                    .foregroundStyle(Color.accentText)
                }
            }
        }
    }

    // MARK: - Save

    private func saveRoutine() {
        // Reindex positions
        let reindexed = entries.enumerated().map { (index: Int, entry: RoutineEntry) in
            RoutineEntry(
                id: entry.id,
                itemId: entry.itemId,
                itemTitle: entry.itemTitle,
                itemType: entry.itemType,
                position: UInt64(index)
            )
        }

        core.update(.routine(.updateRoutine(
            id: routine.id,
            name: name.trimmingCharacters(in: .whitespacesAndNewlines),
            entries: reindexed
        )))
        dismiss()
    }
}

#Preview("RoutineEditView") {
    NavigationStack {
        RoutineEditView(
            routine: RoutineView(
                id: "r1",
                name: "Morning Warm-up",
                entryCount: 2,
                entries: []
            )
        )
    }
    .environment(IntradaCore())
    .preferredColorScheme(.dark)
}
