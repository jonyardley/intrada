import SwiftUI

/// Setlist builder view for creating a new practice session.
struct NewSessionView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.dismiss) private var dismiss

    @State private var searchText = ""
    @State private var showItemPicker = false
    @State private var intention = ""

    private var sessionStatus: String { core.viewModel.sessionStatus }
    private var building: BuildingSetlistView? { core.viewModel.buildingSetlist }

    private var filteredItems: [LibraryItemView] {
        let items = core.viewModel.items
        guard !searchText.isEmpty else { return items }
        return items.filter {
            $0.title.localizedCaseInsensitiveContains(searchText)
            || $0.subtitle.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Recovery banner
            if sessionStatus == "active" {
                recoveryBanner
            }

            if sessionStatus == "building" || sessionStatus == "idle" {
                ScrollView {
                    VStack(spacing: 16) {
                        // Intention
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Session Intention")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                            TextField("What are you focusing on today?", text: $intention)
                                .textFieldStyle(.roundedBorder)
                                .onChange(of: intention) {
                                    core.update(.session(.setSessionIntention(intention: intention.isEmpty ? nil : intention)))
                                }
                        }
                        .padding(.horizontal)

                        // Setlist
                        if let building {
                            if building.entries.isEmpty {
                                EmptyStateView(
                                    icon: "music.note.list",
                                    title: "Empty setlist",
                                    message: "Add items from your library to build a practice session.",
                                    actionTitle: "Add Items",
                                    action: { showItemPicker = true }
                                )
                            } else {
                                VStack(alignment: .leading, spacing: 8) {
                                    HStack {
                                        Text("Setlist (\(building.entries.count) items)")
                                            .font(.subheadline)
                                            .fontWeight(.medium)
                                        Spacer()
                                        Button {
                                            showItemPicker = true
                                        } label: {
                                            Label("Add", systemImage: "plus.circle")
                                                .font(.caption)
                                        }
                                    }
                                    .padding(.horizontal)

                                    ForEach(building.entries, id: \.id) { entry in
                                        SetlistRow(entry: entry) {
                                            core.update(.session(.removeFromSetlist(entryId: entry.id)))
                                        }
                                    }
                                    .padding(.horizontal)
                                }
                            }
                        }
                    }
                    .padding(.vertical)
                }

                // Start button
                if let building, !building.entries.isEmpty {
                    Button {
                        core.update(.session(.startSession(now: Date())))
                    } label: {
                        Text("Start Session")
                            .font(.headline)
                            .foregroundStyle(.white)
                            .frame(maxWidth: .infinity)
                            .padding()
                            .background(.indigo)
                            .clipShape(RoundedRectangle(cornerRadius: 12))
                    }
                    .padding()
                }
            }
        }
        .navigationTitle("New Session")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            if sessionStatus == "building" {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        core.update(.session(.cancelBuilding))
                        dismiss()
                    }
                }
            }
        }
        .onAppear {
            if sessionStatus == "idle" {
                core.update(.session(.startBuilding))
            }
        }
        .onChange(of: sessionStatus) { _, newValue in
            if newValue == "active" {
                // Navigate to active session — handled by MainTabView navigation
            } else if newValue == "idle" {
                dismiss()
            }
        }
        .sheet(isPresented: $showItemPicker) {
            NavigationStack {
                ItemPickerView(items: core.viewModel.items) { itemId in
                    core.update(.session(.addToSetlist(itemId: itemId)))
                }
            }
        }
    }

    private var recoveryBanner: some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle")
                .font(.title2)
                .foregroundStyle(.orange)
            Text("You have a practice session in progress")
                .font(.subheadline)
                .fontWeight(.medium)
            HStack(spacing: 16) {
                NavigationLink {
                    ActiveSessionView()
                } label: {
                    Text("Resume Session")
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .foregroundStyle(.white)
                        .padding(.horizontal, 20)
                        .padding(.vertical, 10)
                        .background(.indigo)
                        .clipShape(Capsule())
                }

                Button {
                    core.update(.session(.discardSession))
                } label: {
                    Text("Discard")
                        .font(.subheadline)
                        .foregroundStyle(.red)
                }
            }
        }
        .padding()
        .frame(maxWidth: .infinity)
        .background(.orange.opacity(0.1))
    }
}

// MARK: - Setlist Row

private struct SetlistRow: View {
    let entry: SetlistEntryView
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "line.3.horizontal")
                .font(.caption)
                .foregroundStyle(.tertiary)

            VStack(alignment: .leading, spacing: 2) {
                Text(entry.itemTitle)
                    .font(.subheadline)
                    .lineLimit(1)
                TypeBadge(itemType: entry.itemType)
            }

            Spacer()

            Button(role: .destructive, action: onRemove) {
                Image(systemName: "minus.circle.fill")
                    .foregroundStyle(.red)
            }
            .buttonStyle(.plain)
        }
        .padding(.vertical, 8)
        .padding(.horizontal, 12)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }
}

// MARK: - Item Picker

struct ItemPickerView: View {
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
        .navigationTitle("Add to Setlist")
        .navigationBarTitleDisplayMode(.inline)
        .searchable(text: $searchText, prompt: "Search library")
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Done") { dismiss() }
            }
        }
    }
}
