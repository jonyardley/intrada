import Foundation
import GRDB
import IntradaCoreFFI
import SharedTypes

extension LibraryStore {
  // ── Row ↔ Item codec ─────────────────────────────────────────────────

  static func item(from row: Row, sections: [ItemSection]) -> Item {
    let marking: String? = row["tempo_marking"]
    let bpm: UInt16? = (row["tempo_bpm"] as Int?).map { UInt16($0) }
    let tempo = (marking == nil && bpm == nil) ? nil : Tempo(marking: marking, bpm: bpm)
    return Item(
      id: row["id"], title: row["title"], kind: kind(from: row["kind"]),
      composer: row["composer"], key: key(text: row["key"], modality: row["modality"]),
      tempo: tempo, notes: row["notes"],
      tags: decodeJSON([String].self, from: row["tags"], field: "tags") ?? [],
      linkedExerciseIds: decodeJSON(
        [String].self, from: row["linked_exercise_ids"], field: "linked_exercise_ids") ?? [],
      createdAt: row["created_at"], updatedAt: row["updated_at"],
      priority: row["priority"],
      chordChart: decodeChordChart(row["chord_chart"]),
      photoId: row["photo_id"], metre: decodeMetre(row["metre"]), sections: sections,
      variationIds: (row["variation_ids"] as String?).flatMap {
        decodeJSON([String].self, from: $0, field: "variation_ids")
      } ?? [],
      keys: decodeKeys(row["keys"]))
  }

  // ── Key codec (#2106) ────────────────────────────────────────────────
  // The core reads and writes a key's two columns; Swift never parses one.

  struct StoredKeyJSON: Codable {
    var key: String
    var modality: String?
  }

  /// Text the core cannot read is `nil`; the column keeps it (`upsert`).
  static func key(text: String?, modality: String?) -> Key? {
    guard let text else { return nil }
    do {
      let mode = modality.flatMap(modalities.decode).map(wheelMode)
      guard let bytes = try keyFromStored(text: text, mode: mode) else { return nil }
      return try Key.bincodeDeserialize(input: [UInt8](bytes))
    } catch {
      report(error, decodeContext)
      return nil
    }
  }

  static func stored(_ key: Key) throws -> StoredKeyJSON {
    let columns = try keyToStored(key: Data(try key.bincodeSerialize()))
    return StoredKeyJSON(
      key: columns.text, modality: columns.mode.map { modalities.encode(modality($0)) })
  }

  static func encodeKeys(_ keys: [Key]) throws -> String {
    try encodeJSON(keys.map(stored))
  }

  /// A key in the list the core cannot read is dropped from the list.
  static func decodeKeys(_ json: String?) -> [Key] {
    guard let json, let dtos = decodeJSON([StoredKeyJSON].self, from: json, field: "keys") else {
      return []
    }
    return dtos.compactMap { key(text: $0.key, modality: $0.modality) }
  }

  static func wheelMode(_ modality: Modality) -> WheelMode {
    switch modality {
    case .major: .major
    case .minor: .minor
    }
  }

  static func modality(_ mode: WheelMode) -> Modality {
    switch mode {
    case .major: .major
    case .minor: .minor
    }
  }

  // ── Metre codec ──────────────────────────────────────────────────────

  struct StoredMetre: Codable {
    var beats: UInt8
    var unit: UInt8
    var groups: [UInt8]?
  }

  static func encodeMetre(_ metre: Metre?) throws -> String? {
    guard let metre else { return nil }
    let dto = StoredMetre(beats: metre.beats, unit: metre.unit, groups: metre.groups)
    return try encodeJSON(dto)
  }

  static func decodeMetre(_ json: String?) -> Metre? {
    guard let json, let dto = decodeJSON(StoredMetre.self, from: json, field: "metre")
    else { return nil }
    return Metre(beats: dto.beats, unit: dto.unit, groups: dto.groups)
  }

  // ── Row ↔ Variation codec ────────────────────────────────────────────

  static func variation(from row: Row) -> Variation {
    Variation(
      id: row["id"], label: row["label"], updatedAt: row["updated_at"],
      deletedAt: row["deleted_at"])
  }

  // ── Row ↔ ItemSection codec ──────────────────────────────────────────────

  static func section(from row: Row) -> ItemSection {
    let first: Int? = row["bar_first"]
    let last: Int? = row["bar_last"]
    let bars = first.flatMap { f in last.map { BarRange(first: UInt16(f), last: UInt16($0)) } }
    return ItemSection(
      id: row["id"], name: row["name"], bars: bars,
      kind: sectionKinds.decode(row["kind"]) ?? .form,
      targetBpm: (row["target_bpm"] as Int?).map { UInt16($0) },
      position: UInt64(row["position"] as Int),
      updatedAt: row["updated_at"], deletedAt: row["deleted_at"])
  }

  static let sectionKinds = StoredEnum<SectionKind>(
    kind: "SectionKind", cases: [.form, .troubleSpot]
  ) {
    switch $0 {
    case .form: "form"
    case .troubleSpot: "trouble_spot"
    }
  }

  static let itemKinds = StoredEnum<ItemKind>(kind: "ItemKind", cases: [.piece, .exercise]) {
    switch $0 {
    case .piece: "piece"
    case .exercise: "exercise"
    }
  }

  static let modalities = StoredEnum<Modality>(kind: "Modality", cases: [.major, .minor]) {
    switch $0 {
    case .major: "major"
    case .minor: "minor"
    }
  }

  static func kind(from raw: String) -> ItemKind {
    itemKinds.decode(raw) ?? .piece
  }

  // ── Row ↔ ChordChart codec ───────────────────────────────────────────
  // A nested aggregate, so JSON via a Codable DTO (like StoredEntry), never
  // bincode: positional encoding would fail to decode old rows after a field
  // change, and the device is the only copy.

  /// `key` holds a spelling the core reads, as the item's column does; an
  /// empty one is a chart whose key could not be read.
  struct StoredChart: Codable {
    var key: String
    var modality: String
    // Pre-v17 rows carry the beats here; the item's `metre` column owns it now.
    var metre: UInt8?
    var sections: [StoredSection]
  }
  struct StoredSection: Codable {
    var label: String?
    var bars: [StoredBar]
  }
  struct StoredBar: Codable {
    var chords: [StoredChartChord]
  }
  // Rows written before #1948 also carry `beats`; decoding ignores it.
  struct StoredChartChord: Codable {
    var symbol: StoredChordSymbol
  }
  struct StoredChordSymbol: Codable {
    var root: UInt8
    var quality: String
    var extensions: [String]
    var bass: UInt8?
    var raw: String
  }

  static func encodeChordChart(_ chart: ChordChart?) throws -> String? {
    guard let chart else { return nil }
    let key = try chart.key.map(stored)
    let dto = StoredChart(
      key: key?.key ?? "", modality: key?.modality ?? modalities.encode(.major), metre: nil,
      sections: chart.sections.map { section in
        StoredSection(
          label: section.label,
          bars: section.bars.map { bar in
            StoredBar(
              chords: bar.chords.map { chord in
                StoredChartChord(
                  symbol: StoredChordSymbol(
                    root: chord.symbol.root, quality: chordQualities.encode(chord.symbol.quality),
                    extensions: chord.symbol.extensions, bass: chord.symbol.bass,
                    raw: chord.symbol.raw))
              })
          })
      })
    return try encodeJSON(dto)
  }

  static func decodeChordChart(_ json: String?) -> ChordChart? {
    guard let json, let dto = decodeJSON(StoredChart.self, from: json, field: "chord_chart")
    else { return nil }
    return ChordChart(
      key: key(text: dto.key, modality: dto.modality),
      sections: dto.sections.map { section in
        ChartSection(
          label: section.label,
          bars: section.bars.map { bar in
            Bar(
              chords: bar.chords.map { chord in
                ChartChord(
                  symbol: ChordSymbol(
                    root: chord.symbol.root,
                    // An unknown quality falls back to arpeggio.
                    quality: chordQualities.decode(chord.symbol.quality) ?? .other,
                    extensions: chord.symbol.extensions, bass: chord.symbol.bass,
                    raw: chord.symbol.raw))
              })
          })
      })
  }

  static let chordQualities = StoredEnum<ChordQuality>(
    kind: "ChordQuality",
    cases: [
      .maj7, .dom7, .min7, .min7b5, .dim7, .minMaj7, .six, .min6, .alt, .sus4, .sus2, .aug,
      .dom7Sharp5, .other,
    ]
  ) {
    switch $0 {
    case .maj7: "maj7"
    case .dom7: "dom7"
    case .min7: "min7"
    case .min7b5: "min7b5"
    case .dim7: "dim7"
    case .minMaj7: "minMaj7"
    case .six: "six"
    case .min6: "min6"
    case .alt: "alt"
    case .sus4: "sus4"
    case .sus2: "sus2"
    case .aug: "aug"
    case .dom7Sharp5: "dom7Sharp5"
    case .other: "other"
    }
  }
}
