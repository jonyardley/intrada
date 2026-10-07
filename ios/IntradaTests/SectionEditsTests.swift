import SharedTypes
import Testing

@testable import Intrada

/// The section sheet's Bars field opens with the stored bars as text (#2247).
struct SectionEditsTests {
  private static func section(
    _ id: String, first: UInt16? = nil, last: UInt16? = nil
  ) -> SectionView {
    SectionView(
      id: id, name: "", kind: .form, targetBpm: nil, firstBar: first, lastBar: last,
      label: "", barsCaption: nil, barsFieldText: "", latestScore: nil, scoreHistory: [],
      caption: "Not yet played", isWeakest: false)
  }

  @Test(
    "the Bars field opens with what the musician would type",
    arguments: [
      (UInt16?.none, UInt16?.none, ""),
      (12, 12, "12"),
      (12, nil, "12"),
      (19, 20, "19 to 20"),
    ] as [(UInt16?, UInt16?, String)])
  func barsText(first: UInt16?, last: UInt16?, expected: String) {
    #expect(SectionEdits.barsText(of: Self.section("s", first: first, last: last)) == expected)
  }
}
