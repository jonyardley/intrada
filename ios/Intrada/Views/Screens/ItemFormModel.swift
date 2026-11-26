import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// Field state for the add/edit item form, shared by `LibraryAddScreen` and
/// `LibraryEditScreen`. The shell only collects values; the core validates.
@Observable
final class ItemFormModel {
  var kind: ItemKind
  var key: Key?
  var formError: String?
  /// Where the core said the refused save failed, until the thing it points at
  /// changes (#1595). The banner keeps its sentence either way.
  private(set) var errorTarget: FormErrorTarget?
  /// Bumped by every refusal, so pressing Add twice on the same fault scrolls
  /// back to it: the target itself is unchanged, and nothing would fire on it.
  private(set) var faultSeq = 0
  var title = "" {
    didSet { cleared(.title) }
  }
  var composer = "" {
    didSet { cleared(.composer) }
  }
  var marking = "" {
    didSet { cleared(.tempo) }
  }
  var bpm = "" {
    didSet { cleared(.tempo) }
  }
  var chartText = "" {
    didSet { if faultsChart { errorTarget = nil } }
  }
  var notes = "" {
    didSet { cleared(.notes) }
  }
  var tags: [String] = [] {
    didSet { cleared(.tags) }
  }

  /// Removing or re-choosing a row rewrites the list the core numbered, so a
  /// mark on any row cannot survive it (#1595, and #1390 decision 12 of
  /// `specs/one-pass-create.md`).
  var stagedExercises: [StagedExercise] = [] {
    didSet {
      if case .exercise = errorTarget { errorTarget = nil }
    }
  }

  var variations: [VariationRow] = [] {
    didSet { cleared(.variations) }
  }

  init(kind: ItemKind = .piece) {
    self.kind = kind
  }

  init(item: LibraryItemView) {
    kind = item.itemType
    title = item.title
    composer = item.subtitle
    tags = item.tags
    key = item.key
    marking = item.tempoMarking ?? ""
    bpm = item.tempoBpm.map(String.init) ?? ""
    notes = item.notes ?? ""
    variations = item.variations.map {
      VariationRow(variantId: $0.id, label: $0.label, hasMarks: !$0.scoreHistory.isEmpty)
    }
  }

  func mark(_ target: FormErrorTarget?) {
    errorTarget = target
    faultSeq += 1
  }

  func clearFault() {
    errorTarget = nil
  }

  private func cleared(_ field: FormErrorField) {
    guard case .piece(let marked) = errorTarget, marked == field else { return }
    errorTarget = nil
  }

  // ── What the mark is on ──

  func faults(_ field: FormErrorField) -> Bool {
    guard case .piece(let marked) = errorTarget else { return false }
    return marked == field
  }

  var faultsChart: Bool {
    switch errorTarget {
    case .chart, .chartBar: true
    default: false
    }
  }

  var faultedBarNumber: UInt64? {
    guard case .chartBar(let bar, _) = errorTarget else { return nil }
    return bar
  }

  func faults(row index: Int) -> Bool {
    guard case .exercise(let marked, _) = errorTarget else { return false }
    return marked == UInt64(index)
  }

  func faultedField(row index: Int) -> FormErrorField? {
    guard case .exercise(let marked, let field) = errorTarget, marked == UInt64(index) else {
      return nil
    }
    return field
  }

  var canSave: Bool { itemFormCanSave(title: title) }

  /// A saved row keeps its library variation; a typed one is a label the core
  /// reuses or mints (#2246), and drops when blank (#2461).
  private var typedLabels: [String] {
    variations.filter { $0.variantId == nil }.map(\.label)
  }

  func editEvent(id: String) -> ItemEvent {
    .edit(
      id: id, input: updateInput(), variationIds: variations.compactMap(\.variantId),
      newLabels: typedLabels)
  }

  func createInput() -> CreateItem {
    CreateItem(
      title: title.trimmingCharacters(in: .whitespacesAndNewlines),
      kind: kind,
      composer: emptyToNil(composer),
      key: key,
      tempo: typedTempo(),
      notes: emptyToNil(notes),
      tags: tags,
      photoId: nil,
      variationLabels: typedLabels)
  }

  var hasStagedExtras: Bool {
    !stagedExercises.isEmpty || emptyToNil(chartText) != nil
  }

  func scaffoldEntries() -> [ScaffoldEntry] {
    stagedExercises.map(\.entry)
  }

  func updateInput() -> UpdateItem {
    UpdateItem(
      title: title,
      kind: kind,
      composer: .some(emptyToNil(composer)),
      key: key.map { .set(key: $0) } ?? .clear,
      tempo: typedTempo(),
      notes: .some(emptyToNil(notes)),
      tags: tags,
      priority: nil, includeInWarmUp: nil)
  }

  private func emptyToNil(_ value: String) -> String? {
    let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
    return trimmed.isEmpty ? nil : trimmed
  }

  private func typedTempo() -> TempoInput {
    TempoInput(marking: marking, bpm: bpm)
  }
}

/// One row of the Variations section. `variantId` is the saved variation it was
/// loaded from, sent by id only; `nil` for a row typed on the form.
struct VariationRow: Identifiable, Hashable {
  let id = UUID()
  var variantId: String?
  var label: String
  var hasMarks = false
}

extension [VariationRow] {
  mutating func move(_ id: UUID, before target: UUID) {
    guard id != target, let from = firstIndex(where: { $0.id == id }) else { return }
    let row = remove(at: from)
    insert(row, at: firstIndex(where: { $0.id == target }) ?? endIndex)
  }
}

enum StagedExercise: Identifiable, Hashable {
  case draft(id: UUID, title: String, key: Key?, bpm: String)
  case existing(id: String, title: String, meta: String?)

  var id: String {
    switch self {
    case .draft(let id, _, _, _): id.uuidString
    case .existing(let id, _, _): id
    }
  }

  var existingId: String? {
    switch self {
    case .draft: nil
    case .existing(let id, _, _): id
    }
  }

  var title: String {
    switch self {
    case .draft(_, let title, _, _): title
    case .existing(_, let title, _): title
    }
  }

  var meta: String? {
    switch self {
    case .draft(_, _, let key, let bpm):
      let tempo = UInt16(bpm.trimmingCharacters(in: .whitespaces))
        .map { declaredTempoWords(bpm: $0).text }
      let parts = [key.flatMap(KeyHelper.display), tempo].compactMap { $0 }
      return parts.isEmpty ? nil : parts.joined(separator: " · ")
    case .existing(_, _, let meta): return meta
    }
  }

  var entry: ScaffoldEntry {
    switch self {
    case .draft(_, let title, let key, let bpm):
      .new(
        CreateItem(
          title: title.trimmingCharacters(in: .whitespacesAndNewlines),
          kind: .exercise,
          composer: nil,
          key: key,
          tempo: TempoInput(marking: nil, bpm: bpm),
          notes: nil,
          tags: [],
          photoId: nil,
          variationLabels: []))
    case .existing(let id, _, _):
      .existing(id: id)
    }
  }
}
