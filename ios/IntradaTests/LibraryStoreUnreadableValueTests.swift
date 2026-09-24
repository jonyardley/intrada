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
    let chart = ChordChart(key: "G", modality: .minor, sections: [])
    let metre = Metre(beats: 3, unit: 4, groups: nil)
    loaded.chordChart = chart
    loaded.metre = metre

    try store.save(loaded)

    #expect(
      try store.rawText("chord_chart", ofItem: "i1") == LibraryStore.encodeChordChart(chart))
    #expect(try store.rawText("metre", ofItem: "i1") == LibraryStore.encodeMetre(metre))
  }

  @Test func clearingAReadableValueClearsIt() throws {
    let store = try LibraryStore.inMemory()
    var saved = LibraryItemFixture.record(
      id: "i1", chordChart: ChordChart(key: "C", modality: .major, sections: []),
      metre: Metre(beats: 4, unit: 4, groups: nil))
    try store.save(saved)
    saved.chordChart = nil
    saved.metre = nil

    try store.save(saved)

    #expect(try store.rawText("chord_chart", ofItem: "i1") == nil)
    #expect(try store.rawText("metre", ofItem: "i1") == nil)
  }
}
