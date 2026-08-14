import XCTest

@testable import Intrada

final class SessionClockTests: XCTestCase {
  private let utc = TimeZone(identifier: "UTC")!

  /// 2026-06-05T14:30:00 UTC, the instant every fixture below shares.
  private var expected: Date {
    var components = DateComponents()
    (components.year, components.month, components.day) = (2026, 6, 5)
    (components.hour, components.minute, components.second) = (14, 30, 0)
    components.timeZone = utc
    return Calendar(identifier: .gregorian).date(from: components)!
  }

  private func assertSameSecond(_ date: Date?, _ message: String) {
    guard let date else { return XCTFail("parse returned nil — \(message)") }
    // Sub-second precision is truncated, so the instant lands within the second.
    XCTAssertLessThan(abs(date.timeIntervalSince(expected)), 1, message)
  }

  // The load-bearing case: chrono `to_rfc3339()` emits microsecond fractions
  // with a `+00:00` offset — exactly what ISO8601DateFormatter rejects (#846 class).
  func testParsesChronoMicrosecondOffsetForm() {
    assertSameSecond(
      SessionClock.parseRFC3339("2026-06-05T14:30:00.123456+00:00"), "microsecond + offset")
  }

  func testParsesNanosecondZuluForm() {
    assertSameSecond(
      SessionClock.parseRFC3339("2026-06-05T14:30:00.123456789Z"), "nanosecond + Z")
  }

  func testParsesMillisecondForm() {
    assertSameSecond(SessionClock.parseRFC3339("2026-06-05T14:30:00.123Z"), "millisecond")
  }

  func testParsesPlainNoFraction() {
    assertSameSecond(SessionClock.parseRFC3339("2026-06-05T14:30:00Z"), "no fraction")
  }

  func testRejectsGarbage() {
    XCTAssertNil(SessionClock.parseRFC3339("not a timestamp"))
  }

  // An event `now:` string must round-trip back through the parser.
  func testNowRoundTrips() {
    assertSameSecond(SessionClock.parseRFC3339(SessionClock.nowRFC3339(expected)), "now round-trip")
  }
}
