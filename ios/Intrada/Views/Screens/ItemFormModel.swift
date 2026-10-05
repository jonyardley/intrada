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
  /// The page the fields were read off, carried onto the piece the form
  /// creates so it is not photographed a second time (#1436).
  var photoId: String?

  /// Which fields still hold what the photo was read into, and whether that
  /// read was weak. Typing takes a field off: it is the user's from that
  /// keystroke on, so it must stop claiming to be the page's.
  private(set) var readFrom: [FormReadField: Bool] = [:]

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
  var chartText: String {
    get { storedChart }
    set {
      storedChart = newValue
      edited(.chart)
    }
  }
  var notes: String {
    get { storedNotes }
    set {
      storedNotes = newValue
      cleared(.notes)
    }
  }
  var tags: [String] {
    get { storedTags }
    set {
      storedTags = newValue
      cleared(.tags)
    }
  }

  /// Removing or re-choosing a row rewrites the list the core numbered, so a
  /// mark on any row cannot survive it (#1595, and decision 12 of
  /// `specs/one-pass-create.md`).
  var stagedExercises: [StagedExercise] = [] {
    didSet {
      if case .exercise = errorTarget { errorTarget = nil }
    }
  }

  var variations: [VariationRow] = [] {
    didSet { cleared(.variations) }
  }

  private var storedTitle = ""
  private var storedComposer = ""
  private var storedMarking = ""
  private var storedBpm = ""
  private var storedChart = ""
  private var storedNotes = ""
  private var storedTags: [String] = []

  init(kind: ItemKind = .piece) {
    self.kind = kind
  }

  init(item: LibraryItemView) {
    kind = item.itemType
    storedTitle = item.title
    storedComposer = item.subtitle
    storedTags = item.tags
    key = item.key
    storedMarking = item.tempoMarking ?? ""
    storedBpm = item.tempoBpm.map(String.init) ?? ""
    storedNotes = item.notes ?? ""
    variations = item.variations.map {
      VariationRow(variantId: $0.id, label: $0.label, hasMarks: !$0.scoreHistory.isEmpty)
    }
  }

  /// The core picks the fields (#2229); a failed call is a wire break (#846).
  /// Nothing here is saved; pressing Add is what writes.
  func fill(from draft: PhotoDraft) {
    let fields = [
      FormFieldNow(field: .title, text: storedTitle, holdsRead: readFrom[.title] != nil),
      FormFieldNow(field: .composer, text: storedComposer, holdsRead: readFrom[.composer] != nil),
      FormFieldNow(field: .marking, text: storedMarking, holdsRead: readFrom[.marking] != nil),
      FormFieldNow(field: .bpm, text: storedBpm, holdsRead: readFrom[.bpm] != nil),
      FormFieldNow(field: .chart, text: storedChart, holdsRead: readFrom[.chart] != nil),
    ]
    let fills: [FormFieldFill]
    do {
      fills = try fillFormFromRead(draft: Data(draft.bincodeSerialize()), fields: fields)
    } catch {
      report(error, "bridge")
      return
    }
    for fill in fills {
      switch fill.field {
      case .title: storedTitle = fill.value
      case .composer: storedComposer = fill.value
      case .marking: storedMarking = fill.value
      case .bpm: storedBpm = fill.value
      case .chart: storedChart = fill.value
      }
      readFrom[fill.field] = fill.weak
    }
  }

  private func edited(_ field: FormReadField) {
    readFrom[field] = nil
    switch field {
    case .title: cleared(.title)
    case .composer: cleared(.composer)
    case .marking, .bpm: cleared(.tempo)
    case .chart: if faultsChart { errorTarget = nil }
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

  var canSubmit: Bool {
    !title.trimmingCharacters(in: .whitespaces).isEmpty
  }

  /// A row added and left blank is not a label, so it is left out rather than
  /// refused. A saved row keeps its library variation; a typed one is a label
  /// the core reuses or mints (#2246).
  private var typedLabels: [String] {
    variations.compactMap { row in
      guard row.variantId == nil else { return nil }
      let label = row.label.trimmingCharacters(in: .whitespacesAndNewlines)
      return label.isEmpty ? nil : label
    }
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
      photoId: photoId,
      variationLabels: kind == .exercise ? typedLabels : [])
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
      key: .some(key),
      tempo: typedTempo(),
      notes: .some(emptyToNil(notes)),
      tags: tags,
      priority: nil)
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
      let tempo = TempoFormatting.display(
        marking: nil, bpm: UInt16(bpm.trimmingCharacters(in: .whitespaces)))
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
