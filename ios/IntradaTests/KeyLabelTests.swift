import Testing

@testable import Intrada

/// Ladder labels a musician would actually type, against the question the
/// library card asks of them.
struct KeyLabelTests {
  @Test(
    "labels that read as keys",
    arguments: ["C", "F#", "Bb", "F\u{266F}", "B\u{266D}", "c", " D ", "C major", "f# minor"])
  func keyLabels(_ label: String) {
    #expect(KeyHelper.isKeyLabel(label))
  }

  @Test(
    "labels that do not",
    arguments: [
      "Root position", "1st inversion", "Land on the 3rd", "Hands together", "H", "", "Step 1",
      "Cx", "Bbb",
    ])
  func nonKeyLabels(_ label: String) {
    #expect(!KeyHelper.isKeyLabel(label))
  }

  @Test func mixedLadderIsNotKeys() {
    let ladder = ["C", "G", "Root position"]
    #expect(!ladder.allSatisfy(KeyHelper.isKeyLabel))
    #expect(["C", "G", "D"].allSatisfy(KeyHelper.isKeyLabel))
  }
}
