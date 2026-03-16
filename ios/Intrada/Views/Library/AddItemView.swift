import SwiftUI

/// Form for creating a new library item (piece or exercise).
/// Presented as a sheet from the library list.
struct AddItemView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(ToastManager.self) private var toast
    @Environment(\.dismiss) private var dismiss

    @State private var kindPicker: ItemKindPicker = .piece
    @State private var title: String = ""
    @State private var composer: String = ""
    @State private var key: String = ""
    @State private var tempoMarking: String = ""
    @State private var bpm: String = ""
    @State private var notes: String = ""
    @State private var tags: [String] = []
    @State private var errors: [String: String] = [:]
    @State private var isSubmitting: Bool = false
    /// Snapshot of item count before submission, used to detect when the core adds the item.
    @State private var itemCountBeforeSubmit: Int?

    private var itemKindValue: ItemKind {
        kindPicker.itemKind
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                // Type selection (Piece / Exercise only — no "All")
                TypeTabs(selection: Binding(
                    get: { kindPicker.filterTab },
                    set: { newTab in
                        if newTab == .pieces { kindPicker = .piece }
                        else if newTab == .exercises { kindPicker = .exercise }
                    }
                ), showAll: false)
                .onChange(of: kindPicker) { _, _ in
                    // Clear composer error when switching types
                    errors.removeValue(forKey: "composer")
                }

                // Title (required)
                TextFieldView(
                    label: "Title",
                    text: $title,
                    placeholder: "e.g. Clair de Lune",
                    error: errors["title"]
                )

                // Composer (required for pieces)
                AutocompleteField(
                    label: itemKindValue == .piece ? "Composer *" : "Composer",
                    text: $composer,
                    placeholder: "e.g. Claude Debussy",
                    error: errors["composer"],
                    suggestions: uniqueComposers(from: core.viewModel.items)
                )

                // Key
                TextFieldView(
                    label: "Key",
                    text: $key,
                    placeholder: "e.g. C Major, Db Minor",
                    error: errors["key"]
                )

                // Tempo: marking + BPM side by side
                HStack(alignment: .top, spacing: 12) {
                    TextFieldView(
                        label: "Tempo Marking",
                        text: $tempoMarking,
                        placeholder: "e.g. Allegro",
                        error: errors["tempoMarking"]
                    )

                    TextFieldView(
                        label: "BPM",
                        text: $bpm,
                        placeholder: "1-400",
                        error: errors["bpm"]
                    )
                    .keyboardType(.numberPad)
                }

                // Notes
                TextAreaView(
                    label: "Notes",
                    text: $notes,
                    placeholder: "Practice notes, goals, or reminders",
                    error: errors["notes"]
                )

                // Tags
                TagInputView(
                    tags: $tags,
                    availableTags: uniqueTags(from: core.viewModel.items),
                    error: errors["tags"]
                )

                // Save button
                ButtonView("Save", variant: .primary, disabled: isSubmitting, loading: isSubmitting) {
                    submitForm()
                }
            }
            .padding(Spacing.card)
        }
        .navigationTitle("Add Item")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") {
                    dismiss()
                }
            }
        }
        // Watch for core completing the add (item count changes) or error
        .onChange(of: core.viewModel.items.count) { _, newCount in
            guard isSubmitting, let before = itemCountBeforeSubmit, newCount > before else { return }
            toast.show("Item added", variant: .success)
            isSubmitting = false
            dismiss()
        }
        .onChange(of: core.viewModel.error) { _, newError in
            guard isSubmitting, newError != nil else { return }
            isSubmitting = false
            // Error is shown by ErrorBanner in the list — just reset the form state
        }
    }

    private func submitForm() {
        let validationErrors = LibraryFormValidator.validate(
            kind: itemKindValue,
            title: title,
            composer: composer,
            key: key,
            tempoMarking: tempoMarking,
            bpm: bpm,
            notes: notes,
            tags: tags
        )

        errors = validationErrors
        guard errors.isEmpty else { return }

        isSubmitting = true
        itemCountBeforeSubmit = core.viewModel.items.count

        // Build Tempo if either field is set
        let trimmedMarking = tempoMarking.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedBpm = bpm.trimmingCharacters(in: .whitespacesAndNewlines)
        var tempo: Tempo? = nil
        if !trimmedMarking.isEmpty || !trimmedBpm.isEmpty {
            tempo = Tempo(
                marking: trimmedMarking.isEmpty ? nil : trimmedMarking,
                bpm: trimmedBpm.isEmpty ? nil : UInt16(trimmedBpm)
            )
        }

        let trimmedTitle = title.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedComposer = composer.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedKey = key.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedNotes = notes.trimmingCharacters(in: .whitespacesAndNewlines)

        let createItem = CreateItem(
            title: trimmedTitle,
            kind: itemKindValue,
            composer: trimmedComposer.isEmpty ? nil : trimmedComposer,
            key: trimmedKey.isEmpty ? nil : trimmedKey,
            tempo: tempo,
            notes: trimmedNotes.isEmpty ? nil : trimmedNotes,
            tags: tags
        )

        core.update(.item(.add(createItem)))
    }
}

#Preview {
    NavigationStack {
        AddItemView()
    }
    .environment(IntradaCore())
    .environment(ToastManager())
    .preferredColorScheme(.dark)
}
