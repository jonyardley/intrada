import XCTest

@testable import Intrada

final class KeyHelperTests: XCTestCase {
  func testParsesCanonicalMajorAndMinor() {
    let major = KeyHelper.parse("F# major")
    XCTAssertEqual(major, KeyHelper.Selection(ring: 6, mode: .major, spelling: "F#"))
    let minor = KeyHelper.parse("A minor")
    XCTAssertEqual(minor, KeyHelper.Selection(ring: 0, mode: .minor, spelling: "A"))
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
    let gb = KeyHelper.parse("Gb major")
    XCTAssertEqual(gb?.ring, 6)
    XCTAssertEqual(gb?.spelling, "Gb")
    let eb = KeyHelper.parse("Eb minor")
    XCTAssertEqual(eb?.ring, 6)
    XCTAssertEqual(eb?.spelling, "Eb")
  }

  func testRejectsEmptyAndGarbage() {
    XCTAssertNil(KeyHelper.parse(""))
    XCTAssertNil(KeyHelper.parse("   "))
    XCTAssertNil(KeyHelper.parse("Lydian"))
    XCTAssertNil(KeyHelper.parse("H major"))
    XCTAssertNil(KeyHelper.parse("C## major"))
    XCTAssertNil(KeyHelper.parse("C dorian"))
  }

  func testFormatsCanonicalString() {
    XCTAssertEqual(KeyHelper.format(tonic: "F#", mode: .major), "F# major")
    XCTAssertEqual(KeyHelper.format(tonic: "Bb", mode: .minor), "Bb minor")
  }

  func testPrettifyConvertsAccidentalsOnly() {
    XCTAssertEqual(KeyHelper.prettify("F# major"), "F\u{266F} major")
    XCTAssertEqual(KeyHelper.prettify("Bb major"), "B\u{266D} major")
    XCTAssertEqual(KeyHelper.prettify("Db minor"), "D\u{266D} minor")
    // The 'b' inside mode words must survive untouched.
    XCTAssertEqual(KeyHelper.prettify("C minor"), "C minor")
  }

  func testTapSelectsThenFlipsEnharmonic() {
    // Fresh tap on the F#/Gb spoke selects the default spelling.
    let first = KeyHelper.nextValueOnTap(current: "", ring: 6, mode: .major)
    XCTAssertEqual(first.value, "F# major")
    XCTAssertFalse(first.flipped)
    // Second tap flips to the alternate.
    let second = KeyHelper.nextValueOnTap(current: first.value, ring: 6, mode: .major)
    XCTAssertEqual(second.value, "Gb major")
    XCTAssertTrue(second.flipped)
    // Third tap flips back.
    let third = KeyHelper.nextValueOnTap(current: second.value, ring: 6, mode: .major)
    XCTAssertEqual(third.value, "F# major")
    XCTAssertTrue(third.flipped)
  }

  func testTapFlipsEnharmonicOnMinorSpoke() {
    let first = KeyHelper.nextValueOnTap(current: "", ring: 5, mode: .minor)
    XCTAssertEqual(first.value, "G# minor")
    XCTAssertFalse(first.flipped)
    let second = KeyHelper.nextValueOnTap(current: first.value, ring: 5, mode: .minor)
    XCTAssertEqual(second.value, "Ab minor")
    XCTAssertTrue(second.flipped)
  }

  func testParsesRingFiveEnharmonicAlternates() {
    XCTAssertEqual(
      KeyHelper.parse("Cb major"), KeyHelper.Selection(ring: 5, mode: .major, spelling: "Cb"))
    XCTAssertEqual(
      KeyHelper.parse("Ab minor"), KeyHelper.Selection(ring: 5, mode: .minor, spelling: "Ab"))
  }

  func testEnharmonicAltOnlyExistsForAmbiguousSpokes() {
    XCTAssertNil(KeyHelper.enharmonicAlt(ring: 0, mode: .major))
    XCTAssertEqual(KeyHelper.enharmonicAlt(ring: 6, mode: .major), "Gb")
    XCTAssertEqual(KeyHelper.enharmonicAlt(ring: 6, mode: .minor), "Eb")
  }

  func testTapOnNonEnharmonicSpokeNeverFlips() {
    let result = KeyHelper.nextValueOnTap(current: "C major", ring: 0, mode: .major)
    XCTAssertEqual(result.value, "C major")
    XCTAssertFalse(result.flipped)
  }

  func testTapSwitchingSpokeIsAFreshSelection() {
    let result = KeyHelper.nextValueOnTap(current: "F# major", ring: 0, mode: .major)
    XCTAssertEqual(result.value, "C major")
    XCTAssertFalse(result.flipped)
  }

  func testAccessibilityLabelSpeaksAccidentals() {
    XCTAssertEqual(KeyHelper.accessibilityLabel("F#", mode: .major), "F sharp major")
    XCTAssertEqual(KeyHelper.accessibilityLabel("Db", mode: .minor), "D flat minor")
    XCTAssertEqual(KeyHelper.accessibilityLabel("C", mode: .major), "C major")
  }

  func testWedgeAccessibilityAnnouncesBothEnharmonicSpellings() {
    XCTAssertEqual(
      KeyHelper.wedgeAccessibilityLabel(ring: 6, mode: .major), "F sharp or G flat major")
    XCTAssertEqual(
      KeyHelper.wedgeAccessibilityLabel(ring: 6, mode: .minor), "D sharp or E flat minor")
    XCTAssertEqual(KeyHelper.wedgeAccessibilityLabel(ring: 0, mode: .major), "C major")
  }
}
