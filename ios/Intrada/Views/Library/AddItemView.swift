import SwiftUI

/// Form to create a new library item (piece or exercise).
struct AddItemView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.dismiss) private var dismiss

    @State private var itemType: ItemKind = .piece
    @State private var title = ""
    @State private var composer = ""
    @State private var category = ""
    @State private var key = ""
    @State private var tempoMarking = ""
    @State private var bpm = ""
    @State private var notes = ""
    @State private var tagsText = ""

    @State private var errors: [String: String] = [:]

    var body: some View {
        Form {
            Section {
                Picker("Type", selection: $itemType) {
                    Text("Piece").tag(ItemKind.piece)
                    Text("Exercise").tag(ItemKind.exercise)
                }
                .pickerStyle(.segmented)
                .listRowBackground(Color.clear)
                .listRowInsets(EdgeInsets())
                .padding(.horizontal)
                .onChange(of: itemType) { errors.removeAll() }
            }

            Section("Details") {
                ValidatedField("Title", text: $title, error: errors["title"])
                if itemType == .piece {
                    ValidatedField("Composer", text: $composer, error: errors["composer"])
                } else {
                    ValidatedField("Category", text: $category, error: errors["category"],
                                   placeholder: "e.g. Scales, Arpeggios")
                    ValidatedField("Composer", text: $composer, error: errors["composer"],
                                   placeholder: "Optional")
                }
                ValidatedField("Key", text: $key, placeholder: "e.g. C major")
            }

            Section("Tempo") {
                ValidatedField("Tempo Marking", text: $tempoMarking,
                               placeholder: "e.g. Allegro")
                ValidatedField("BPM", text: $bpm, placeholder: "e.g. 120")
                    .keyboardType(.numberPad)
            }

            Section("Additional") {
                TextField("Notes", text: $notes, axis: .vertical)
                    .lineLimit(3...6)
                ValidatedField("Tags", text: $tagsText,
                               placeholder: "Comma-separated (e.g. Bach, Baroque)")
            }
        }
        .navigationTitle("Add \(itemType == .piece ? "Piece" : "Exercise")")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") { dismiss() }
            }
            ToolbarItem(placement: .confirmationAction) {
                Button("Save") { saveItem() }
                    .fontWeight(.semibold)
            }
        }
    }

    private func saveItem() {
        errors.removeAll()

        // Validation
        if title.trimmingCharacters(in: .whitespaces).isEmpty {
            errors["title"] = "Title is required"
        }
        if itemType == .piece && composer.trimmingCharacters(in: .whitespaces).isEmpty {
            errors["composer"] = "Composer is required for pieces"
        }

        guard errors.isEmpty else { return }

        let parsedTempo: Tempo?
        if !tempoMarking.isEmpty || !bpm.isEmpty {
            parsedTempo = Tempo(
                marking: tempoMarking.isEmpty ? nil : tempoMarking,
                bpm: bpm.isEmpty ? nil : UInt16(bpm)
            )
        } else {
            parsedTempo = nil
        }

        let tags = tagsText
            .split(separator: ",")
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }

        let input = CreateItem(
            title: title.trimmingCharacters(in: .whitespaces),
            kind: itemType,
            composer: composer.isEmpty ? nil : composer.trimmingCharacters(in: .whitespaces),
            category: category.isEmpty ? nil : category.trimmingCharacters(in: .whitespaces),
            key: key.isEmpty ? nil : key.trimmingCharacters(in: .whitespaces),
            tempo: parsedTempo,
            notes: notes.isEmpty ? nil : notes.trimmingCharacters(in: .whitespaces),
            tags: tags
        )

        core.update(.item(.add(input)))
        dismiss()
    }
}

// MARK: - Validated Field

struct ValidatedField: View {
    let label: String
    @Binding var text: String
    var error: String?
    var placeholder: String?

    init(_ label: String, text: Binding<String>, error: String? = nil, placeholder: String? = nil) {
        self.label = label
        self._text = text
        self.error = error
        self.placeholder = placeholder
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            TextField(placeholder ?? label, text: $text)
            if let error {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.red)
            }
        }
    }
}
