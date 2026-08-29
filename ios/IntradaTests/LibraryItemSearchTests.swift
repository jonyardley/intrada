import Foundation
import SharedTypes
import Testing

@testable import Intrada

/// Each field carries a word none of the others do, so dropping any one branch
/// of `matchesSearch` fails a case here (#1440).
struct LibraryItemSearchTests {
  private static func exercise(
    title: String = "Hanon No. 1", subtitle: String = "Charles-Louis",
    notes: String? = "left hand only", tags: [String] = ["warm-up"]
  ) -> LibraryItemView {
    LibraryItemView(
      id: "exercise-1", itemType: .exercise, title: title, subtitle: subtitle,
      key: "C", modality: .major, tempo: "108 BPM", tempoMarking: nil, tempoBpm: 108,
      notes: notes, tags: tags, createdAt: "", updatedAt: "", practice: nil,
      latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: [],
      exerciseContexts: [], scaffoldPreview: nil, chordChart: nil, variants: [])
  }

  @Test(
    "words a musician would type find the exercise",
    arguments: ["No. 1", "no. 1", "Charles-Louis", "warm-up", "left hand", "  No. 1  ", ""])
  func matchesTheFieldsTheLibraryMatches(text: String) {
    #expect(Self.exercise().matchesSearch(text))
  }

  @Test("a word in none of those fields does not match")
  func missesWhatIsNotThere() {
    #expect(!Self.exercise().matchesSearch("Debussy"))
  }

  @Test("key and tempo are not searchable, matching the Library")
  func doesNotMatchTheMetaLine() {
    #expect(!Self.exercise(notes: nil, tags: []).matchesSearch("108"))
  }
}
