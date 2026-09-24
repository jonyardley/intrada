import SharedTypes
import XCTest

@testable import Intrada

final class KeyHelperTests: XCTestCase {
  func testSelectionMatchesStructuredTonicOnly() {
    XCTAssertEqual(
      KeyHelper.selection(key: "F#", modality: .major),
      KeyHelper.Selection(ring: 6, mode: .major, spelling: "F#"))
    XCTAssertNil(KeyHelper.selection(key: "F# major", modality: nil))
    XCTAssertNil(KeyHelper.selection(key: "", modality: nil))
  }

  func testPrettifyConvertsAccidentalsOnly() {
    XCTAssertEqual(KeyHelper.prettify("F#"), "F\u{266F}")
    XCTAssertEqual(KeyHelper.prettify("Bb major"), "B\u{266D} major")
    // The 'b' inside mode words must survive untouched.
    XCTAssertEqual(KeyHelper.prettify("C minor"), "C minor")
  }

  func testDisplayComposesAndHandlesLegacy() {
    XCTAssertEqual(KeyHelper.display(key: "F#", modality: .major), "F\u{266F} major")
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
    XCTAssertEqual(KeyHelper.accessibilityLabel("F#", mode: .major), "F sharp major")
    XCTAssertEqual(KeyHelper.accessibilityLabel("Db", mode: .minor), "D flat minor")
    XCTAssertEqual(KeyHelper.accessibilityLabel("C", mode: .major), "C major")
  }

  func testWedgeAccessibilityAnnouncesBothEnharmonicSpellings() {
    XCTAssertEqual(
      KeyHelper.wedgeAccessibilityLabel(ring: 6, mode: .major), "G flat or F sharp major")
    XCTAssertEqual(
      KeyHelper.wedgeAccessibilityLabel(ring: 6, mode: .minor), "E flat or D sharp minor")
    XCTAssertEqual(KeyHelper.wedgeAccessibilityLabel(ring: 0, mode: .major), "C major")
  }
}
