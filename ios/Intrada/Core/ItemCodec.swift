import Foundation
import GRDB
import SharedTypes

extension LibraryStore {
  // ── Row ↔ Item codec ─────────────────────────────────────────────────

  static func item(from row: Row, variants: [Variant]) -> Item {
    let marking: String? = row["tempo_marking"]
    let bpm: UInt16? = (row["tempo_bpm"] as Int?).map { UInt16($0) }
    let tempo = (marking == nil && bpm == nil) ? nil : Tempo(marking: marking, bpm: bpm)
    return Item(
      id: row["id"], title: row["title"], kind: kind(from: row["kind"]),
      composer: row["composer"], key: row["key"],
      modality: (row["modality"] as String?).flatMap(modalities.decode),
      tempo: tempo, notes: row["notes"],
      tags: decodeJSON([String].self, from: row["tags"], field: "tags") ?? [],
      linkedExerciseIds: decodeJSON(
        [String].self, from: row["linked_exercise_ids"], field: "linked_exercise_ids") ?? [],
      createdAt: row["created_at"], updatedAt: row["updated_at"],
      priority: row["priority"],
      chordChart: decodeChordChart(row["chord_chart"]),
      variants: variants,
      photoId: row["photo_id"], metre: decodeMetre(row["metre"]))
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

  // ── Row ↔ Variant codec ──────────────────────────────────────────────

  static func variant(from row: Row) -> Variant {
    Variant(
      id: row["id"], label: row["label"], position: UInt64(row["position"] as Int),
      updatedAt: row["updated_at"], deletedAt: row["deleted_at"])
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
    let dto = StoredChart(
      key: chart.key, modality: modalities.encode(chart.modality), metre: nil,
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
      key: dto.key, modality: modalities.decode(dto.modality) ?? .major,
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
