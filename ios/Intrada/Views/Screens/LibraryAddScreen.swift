import SharedTypes
import SwiftUI

/// Create sheet for a new library item. Sends `Event.item(.add)` — the core
/// validates and (in local-first mode) persists locally with a client-minted
/// ulid; the shell only collects field values.
struct LibraryAddScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  @State private var kind: ItemKind
  @State private var title = ""
  @State private var composer = ""
  @State private var key = ""
  @State private var modality: Modality?
  @State private var marking = ""
  @State private var bpm = ""
  @State private var notes = ""
  @State private var tags: [String] = []

  init(defaultKind: ItemKind = .piece) {
    _kind = State(initialValue: defaultKind)
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
      .navigationTitle("New \(kind.label)")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Cancel") { dismiss() }
        }
        ToolbarItem(placement: .confirmationAction) {
          Button("Add", action: add)
            .disabled(title.trimmingCharacters(in: .whitespaces).isEmpty)
        }
      }
    }
  }

  private var divider: some View {
    Rectangle().fill(IntradaColor.hairline).frame(height: 1)
  }

  private var composerSuggestions: [String] {
    store.viewModel?.availableComposers ?? []
  }

  private var availableTags: [String] {
    store.viewModel?.availableTags ?? []
  }

  private func add() {
    let input = CreateItem(
      title: title.trimmingCharacters(in: .whitespacesAndNewlines),
      kind: kind,
      composer: emptyToNil(composer),
      key: emptyToNil(key),
      modality: modality,
      tempo: buildTempo(),
      notes: emptyToNil(notes),
      tags: tags)
    store.send(.item(.add(input)))
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
    LibraryAddScreen()
      .environment(Store.preview)
  }
#endif
