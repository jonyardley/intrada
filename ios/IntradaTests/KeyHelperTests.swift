import SharedTypes
import XCTest

@testable import Intrada

final class KeyHelperTests: XCTestCase {
  func testParsesCanonicalMajorAndMinor() {
    XCTAssertEqual(
      KeyHelper.parse("F# major"), KeyHelper.Selection(ring: 6, mode: .major, spelling: "F#"))
    XCTAssertEqual(
      KeyHelper.parse("A minor"), KeyHelper.Selection(ring: 0, mode: .minor, spelling: "A"))
  }

  func testParseIsCaseAndAccidentalInsensitive() {
    XCTAssertEqual(KeyHelper.parse("c MAJOR")?.spelling, "C")
    XCTAssertEqual(KeyHelper.parse("f# MINOR")?.mode, .minor)
    // Unicode accidentals normalise to ASCII.
    let s = KeyHelper.parse("C\u{266F} minor")
    XCTAssertEqual(s?.spelling, "C#")
    XCTAssertEqual(s?.mode, .minor)
  }

  func testParsesEnharmonicAlternateSpelling() {
    XCTAssertEqual(KeyHelper.parse("Gb major")?.spelling, "Gb")
    XCTAssertEqual(
      KeyHelper.parse("Eb minor"), KeyHelper.Selection(ring: 6, mode: .minor, spelling: "Eb"))
  }

  func testParsesRingFiveEnharmonicAlternates() {
    XCTAssertEqual(
      KeyHelper.parse("Cb major"), KeyHelper.Selection(ring: 5, mode: .major, spelling: "Cb"))
    XCTAssertEqual(
      KeyHelper.parse("Ab minor"), KeyHelper.Selection(ring: 5, mode: .minor, spelling: "Ab"))
  }

  func testRejectsEmptyAndGarbage() {
    XCTAssertNil(KeyHelper.parse(""))
    XCTAssertNil(KeyHelper.parse("   "))
    XCTAssertNil(KeyHelper.parse("Lydian"))
    XCTAssertNil(KeyHelper.parse("H major"))
    XCTAssertNil(KeyHelper.parse("C## major"))
    XCTAssertNil(KeyHelper.parse("C dorian"))
  }

  func testSelectionPrefersStructuredAndFallsBackToLegacy() {
    XCTAssertEqual(
      KeyHelper.selection(key: "F#", modality: .major),
      KeyHelper.Selection(ring: 6, mode: .major, spelling: "F#"))
    // Legacy combined-string path: no modality given.
    XCTAssertEqual(
      KeyHelper.selection(key: "F# major", modality: nil),
      KeyHelper.Selection(ring: 6, mode: .major, spelling: "F#"))
    XCTAssertNil(KeyHelper.selection(key: "", modality: nil))
  }

  func testPrettifyConvertsAccidentalsOnly() {
    XCTAssertEqual(KeyHelper.prettify("F#"), "F\u{266F}")
    XCTAssertEqual(KeyHelper.prettify("Bb major"), "B\u{266D} major")
    // The 'b' inside mode words must survive untouched.
    XCTAssertEqual(KeyHelper.prettify("C minor"), "C minor")
  }

  func testDisplayComposesAndHandlesLegacy() {
    XCTAssertEqual(KeyHelper.display(key: "F#", modality: .major), "F\u{266F} Major")
    XCTAssertEqual(KeyHelper.display(key: "Db", modality: .minor), "D\u{266D} minor")
    // Legacy combined value with no modality still prettifies.
    XCTAssertEqual(KeyHelper.display(key: "F# major", modality: nil), "F\u{266F} major")
    XCTAssertNil(KeyHelper.display(key: "", modality: nil))
    XCTAssertNil(KeyHelper.display(key: nil, modality: .major))
  }

  func testTapSelectsThenFlipsEnharmonic() {
    let first = KeyHelper.nextOnTap(currentKey: "", currentModality: nil, ring: 6, mode: .major)
    XCTAssertEqual(first.tonic, "Gb")
    XCTAssertEqual(first.modality, .major)
    XCTAssertFalse(first.flipped)
    let second = KeyHelper.nextOnTap(
      currentKey: first.tonic, currentModality: first.modality, ring: 6, mode: .major)
    XCTAssertEqual(second.tonic, "F#")
    XCTAssertTrue(second.flipped)
    let third = KeyHelper.nextOnTap(
      currentKey: second.tonic, currentModality: second.modality, ring: 6, mode: .major)
    XCTAssertEqual(third.tonic, "Gb")
    XCTAssertTrue(third.flipped)
  }

  func testTapFlipsEnharmonicOnMinorSpoke() {
    let first = KeyHelper.nextOnTap(currentKey: "", currentModality: nil, ring: 5, mode: .minor)
    XCTAssertEqual(first.tonic, "G#")
    XCTAssertFalse(first.flipped)
    let second = KeyHelper.nextOnTap(
      currentKey: first.tonic, currentModality: first.modality, ring: 5, mode: .minor)
    XCTAssertEqual(second.tonic, "Ab")
    XCTAssertTrue(second.flipped)
  }

  func testTapOnNonEnharmonicSpokeNeverFlips() {
    let result = KeyHelper.nextOnTap(
      currentKey: "C", currentModality: .major, ring: 0, mode: .major)
    XCTAssertEqual(result.tonic, "C")
    XCTAssertFalse(result.flipped)
  }

  func testTapSwitchingSpokeIsAFreshSelection() {
    let result = KeyHelper.nextOnTap(
      currentKey: "F#", currentModality: .major, ring: 0, mode: .major)
    XCTAssertEqual(result.tonic, "C")
    XCTAssertEqual(result.modality, .major)
    XCTAssertFalse(result.flipped)
  }

  func testEnharmonicAltOnlyExistsForAmbiguousSpokes() {
    XCTAssertNil(KeyHelper.enharmonicAlt(ring: 0, mode: .major))
    XCTAssertEqual(KeyHelper.enharmonicAlt(ring: 6, mode: .major), "F#")
    XCTAssertEqual(KeyHelper.enharmonicAlt(ring: 6, mode: .minor), "D#")
  }

  func testAccessibilityLabelSpeaksAccidentals() {
    XCTAssertEqual(KeyHelper.accessibilityLabel("F#", mode: .major), "F sharp Major")
    XCTAssertEqual(KeyHelper.accessibilityLabel("Db", mode: .minor), "D flat minor")
    XCTAssertEqual(KeyHelper.accessibilityLabel("C", mode: .major), "C Major")
  }

  func testWedgeAccessibilityAnnouncesBothEnharmonicSpellings() {
    XCTAssertEqual(
      KeyHelper.wedgeAccessibilityLabel(ring: 6, mode: .major), "G flat or F sharp Major")
    XCTAssertEqual(
      KeyHelper.wedgeAccessibilityLabel(ring: 6, mode: .minor), "E flat or D sharp minor")
    XCTAssertEqual(KeyHelper.wedgeAccessibilityLabel(ring: 0, mode: .major), "C Major")
  }
}
