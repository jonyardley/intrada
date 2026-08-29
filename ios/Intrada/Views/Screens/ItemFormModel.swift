import SharedTypes
import SwiftUI

/// Field state for the add/edit item form, shared by `LibraryAddScreen` and
/// `LibraryEditScreen`. The shell only collects values; the core validates.
@Observable
final class ItemFormModel {
  /// The fields a photographed page can fill (#1436). Key and notes are not
  /// among them: nothing on a page reliably says either.
  enum ReadField: Hashable {
    case title, composer, marking, bpm
  }

  var kind: ItemKind
  var key = ""
  var modality: Modality?
  var notes = ""
  var tags: [String] = []
  var formError: String?
  /// The page the fields were read off, carried onto the piece the form
  /// creates so it is not photographed a second time (#1436).
  var photoId: String?

  /// Which fields still hold what the photo was read into, and whether that
  /// read was weak. Typing takes a field off: it is the user's from that
  /// keystroke on, so it must stop claiming to be the page's.
  private(set) var readFrom: [ReadField: Bool] = [:]

  // Written through `edited` so the mark clears on the keystroke, not on
  // submit; `fill(from:)` sets the storage directly.
  var title: String {
    get { storedTitle }
    set {
      storedTitle = newValue
      edited(.title)
    }
  }
  var composer: String {
    get { storedComposer }
    set {
      storedComposer = newValue
      edited(.composer)
    }
  }
  var marking: String {
    get { storedMarking }
    set {
      storedMarking = newValue
      edited(.marking)
    }
  }
  var bpm: String {
    get { storedBpm }
    set {
      storedBpm = newValue
      edited(.bpm)
    }
  }

  private var storedTitle = ""
  private var storedComposer = ""
  private var storedMarking = ""
  private var storedBpm = ""

  init(kind: ItemKind = .piece) {
    self.kind = kind
  }

  init(item: LibraryItemView) {
    kind = item.itemType
    storedTitle = item.title
    storedComposer = item.subtitle
    tags = item.tags
    // Normalise on load so editing self-heals legacy combined values
    // ("F# major") into tonic + modality even if the user never re-taps a spoke.
    let selection = KeyHelper.selection(key: item.key ?? "", modality: item.modality)
    key = selection?.spelling ?? item.key ?? ""
    modality = selection?.mode ?? item.modality
    storedMarking = item.tempoMarking ?? ""
    storedBpm = item.tempoBpm.map(String.init) ?? ""
    notes = item.notes ?? ""
  }

  /// A field is written when empty, or when it still holds an earlier read:
  /// what the user typed is theirs, but a second scan must replace what the
  /// first one wrote, and `isEmpty` alone cannot tell those apart.
  /// Nothing here is saved; pressing Add is what writes.
  func fill(from draft: PhotoDraft) {
    func take(_ field: ReadField, _ value: String, weak: Bool, into store: (String) -> Void) {
      store(value)
      readFrom[field] = weak
    }

    if let read = draft.title, replaceable(.title, storedTitle) {
      take(.title, read.value, weak: read.weak) { storedTitle = $0 }
    }
    if let read = draft.composer, replaceable(.composer, storedComposer) {
      take(.composer, read.value, weak: read.weak) { storedComposer = $0 }
    }
    if let read = draft.tempo {
      if let marking = read.value.marking, replaceable(.marking, storedMarking) {
        take(.marking, marking, weak: read.weak) { storedMarking = $0 }
      }
      if let beats = read.value.bpm, replaceable(.bpm, storedBpm) {
        take(.bpm, String(beats), weak: read.weak) { storedBpm = $0 }
      }
    }
  }

  private func replaceable(_ field: ReadField, _ current: String) -> Bool {
    current.isEmpty || readFrom[field] != nil
  }

  private func edited(_ field: ReadField) {
    readFrom[field] = nil
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
      tags: tags,
      photoId: photoId)
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
