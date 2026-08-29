import SharedTypes
import Testing

@testable import Intrada

/// Ladder labels a musician would actually type, and what the card calls them.
@MainActor
struct KeyLabelTests {
  @Test(
    "labels that read as keys",
    arguments: [
      "C", "F#", "Bb", "F\u{266F}", "B\u{266D}", "c", " D ", "C major", "f# minor",
      // Spelled out and off the wheel: "C major, C♯ major, D♯ major…" is all keys.
      "D\u{266F} major", "G# major", "Db minor",
    ])
  func keyLabels(_ label: String) {
    #expect(KeyHelper.isKeyLabel(label))
  }

  @Test(
    "labels that do not",
    arguments: [
      "Root position", "1st inversion", "Land on the 3rd", "Hands together", "Step 1",
      "Am", "Dm", "C dorian", "G mixolydian", "C/E", "H", "",
    ])
  func nonKeyLabels(_ label: String) {
    #expect(!KeyHelper.isKeyLabel(label))
  }

  @Test(
    "what the card calls a ladder",
    arguments: [
      (["C", "G", "D"], "3 keys"),
      (["C major", "C\u{266F} major", "D major"], "3 keys"),
      (["Root position", "1st inversion"], "2 steps"),
      // All-or-nothing: one non-key rung and "keys" would be a lie about it.
      (["C", "G", "Hands together"], "3 steps"),
    ])
  func ladderLabel(_ labels: [String], _ expected: String) {
    #expect(LibraryItemCard.ladderLabel(labels.map(Self.rung)) == expected)
  }

  private static func rung(_ label: String) -> VariantView {
    VariantView(
      id: label, label: label, position: 0, latestScore: nil, scoreHistory: [],
      isSolid: false, isCurrent: false)
  }
}
