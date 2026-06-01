import SharedTypes
import SwiftUI

/// Edit sheet for a library item. Sends `Event.item(.update)` — the core
/// validates and reconciles; the shell only collects field values.
/// Tags and priority editing are deferred to a later increment.
struct LibraryEditScreen: View {
  let item: LibraryItemView
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  @State private var title: String
  @State private var composer: String
  @State private var key: String
  @State private var marking: String
  @State private var bpm: String
  @State private var notes: String

  init(item: LibraryItemView) {
    self.item = item
    _title = State(initialValue: item.title)
    _composer = State(initialValue: item.subtitle)
    _key = State(initialValue: item.key ?? "")
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
            VStack(spacing: 0) {
              FormField(label: "Title", text: $title, placeholder: "Required")
              divider
              FormField(label: "Composer", text: $composer)
              divider
              KeyPicker(label: "Key", text: $key)
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

  private func save() {
    let input = UpdateItem(
      title: title,
      composer: .some(emptyToNil(composer)),
      key: .some(emptyToNil(key)),
      tempo: .some(buildTempo()),
      notes: .some(emptyToNil(notes)),
      tags: nil,
      priority: nil)
    store.send(.item(.update(id: item.id, input: input)))
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
