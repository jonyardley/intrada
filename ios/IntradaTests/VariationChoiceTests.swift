import SharedTypes
import Testing

@testable import Intrada

/// The variations sheet's ticks (#2247); finding and adding are core rules
/// (#2754), so only the sheet's own typed labels feeding the add check stay here.
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

  @Test("a label already typed in this sheet is not offered twice")
  func addableOnce() {
    var choice = VariationChoice(ids: [])
    choice.toggleNew("Slow")
    #expect(choice.addable("slow", in: Self.library) == nil)
  }
}
