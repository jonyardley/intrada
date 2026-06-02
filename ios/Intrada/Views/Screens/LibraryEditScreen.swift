import SharedTypes
import SwiftUI

/// Edit sheet for a library item. Sends `Event.item(.update)` — the core
/// validates and reconciles; the shell only collects field values.
/// Priority editing is deferred to a later increment.
struct LibraryEditScreen: View {
  let item: LibraryItemView
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  @State private var kind: ItemKind
  @State private var title: String
  @State private var composer: String
  @State private var key: String
  @State private var modality: Modality?
  @State private var marking: String
  @State private var bpm: String
  @State private var notes: String
  @State private var tags: [String]

  init(item: LibraryItemView) {
    self.item = item
    _kind = State(initialValue: item.itemType)
    _tags = State(initialValue: item.tags)
    _title = State(initialValue: item.title)
    _composer = State(initialValue: item.subtitle)
    // Normalise on load so editing self-heals legacy combined values
    // ("F# major") into tonic + modality even if the user never re-taps a spoke.
    let selection = KeyHelper.selection(key: item.key ?? "", modality: item.modality)
    _key = State(initialValue: selection?.spelling ?? item.key ?? "")
    _modality = State(initialValue: selection?.mode ?? item.modality)
    _marking = State(initialValue: item.tempoMarking ?? "")
    _bpm = State(initialValue: item.tempoBpm.map(String.init) ?? "")
    _notes = State(initialValue: item.notes ?? "")
  }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        ScrollView {
          VStack(spacing: 16) {
            KindSegment(selection: $kind)

            VStack(spacing: 0) {
              FormField(label: "Title", text: $title, placeholder: "Required")
              divider
              AutocompleteField(
                label: "Composer", text: $composer, suggestions: composerSuggestions)
              divider
              KeyPicker(label: "Key", key: $key, modality: $modality)
            }
            .cardSurface()

            VStack(spacing: 0) {
              FormField(label: "Tempo marking", text: $marking, placeholder: "e.g. Allegro")
              divider
              FormField(label: "Beats per minute", text: $bpm, keyboard: .numberPad)
            }
            .cardSurface()

            FormField(label: "Notes", text: $notes, axis: .vertical)
              .cardSurface()

            VStack(spacing: 0) {
              TagChipInput(label: "Tags", tags: $tags, suggestions: availableTags)
            }
            .cardSurface()
          }
          .padding(16)
        }
      }
      .navigationTitle("Edit")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Cancel") { dismiss() }
        }
        ToolbarItem(placement: .confirmationAction) {
          Button("Save", action: save)
            .disabled(title.trimmingCharacters(in: .whitespaces).isEmpty)
        }
      }
    }
  }

  private var divider: some View {
    Rectangle().fill(IntradaColor.hairline).frame(height: 1)
  }

  private var composerSuggestions: [String] {
    ComposerSuggestions.from(store.viewModel?.items)
  }

  private var availableTags: [String] {
    store.viewModel?.availableTags ?? []
  }

  private func save() {
    let input = UpdateItem(
      title: title,
      kind: kind,
      composer: .some(emptyToNil(composer)),
      key: .some(emptyToNil(key)),
      modality: .some(modality),
      tempo: .some(buildTempo()),
      notes: .some(emptyToNil(notes)),
      tags: tags,
      priority: nil)
    store.send(.item(.update(id: item.id, input: input)))
    UINotificationFeedbackGenerator().notificationOccurred(.success)
    dismiss()
  }

  private func emptyToNil(_ value: String) -> String? {
    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    return trimmed.isEmpty ? nil : trimmed
  }

  private func buildTempo() -> Tempo? {
    let mark = emptyToNil(marking)
    let beats = UInt16(bpm.trimmingCharacters(in: .whitespaces))
    if mark == nil && beats == nil { return nil }
    return Tempo(marking: mark, bpm: beats)
  }
}

#if DEBUG
  #Preview {
    LibraryEditScreen(item: .previewDetail)
      .environment(Store.preview)
  }
#endif
