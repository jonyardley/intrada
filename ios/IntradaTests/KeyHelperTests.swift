import SharedTypes
import XCTest

@testable import Intrada

final class KeyHelperTests: XCTestCase {
  private func key(_ letter: Letter, _ accidental: Accidental = .natural, _ mode: Modality?)
    -> Key
  {
    Key(letter: letter, accidental: accidental, mode: mode)
  }

  private func spelling(_ key: Key) -> String? {
    KeyHelper.selection(key)?.spelling
  }

  func testSelectionReadsTheCoresRule() {
    XCTAssertEqual(
      KeyHelper.selection(key(.f, .sharp, .major)),
      KeyHelper.Selection(ring: 6, mode: .major, spelling: "F#"))
    XCTAssertEqual(
      KeyHelper.selection(key(.e, .flat, .minor)),
      KeyHelper.Selection(ring: 6, mode: .minor, spelling: "Eb"))
  }

  func testPresetsFollowTheCircleFromC() {
    XCTAssertEqual(
      KeyHelper.circle(.major).compactMap(spelling),
      ["C", "G", "D", "A", "E", "B", "Gb", "Db", "Ab", "Eb", "Bb", "F"])
    XCTAssertEqual(
      KeyHelper.circle(.minor).compactMap(spelling),
      ["A", "E", "B", "F#", "C#", "G#", "Eb", "Bb", "F", "C", "G", "D"])
  }

  func testPrettifyConvertsAccidentalsOnly() {
    XCTAssertEqual(KeyHelper.prettify("F#"), "F\u{266F}")
    XCTAssertEqual(KeyHelper.prettify("Bb major"), "B\u{266D} major")
    // The 'b' inside mode words must survive untouched.
    XCTAssertEqual(KeyHelper.prettify("C minor"), "C minor")
  }

  func testDisplayIsTheCoresLabel() {
    XCTAssertEqual(KeyHelper.display(key(.f, .sharp, .major)), "F\u{266F} major")
    XCTAssertEqual(KeyHelper.display(key(.d, .flat, .minor)), "D\u{266D} minor")
    XCTAssertEqual(KeyHelper.display(key(.g, .natural, nil)), "G")
  }

  func testTapSelectsThenFlipsEnharmonic() throws {
    let first = try XCTUnwrap(KeyHelper.nextOnTap(current: nil, ring: 6, mode: .major))
    XCTAssertEqual(first.key, key(.g, .flat, .major))
    XCTAssertFalse(first.flipped)
    let second = try XCTUnwrap(KeyHelper.nextOnTap(current: first.key, ring: 6, mode: .major))
    XCTAssertEqual(second.key, key(.f, .sharp, .major))
    XCTAssertTrue(second.flipped)
    let third = try XCTUnwrap(KeyHelper.nextOnTap(current: second.key, ring: 6, mode: .major))
    XCTAssertEqual(third.key, key(.g, .flat, .major))
    XCTAssertTrue(third.flipped)
  }

  func testTapFlipsEnharmonicOnMinorSpoke() throws {
    let first = try XCTUnwrap(KeyHelper.nextOnTap(current: nil, ring: 5, mode: .minor))
    XCTAssertEqual(first.key, key(.g, .sharp, .minor))
    XCTAssertFalse(first.flipped)
    let second = try XCTUnwrap(KeyHelper.nextOnTap(current: first.key, ring: 5, mode: .minor))
    XCTAssertEqual(second.key, key(.a, .flat, .minor))
    XCTAssertTrue(second.flipped)
  }

  func testTapOnNonEnharmonicSpokeNeverFlips() throws {
    let result = try XCTUnwrap(
      KeyHelper.nextOnTap(current: key(.c, .natural, .major), ring: 0, mode: .major))
    XCTAssertEqual(result.key, key(.c, .natural, .major))
    XCTAssertFalse(result.flipped)
  }

  func testTapSwitchingSpokeIsAFreshSelection() throws {
    let result = try XCTUnwrap(
      KeyHelper.nextOnTap(current: key(.f, .sharp, .major), ring: 0, mode: .major))
    XCTAssertEqual(result.key, key(.c, .natural, .major))
    XCTAssertFalse(result.flipped)
  }

  func testATapOffTheWheelIsIgnored() {
    XCTAssertNil(KeyHelper.nextOnTap(current: key(.c, .natural, .major), ring: 12, mode: .major))
    XCTAssertNil(KeyHelper.nextOnTap(current: key(.c, .natural, .major), ring: -1, mode: .major))
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
