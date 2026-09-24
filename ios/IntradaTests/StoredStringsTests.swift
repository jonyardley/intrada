import Foundation
import IntradaCoreFFI
import SharedTypes
import Testing

@testable import Intrada

// Rows and blobs on the device are the only copy: a renamed string or key
// strands every value written under the old one.
@MainActor
struct StoredStringsTests {
  private func pin<Value: Equatable>(_ table: LibraryStore.StoredEnum<Value>, _ expected: [String])
  {
    #expect(table.cases.map(table.encode) == expected)
    for value in table.cases {
      #expect(table.decode(table.encode(value)) == value)
    }
  }

  @Test func itemKindStrings() {
    pin(LibraryStore.itemKinds, ["piece", "exercise"])
  }

  @Test func modalityStrings() {
    pin(LibraryStore.modalities, ["major", "minor"])
  }

  @Test func chordQualityStrings() {
    pin(
      LibraryStore.chordQualities,
      [
        "maj7", "dom7", "min7", "min7b5", "dim7", "minMaj7", "six", "min6", "alt", "sus4", "sus2",
        "aug", "dom7Sharp5", "other",
      ])
  }

  @Test func completionStatusStrings() {
    pin(LibraryStore.completionStatuses, ["completed", "ended_early"])
  }

  @Test func entryStatusStrings() {
    pin(LibraryStore.entryStatuses, ["completed", "skipped", "not_attempted"])
  }

  @Test func repActionStrings() {
    pin(LibraryStore.repActions, ["missed", "success"])
  }

  @Test(arguments: ["", "Piece", "ended-early", "notAttempted", " major"])
  func nearMissStringsDoNotDecode(raw: String) {
    #expect(LibraryStore.itemKinds.decode(raw) == nil)
    #expect(LibraryStore.modalities.decode(raw) == nil)
    #expect(LibraryStore.completionStatuses.decode(raw) == nil)
    #expect(LibraryStore.entryStatuses.decode(raw) == nil)
  }

  @Test func defaultsKeys() {
    #expect(Store.sortDefaultsKey == "intrada.library-sort")
    #expect(Store.profileDefaultsKey == "intrada.profile.v1")
    #expect(Store.sessionInProgressKey == "intrada.session-in-progress.v\(sessionBlobVersion())")
  }
}
