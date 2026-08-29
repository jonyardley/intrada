import SharedTypes
import Testing

@testable import Intrada

struct PageSuggesterTests {
  @Test func aPageWithNothingOnItIsNeverAsked() {
    #expect(!PageSuggester.hasSomethingToRead([]))
    #expect(!PageSuggester.hasSomethingToRead(["", "   ", "\n", "\t"]))
    #expect(PageSuggester.hasSomethingToRead(["", "Autumn Leaves"]))
  }

  @Test func aSuggestionCrossesToTheCoreUntouched() {
    let bridged = PageSuggester.Suggestion(
      title: "autumn leaves", composer: "Music by Kosma", tempoMarking: "MODERATO", bpm: 120
    ).bridged

    #expect(bridged.title == "autumn leaves")
    #expect(bridged.composer == "Music by Kosma", "the credit prefix is the core's to strip")
    #expect(bridged.tempoMarking == "MODERATO")
    #expect(bridged.bpm == 120)
  }

  @Test func aBpmTheBridgeCannotCarryIsDroppedRatherThanWrapped() {
    #expect(PageSuggester.Suggestion(bpm: 65656).bridged.bpm == nil)
    #expect(PageSuggester.Suggestion(bpm: -1).bridged.bpm == nil)
  }
}
