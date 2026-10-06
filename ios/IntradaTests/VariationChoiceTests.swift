import SharedTypes
import Testing

@testable import Intrada

/// The variations sheet's ticks and its find-or-add field (#2247).
struct VariationChoiceTests {
  private static var library: [VariationOptionView] {
    [
      VariationOptionView(id: "v1", label: "Hands separately", usage: nil),
      VariationOptionView(id: "v2", label: "Dotted rhythms", usage: nil),
      VariationOptionView(id: "v3", label: "Left hand alone, eyes closed", usage: nil),
    ]
  }

  @Test("ticking adds a row after the others and ticking again takes it off")
  func toggle() {
    var choice = VariationChoice(ids: ["v1"])
    choice.toggle("v3")
    #expect(choice.ids == ["v1", "v3"])
    choice.toggle("v1")
    #expect(choice.ids == ["v3"])
    #expect(!choice.isChosen("v1"))
  }

  @Test("a typed label is ticked once and untouched by the library ticks")
  func toggleNew() {
    var choice = VariationChoice(ids: [])
    choice.toggleNew("Slow")
    choice.toggle("v2")
    #expect(choice.newLabels == ["Slow"])
    choice.toggleNew("Slow")
    #expect(choice.newLabels.isEmpty)
    #expect(choice.ids == ["v2"])
  }

  @Test(
    "finding matches any part of a label, ignoring case and spaces round the text",
    arguments: [
      ("left hand", ["v3"]),
      ("  HANDS ", ["v1"]),
      ("hand", ["v1", "v3"]),
      ("", []),
      ("   ", []),
      ("trill", []),
    ] as [(String, [String])])
  func matches(query: String, ids: [String]) {
    #expect(VariationChoice.matches(query, in: Self.library).map(\.id) == ids)
  }

  @Test(
    "the typed text is offered as new only when no row holds that label",
    arguments: [
      ("Left hand", "Left hand"),
      ("  Slow practice ", "Slow practice"),
      ("dotted rhythms", nil),
      ("", nil),
    ] as [(String, String?)])
  func addable(query: String, expected: String?) {
    #expect(VariationChoice(ids: []).addable(query, in: Self.library) == expected)
  }

  @Test("a label already typed in this sheet is not offered twice")
  func addableOnce() {
    var choice = VariationChoice(ids: [])
    choice.toggleNew("Slow")
    #expect(choice.addable("slow", in: Self.library) == nil)
  }
}
