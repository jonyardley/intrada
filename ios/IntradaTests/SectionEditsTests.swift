import SharedTypes
import Testing

@testable import Intrada

/// The section list the item screen sends back whole (#2247): a save, a removal
/// or a reorder must carry every other section through unchanged.
struct SectionEditsTests {
  private static func section(
    _ id: String, name: String = "", first: UInt16? = nil, last: UInt16? = nil,
    kind: SectionKind = .form, bpm: UInt16? = nil
  ) -> SectionView {
    SectionView(
      id: id, name: name, kind: kind, targetBpm: bpm, firstBar: first, lastBar: last,
      label: name, barsCaption: nil)
  }

  @Test("a section keeps its name, bars, kind and tempo when sent back")
  func editCarriesEveryField() {
    let spot = Self.section("s1", name: "Run", first: 19, last: 20, kind: .troubleSpot, bpm: 54)
    #expect(
      SectionEdits.edit(from: spot)
        == SectionEdit(
          id: "s1", name: "Run", bars: .picked(first: 19, last: 20), kind: .troubleSpot,
          targetBpm: "54"))
  }

  @Test("no bars and no tempo go back blank")
  func editWithNothingOptional() {
    #expect(
      SectionEdits.edit(from: Self.section("s1", name: "Coda"))
        == SectionEdit(id: "s1", name: "Coda", bars: .blank, kind: .form, targetBpm: ""))
  }

  @Test("a single bar with no last bar goes back as that bar alone")
  func editWithOnlyAFirstBar() {
    #expect(
      SectionEdits.edit(from: Self.section("s1", first: 12)).bars == .picked(first: 12, last: 12))
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

  @Test(
    "typed bars go to the core as typed, and an empty field is blank",
    arguments: [
      ("", nil),
      ("   ", nil),
      ("1 to 16", "1 to 16"),
      (" 16-1 ", "16-1"),
      ("bar 12", "bar 12"),
    ] as [(String, String?)])
  func typedBars(text: String, typed: String?) {
    let expected: BarsInput = typed.map { .typed($0) } ?? .blank
    #expect(SectionEdits.bars(typed: text) == expected)
  }

  @Test("saving a new section appends it after the others")
  func savingANewSection() {
    let sections = [Self.section("a", name: "A1", first: 1, last: 14)]
    let new = SectionEdit(id: nil, name: "B", bars: .typed("15 to 26"), kind: .form, targetBpm: "")
    let edits = SectionEdits.saving(new, into: sections)
    #expect(edits.map(\.id) == ["a", nil])
    #expect(edits.last == new)
  }

  @Test("saving an existing section replaces it in place")
  func savingAnExistingSection() {
    let sections = [
      Self.section("a", name: "A1"), Self.section("b", name: "B"), Self.section("c", name: "Coda"),
    ]
    let renamed = SectionEdit(id: "b", name: "B section", bars: .blank, kind: .form, targetBpm: "")
    let edits = SectionEdits.saving(renamed, into: sections)
    #expect(edits.map(\.id) == ["a", "b", "c"])
    #expect(edits[1].name == "B section")
    #expect(edits[0].name == "A1")
  }

  @Test("removing a section leaves the rest in order")
  func removingASection() {
    let sections = [Self.section("a"), Self.section("b"), Self.section("c")]
    #expect(SectionEdits.removing("b", from: sections).map(\.id) == ["a", "c"])
  }

  @Test("removing a section that is not there sends the list unchanged")
  func removingAMissingSection() {
    #expect(SectionEdits.removing("z", from: [Self.section("a")]).map(\.id) == ["a"])
  }

  @Test("the empty list saves a first section")
  func savingIntoNothing() {
    let new = SectionEdit(id: nil, name: "A1", bars: .typed("1 to 16"), kind: .form, targetBpm: "")
    #expect(SectionEdits.saving(new, into: []) == [new])
  }
}
