import SharedTypes
import Testing

@testable import Intrada

// #2097: a chord chart or time signature that will not decode reads back as
// none, so a save that writes that none back must not destroy the stored copy.
@MainActor
struct LibraryStoreUnreadableValueTests {
  private let damagedChart = #"{"key":"C","sections":"torn"}"#
  private let damagedMetre = #"{"beats":900,"unit":4}"#

  private func storeWithDamagedValues() throws -> LibraryStore {
    try LibraryStore.upgradeTestStore(
      migratedTo: "v17_item_metre",
      seed: """
        INSERT INTO item
          (id, title, kind, tags, linked_exercise_ids, created_at, updated_at, priority,
           chord_chart, metre)
        VALUES ('i1', 'X', 'piece', '[]', '[]',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0,
                '\(damagedChart)', '\(damagedMetre)')
        """)
  }

  @Test func savingKeepsAnUnreadableChartAndTimeSignature() throws {
    let store = try storeWithDamagedValues()
    var loaded = try #require(try store.loadItems().first)
    #expect(loaded.chordChart == nil)
    #expect(loaded.metre == nil)

    loaded.title = "Renamed"
    try store.save(loaded)

    #expect(try store.rawText("title", ofItem: "i1") == "Renamed")
    #expect(try store.rawText("chord_chart", ofItem: "i1") == damagedChart)
    #expect(try store.rawText("metre", ofItem: "i1") == damagedMetre)
  }

  @Test func settingARealValueOverwritesAnUnreadableOne() throws {
    let store = try storeWithDamagedValues()
    var loaded = try #require(try store.loadItems().first)
    let chart = ChordChart(key: Key(letter: .g, accidental: .natural, mode: .minor), sections: [])
    let metre = Metre(beats: 3, unit: 4, groups: nil)
    loaded.chordChart = chart
    loaded.metre = metre

    try store.save(loaded)

    // Decoded, not compared as text: JSONEncoder promises no key order, and a
    // reordered string failed this on CI (#2202).
    #expect(LibraryStore.decodeChordChart(try store.rawText("chord_chart", ofItem: "i1")) == chart)
    #expect(LibraryStore.decodeMetre(try store.rawText("metre", ofItem: "i1")) == metre)
  }

  @Test func clearingAReadableValueClearsIt() throws {
    let store = try LibraryStore.inMemory()
    var saved = LibraryItemFixture.record(
      id: "i1",
      chordChart: ChordChart(
        key: Key(letter: .c, accidental: .natural, mode: .major), sections: []),
      metre: Metre(beats: 4, unit: 4, groups: nil))
    try store.save(saved)
    saved.chordChart = nil
    saved.metre = nil

    try store.save(saved)

    #expect(try store.rawText("chord_chart", ofItem: "i1") == nil)
    #expect(try store.rawText("metre", ofItem: "i1") == nil)
  }

  // ── Keys a later core wrote (#2106) ──

  private let keysWithAnOddOne =
    #"[{"key":"Eb","modality":"major"},{"key":"H","modality":"dorian"}]"#
  private let chartWithAnOddKey = #"{"key":"H","modality":"dorian","sections":[]}"#

  private func storeWithOddKeys() throws -> LibraryStore {
    try LibraryStore.upgradeTestStore(
      migratedTo: "v19_keys_and_variations",
      seed: """
        INSERT INTO item
          (id, title, kind, tags, linked_exercise_ids, created_at, updated_at, priority,
           chord_chart, keys)
        VALUES ('i1', 'X', 'piece', '[]', '[]',
                '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z', 0,
                '\(chartWithAnOddKey)', '\(keysWithAnOddOne)')
        """)
  }

  @Test func savingKeepsAKeyListAndAChartKeyTheCoreCannotRead() throws {
    let store = try storeWithOddKeys()
    var loaded = try #require(try store.loadItems().first)
    #expect(loaded.keys == [Key(letter: .e, accidental: .flat, mode: .major)])
    #expect(loaded.chordChart?.key == nil)

    loaded.title = "Renamed"
    try store.save(loaded)

    #expect(try store.rawText("keys", ofItem: "i1") == keysWithAnOddOne)
    let chart = try #require(try store.rawText("chord_chart", ofItem: "i1"))
    #expect(chart.contains(#""key":"H""#), "the chart keeps its key text: \(chart)")
  }

  @Test func choosingKeysReplacesAListWithAnOddOne() throws {
    let store = try storeWithOddKeys()
    var loaded = try #require(try store.loadItems().first)
    let keys = [Key(letter: .c, accidental: .natural, mode: .major)]
    loaded.keys = keys
    loaded.chordChart?.key = Key(letter: .g, accidental: .natural, mode: .minor)

    try store.save(loaded)

    #expect(LibraryStore.decodeKeys(try store.rawText("keys", ofItem: "i1")) == keys)
    #expect(
      LibraryStore.decodeChordChart(try store.rawText("chord_chart", ofItem: "i1"))?.key
        == Key(letter: .g, accidental: .natural, mode: .minor))
  }
}
