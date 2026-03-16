import SwiftUI

/// Form for editing an existing library item.
/// Pre-populates all fields from the item's current values.
/// Type (piece/exercise) is read-only after creation.
struct EditItemView: View {
    let itemId: String
    @Environment(IntradaCore.self) private var core
    @Environment(ToastManager.self) private var toast
    @Environment(\.dismiss) private var dismiss

    @State private var title: String = ""
    @State private var composer: String = ""
    @State private var key: String = ""
    @State private var tempoMarking: String = ""
    @State private var bpm: String = ""
    @State private var notes: String = ""
    @State private var tags: [String] = []
    @State private var errors: [String: String] = [:]
    @State private var isSubmitting: Bool = false
    @State private var hasLoaded: Bool = false

    private var item: LibraryItemView? {
        core.viewModel.items.first(where: { $0.id == itemId })
    }

    private var kind: ItemKind {
        guard let item else { return .piece }
        return itemKind(from: item.itemType)
    }

    var body: some View {
        Group {
            if let item, hasLoaded {
                formContent(item: item)
            } else if let item, !hasLoaded {
                Color.clear.onAppear { populateFields(from: item) }
            } else if core.isLoading {
                DetailSkeletonView()
            } else {
                ContentUnavailableView(
                    "Item Not Found",
                    systemImage: "questionmark.circle",
                    description: Text("This item may have been deleted")
                )
            }
        }
        .navigationTitle("Edit Item")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") {
                    dismiss()
                }
            }
        }
    }

    @ViewBuilder
    private func formContent(item: LibraryItemView) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                // Type (read-only)
                TypeTabsDisplayOnly(kind: kind)

                // Title (required)
                TextFieldView(
                    label: "Title",
                    text: $title,
                    placeholder: "e.g. Clair de Lune",
                    error: errors["title"]
                )

                // Composer
                AutocompleteField(
                    label: kind == .piece ? "Composer *" : "Composer",
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

                // Tempo
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
                ButtonView("Save Changes", variant: .primary, disabled: isSubmitting, loading: isSubmitting) {
                    submitForm()
                }
            }
            .padding(Spacing.card)
        }
    }

    private func populateFields(from item: LibraryItemView) {
        title = item.title
        composer = item.subtitle
        key = item.key ?? ""
        let parsed = parseTempoDisplay(item.tempo)
        tempoMarking = parsed.marking
        bpm = parsed.bpm
        notes = item.notes ?? ""
        tags = item.tags
        hasLoaded = true
    }

    private func submitForm() {
        let validationErrors = LibraryFormValidator.validate(
            kind: kind,
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

        // Build Tempo
        let trimmedMarking = tempoMarking.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedBpm = bpm.trimmingCharacters(in: .whitespacesAndNewlines)
        var tempo: Tempo?? = .some(nil) // Clear tempo by default
        if !trimmedMarking.isEmpty || !trimmedBpm.isEmpty {
            tempo = .some(Tempo(
                marking: trimmedMarking.isEmpty ? nil : trimmedMarking,
                bpm: trimmedBpm.isEmpty ? nil : UInt16(trimmedBpm)
            ))
        }

        let trimmedTitle = title.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedComposer = composer.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedKey = key.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedNotes = notes.trimmingCharacters(in: .whitespacesAndNewlines)

        let updateItem = UpdateItem(
            title: trimmedTitle,
            composer: trimmedComposer.isEmpty ? .some(nil) : .some(trimmedComposer),
            key: trimmedKey.isEmpty ? .some(nil) : .some(trimmedKey),
            tempo: tempo,
            notes: trimmedNotes.isEmpty ? .some(nil) : .some(trimmedNotes),
            tags: tags
        )

        core.update(.item(.update(id: itemId, input: updateItem)))
        toast.show("Item updated", variant: .success)
        isSubmitting = false
        dismiss()
    }
}

#Preview {
    NavigationStack {
        EditItemView(itemId: "preview-id")
    }
    .environment(IntradaCore())
    .environment(ToastManager())
    .preferredColorScheme(.dark)
}
