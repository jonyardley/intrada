import SharedTypes
import SwiftUI

/// Field state for the add/edit item form, shared by `LibraryAddScreen` and
/// `LibraryEditScreen`. The shell only collects values; the core validates.
@Observable
final class ItemFormModel {
  var kind: ItemKind
  var title = ""
  var composer = ""
  var key = ""
  var modality: Modality?
  var marking = ""
  var bpm = ""
  var notes = ""
  var tags: [String] = []
  var formError: String?

  init(kind: ItemKind = .piece) {
    self.kind = kind
  }

  init(item: LibraryItemView) {
    kind = item.itemType
    title = item.title
    composer = item.subtitle
    tags = item.tags
    // Normalise on load so editing self-heals legacy combined values
    // ("F# major") into tonic + modality even if the user never re-taps a spoke.
    let selection = KeyHelper.selection(key: item.key ?? "", modality: item.modality)
    key = selection?.spelling ?? item.key ?? ""
    modality = selection?.mode ?? item.modality
    marking = item.tempoMarking ?? ""
    bpm = item.tempoBpm.map(String.init) ?? ""
    notes = item.notes ?? ""
  }

  var canSubmit: Bool {
    !title.trimmingCharacters(in: .whitespaces).isEmpty
  }

  func createInput() -> CreateItem {
    CreateItem(
      title: title.trimmingCharacters(in: .whitespacesAndNewlines),
      kind: kind,
      composer: emptyToNil(composer),
      key: emptyToNil(key),
      modality: modality,
      tempo: buildTempo(),
      notes: emptyToNil(notes),
      tags: tags)
  }

  func updateInput() -> UpdateItem {
    UpdateItem(
      title: title,
      kind: kind,
      composer: .some(emptyToNil(composer)),
      key: .some(emptyToNil(key)),
      modality: .some(modality),
      tempo: .some(buildTempo()),
      notes: .some(emptyToNil(notes)),
      tags: tags,
      priority: nil)
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
