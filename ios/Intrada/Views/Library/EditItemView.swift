import SwiftUI

/// Form to edit an existing library item.
struct EditItemView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(\.dismiss) private var dismiss

    let item: LibraryItemView

    @State private var title: String
    @State private var composer: String
    @State private var category: String
    @State private var key: String
    @State private var tempoMarking: String
    @State private var bpm: String
    @State private var notes: String
    @State private var tagsText: String

    @State private var errors: [String: String] = [:]

    init(item: LibraryItemView) {
        self.item = item
        // Parse tempo string "Allegro (120 BPM)" back into parts
        let tempoStr = item.tempo ?? ""
        let parts = Self.parseTempo(tempoStr)

        _title = State(initialValue: item.title)
        _composer = State(initialValue: item.subtitle)
        _category = State(initialValue: item.category ?? "")
        _key = State(initialValue: item.key ?? "")
        _tempoMarking = State(initialValue: parts.marking)
        _bpm = State(initialValue: parts.bpm)
        _notes = State(initialValue: item.notes ?? "")
        _tagsText = State(initialValue: item.tags.joined(separator: ", "))
    }

    private static func parseTempo(_ str: String) -> (marking: String, bpm: String) {
        // The display format is "marking (N BPM)" or just "N BPM" or just "marking"
        guard !str.isEmpty else { return ("", "") }

        if let range = str.range(of: "(", options: .backwards) {
            let marking = str[str.startIndex..<range.lowerBound].trimmingCharacters(in: .whitespaces)
            let bpmPart = str[range.upperBound...]
                .replacingOccurrences(of: ")", with: "")
                .replacingOccurrences(of: " BPM", with: "")
                .trimmingCharacters(in: .whitespaces)
            return (marking, bpmPart)
        }

        if str.hasSuffix("BPM") {
            let bpmPart = str.replacingOccurrences(of: " BPM", with: "").trimmingCharacters(in: .whitespaces)
            return ("", bpmPart)
        }

        return (str, "")
    }

    private var isPiece: Bool { item.itemType.lowercased() == "piece" }

    var body: some View {
        Form {
            Section {
                HStack {
                    Text("Type")
                    Spacer()
                    TypeBadge(itemType: item.itemType)
                }
            }

            Section("Details") {
                ValidatedField("Title", text: $title, error: errors["title"])
                if isPiece {
                    ValidatedField("Composer", text: $composer, error: errors["composer"])
                } else {
                    ValidatedField("Category", text: $category, placeholder: "e.g. Scales")
                    ValidatedField("Composer", text: $composer, placeholder: "Optional")
                }
                ValidatedField("Key", text: $key, placeholder: "e.g. C major")
            }

            Section("Tempo") {
                ValidatedField("Tempo Marking", text: $tempoMarking, placeholder: "e.g. Allegro")
                ValidatedField("BPM", text: $bpm, placeholder: "e.g. 120")
                    .keyboardType(.numberPad)
            }

            Section("Additional") {
                TextField("Notes", text: $notes, axis: .vertical)
                    .lineLimit(3...6)
                ValidatedField("Tags", text: $tagsText, placeholder: "Comma-separated")
            }
        }
        .navigationTitle("Edit \(isPiece ? "Piece" : "Exercise")")
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

        if title.trimmingCharacters(in: .whitespaces).isEmpty {
            errors["title"] = "Title is required"
        }
        if isPiece && composer.trimmingCharacters(in: .whitespaces).isEmpty {
            errors["composer"] = "Composer is required"
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

        let input = UpdateItem(
            title: title.trimmingCharacters(in: .whitespaces),
            composer: composer.isEmpty ? nil : composer.trimmingCharacters(in: .whitespaces),
            category: category.isEmpty ? nil : category.trimmingCharacters(in: .whitespaces),
            key: key.isEmpty ? nil : key.trimmingCharacters(in: .whitespaces),
            tempo: parsedTempo,
            notes: notes.isEmpty ? nil : notes.trimmingCharacters(in: .whitespaces),
            tags: tags
        )

        core.update(.item(.update(id: item.id, input: input)))
        dismiss()
    }
}
