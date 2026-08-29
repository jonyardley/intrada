import SharedTypes
import Testing

@testable import Intrada

/// The model itself needs Apple Intelligence hardware, which no simulator has,
/// so what is checked here is the half that must hold everywhere: the
/// enhancement never becomes a requirement (spec decision 6), and what does
/// cross to the core crosses untouched (decision 4).
struct PageSuggesterTests {
  /// A blank page must never reach the model. Asserted against the rule rather
  /// than against `suggest`, because on a simulator the availability check
  /// short-circuits first and would answer `nil` whether or not this held.
  @Test func aPageWithNothingOnItIsNeverAsked() {
    #expect(!PageSuggester.hasSomethingToRead([]))
    #expect(!PageSuggester.hasSomethingToRead(["", "   ", "\n", "\t"]))
    #expect(PageSuggester.hasSomethingToRead(["", "Autumn Leaves"]))
  }

  /// Decision 6, the shell half: asking must return rather than hang or trap,
  /// on every device. A simulator has no Apple Intelligence, so this exercises
  /// the unavailable path; on hardware with a model it is the real one.
  @Test func askingAPageThatCannotBeAnsweredStillReturns() async {
    let suggestion = await PageSuggester.suggest(readingFrom: ["Autumn Leaves"])
    #expect(suggestion?.bridged.chartText == nil)
  }

  @Test func aSuggestionCrossesToTheCoreUntouched() {
    let bridged = PageSuggester.Suggestion(
      title: "autumn leaves", composer: "Music by Kosma", tempoMarking: "MODERATO", bpm: 120
    ).bridged

    #expect(bridged.title == "autumn leaves")
    #expect(bridged.composer == "Music by Kosma", "tidying it up is the core's call, not ours")
    #expect(bridged.tempoMarking == "MODERATO")
    #expect(bridged.bpm == 120)
  }

  /// The core's bpm is a `u16` and the model answers with an `Int`, so the
  /// conversion has to say no rather than wrap round to a plausible tempo.
  @Test func aBpmTheBridgeCannotCarryIsDroppedRatherThanWrapped() {
    #expect(PageSuggester.Suggestion(bpm: 65656).bridged.bpm == nil)
    #expect(PageSuggester.Suggestion(bpm: -1).bridged.bpm == nil)
  }

  /// Phase D's field. The model is not asked for a chart, so it must not be
  /// able to fill one in by the back door.
  @Test func noChartIsEverSuggested() {
    #expect(PageSuggester.Suggestion(title: "Blues in F").bridged.chartText == nil)
  }
}
