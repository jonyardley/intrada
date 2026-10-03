import SharedTypes
import XCTest

@testable import Intrada

final class KeyHelperTests: XCTestCase {
  func testSelectionReadsTheCoresRule() {
    XCTAssertEqual(
      KeyHelper.selection(key: "F#", modality: .major),
      KeyHelper.Selection(ring: 6, mode: .major, spelling: "F#"))
    XCTAssertEqual(
      KeyHelper.selection(key: "F# major", modality: nil),
      KeyHelper.Selection(ring: 6, mode: .major, spelling: "F#"))
    XCTAssertNil(KeyHelper.selection(key: "", modality: nil))
  }

  func testPresetsFollowTheCircleFromC() {
    XCTAssertEqual(
      KeyHelper.circle(.major), ["C", "G", "D", "A", "E", "B", "Gb", "Db", "Ab", "Eb", "Bb", "F"])
    XCTAssertEqual(
      KeyHelper.circle(.minor), ["A", "E", "B", "F#", "C#", "G#", "Eb", "Bb", "F", "C", "G", "D"])
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

  func testTapSelectsThenFlipsEnharmonic() throws {
    let first = try XCTUnwrap(
      KeyHelper.nextOnTap(currentKey: "", currentModality: nil, ring: 6, mode: .major))
    XCTAssertEqual(first.tonic, "Gb")
    XCTAssertEqual(first.modality, .major)
    XCTAssertFalse(first.flipped)
    let second = try XCTUnwrap(
      KeyHelper.nextOnTap(
        currentKey: first.tonic, currentModality: first.modality, ring: 6, mode: .major))
    XCTAssertEqual(second.tonic, "F#")
    XCTAssertTrue(second.flipped)
    let third = try XCTUnwrap(
      KeyHelper.nextOnTap(
        currentKey: second.tonic, currentModality: second.modality, ring: 6, mode: .major))
    XCTAssertEqual(third.tonic, "Gb")
    XCTAssertTrue(third.flipped)
  }

  func testTapFlipsEnharmonicOnMinorSpoke() throws {
    let first = try XCTUnwrap(
      KeyHelper.nextOnTap(currentKey: "", currentModality: nil, ring: 5, mode: .minor))
    XCTAssertEqual(first.tonic, "G#")
    XCTAssertFalse(first.flipped)
    let second = try XCTUnwrap(
      KeyHelper.nextOnTap(
        currentKey: first.tonic, currentModality: first.modality, ring: 5, mode: .minor))
    XCTAssertEqual(second.tonic, "Ab")
    XCTAssertTrue(second.flipped)
  }

  func testTapOnNonEnharmonicSpokeNeverFlips() throws {
    let result = try XCTUnwrap(
      KeyHelper.nextOnTap(currentKey: "C", currentModality: .major, ring: 0, mode: .major))
    XCTAssertEqual(result.tonic, "C")
    XCTAssertFalse(result.flipped)
  }

  func testTapSwitchingSpokeIsAFreshSelection() throws {
    let result = try XCTUnwrap(
      KeyHelper.nextOnTap(currentKey: "F#", currentModality: .major, ring: 0, mode: .major))
    XCTAssertEqual(result.tonic, "C")
    XCTAssertEqual(result.modality, .major)
    XCTAssertFalse(result.flipped)
  }

  func testATapOffTheWheelIsIgnored() {
    XCTAssertNil(
      KeyHelper.nextOnTap(currentKey: "C", currentModality: .major, ring: 12, mode: .major))
    XCTAssertNil(
      KeyHelper.nextOnTap(currentKey: "C", currentModality: .major, ring: -1, mode: .major))
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
