import IntradaCoreFFI
import SharedTypes
import SwiftUI

/// Field state for the add/edit item form, shared by `LibraryAddScreen` and
/// `LibraryEditScreen`. The shell only collects values; the core validates.
@Observable
final class ItemFormModel {
  /// The fields a photographed page can fill (#1436). Key and notes are not
  /// among them: nothing on a page reliably says either.
  /// `chart` is always empty until phase D of `specs/piece-from-photo.md`.
  enum ReadField: Hashable {
    case title, composer, marking, bpm, chart
  }

  var kind: ItemKind
  var key = ""
  var modality: Modality?
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
  private var loadedVariations = false

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
    key = item.keySelection?.spelling ?? item.key ?? ""
    modality = item.keySelection?.modality ?? item.modality
    storedMarking = item.tempoMarking ?? ""
    storedBpm = item.tempoBpm.map(String.init) ?? ""
    storedNotes = item.notes ?? ""
    variations = item.variants.map {
      VariationRow(variantId: $0.id, label: $0.label, hasMarks: !$0.scoreHistory.isEmpty)
    }
    loadedVariations = !item.variants.isEmpty
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
    if let read = draft.chartText, replaceable(.chart, storedChart) {
      take(.chart, read.value, weak: read.weak) { storedChart = $0 }
    }
  }

  private func replaceable(_ field: ReadField, _ current: String) -> Bool {
    current.isEmpty || readFrom[field] != nil
  }

  private func edited(_ field: ReadField) {
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

  /// The core decides (#1783): an exercise in several keys has no single key.
  /// Blank rows count, so Key does not flicker back while a new row is empty.
  var showsKey: Bool {
    kind != .exercise || exerciseFormShowsKey(liveVariantCount: UInt32(variations.count))
  }

  /// A row added and left blank is not a label, so it is left out rather than
  /// refused. A saved row blanked is still sent, for the core to refuse, so a
  /// row with marks never goes without the removal prompt.
  private var sentEdits: [VariantEdit] {
    variations.compactMap { row in
      let label = row.label.trimmingCharacters(in: .whitespacesAndNewlines)
      guard !label.isEmpty || row.variantId != nil else { return nil }
      return VariantEdit(id: row.variantId, label: label)
    }
  }

  /// Fields and rows are two events (#1910). With rows, the rows go last, or the
  /// key the core has just folded into a new ladder comes back as a second key;
  /// with none they go first, or a key typed after clearing them is refused
  /// while the old ones still stand.
  func editEvents(id: String) -> [ItemEvent] {
    let fields = ItemEvent.update(id: id, input: updateInput())
    let edits = sentEdits
    guard kind == .exercise, loadedVariations || !edits.isEmpty else { return [fields] }
    let rows = ItemEvent.updateVariants(id: id, variants: edits)
    return edits.isEmpty ? [rows, fields] : [fields, rows]
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
      photoId: photoId,
      variantLabels: kind == .exercise ? sentEdits.map(\.label) : [])
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
      // Hidden over saved rows, Key is not the musician's to change, and a key
      // sent then would be refused as a second key while those rows stand.
      key: loadedVariations && !showsKey ? nil : .some(emptyToNil(key)),
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

/// One row of the Variations section. `variantId` is the saved variation it was
/// loaded from, so a rename keeps its marks; `nil` for a row typed on the form.
struct VariationRow: Identifiable, Hashable {
  let id = UUID()
  var variantId: String?
  var label: String
  var hasMarks = false
}

extension [VariationRow] {
  mutating func move(_ id: UUID, by offset: Int) {
    guard let from = firstIndex(where: { $0.id == id }), indices.contains(from + offset) else {
      return
    }
    swapAt(from, from + offset)
  }

  mutating func move(_ id: UUID, before target: UUID) {
    guard id != target, let from = firstIndex(where: { $0.id == id }) else { return }
    let row = remove(at: from)
    insert(row, at: firstIndex(where: { $0.id == target }) ?? endIndex)
  }
}

enum StagedExercise: Identifiable, Hashable {
  case draft(id: UUID, title: String, key: String, modality: Modality?, bpm: String)
  case existing(id: String, title: String, meta: String?)

  var id: String {
    switch self {
    case .draft(let id, _, _, _, _): id.uuidString
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
    case .draft(_, let title, _, _, _): title
    case .existing(_, let title, _): title
    }
  }

  var meta: String? {
    switch self {
    case .draft(_, _, let key, let modality, let bpm):
      let tempo = TempoFormatting.display(
        marking: nil, bpm: UInt16(bpm.trimmingCharacters(in: .whitespaces)))
      let parts = [KeyHelper.display(key: key, modality: modality), tempo].compactMap { $0 }
      return parts.isEmpty ? nil : parts.joined(separator: " · ")
    case .existing(_, _, let meta): return meta
    }
  }

  var entry: ScaffoldEntry {
    switch self {
    // Trimmed here rather than relying on the sheet to have done it, so the
    // tempo survives whoever builds the case.
    case .draft(_, let title, let key, let modality, let bpm):
      .new(
        CreateItem(
          title: title.trimmingCharacters(in: .whitespacesAndNewlines),
          kind: .exercise,
          composer: nil,
          key: key.isEmpty ? nil : key,
          modality: modality,
          tempo: UInt16(bpm.trimmingCharacters(in: .whitespaces)).map {
            Tempo(marking: nil, bpm: $0)
          },
          notes: nil,
          tags: [],
          photoId: nil,
          variantLabels: []))
    case .existing(let id, _, _):
      .existing(id: id)
    }
  }
}
