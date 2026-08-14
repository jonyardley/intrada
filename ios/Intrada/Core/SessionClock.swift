import Foundation

/// RFC3339 ⇄ `Date` for the session player. The core emits timestamps via chrono
/// `to_rfc3339()` (micro/nanosecond fractional precision) and accepts any RFC3339
/// string for event `now:` fields. `ISO8601DateFormatter.withFractionalSeconds`
/// only parses millisecond precision and the plain option rejects any fraction at
/// all — so a 6–9 digit chrono fraction would parse to `nil`. Parsing here
/// normalizes an over-long fraction to milliseconds first.
enum SessionClock {
  /// RFC3339 string for an event `now:` argument (millisecond precision + `Z`).
  /// chrono's serde deserializer parses this on the core side.
  static func nowRFC3339(_ date: Date = Date()) -> String {
    fractionalFormatter().string(from: date)
  }

  /// `MM:SS` (or `H:MM:SS` past an hour) for the live count-up timer. Negative
  /// inputs clamp to zero.
  static func clockDisplay(_ seconds: Int) -> String {
    let total = max(0, seconds)
    let hours = total / 3600
    let minutes = (total % 3600) / 60
    let secs = total % 60
    return hours > 0
      ? String(format: "%d:%02d:%02d", hours, minutes, secs)
      : String(format: "%02d:%02d", minutes, secs)
  }

  /// Parse a core-emitted RFC3339 timestamp, tolerant of chrono's >3-digit
  /// fractional seconds (which `ISO8601DateFormatter` otherwise rejects).
  static func parseRFC3339(_ string: String) -> Date? {
    let fractional = fractionalFormatter()
    if let date = fractional.date(from: string) { return date }
    let plain = ISO8601DateFormatter()
    plain.formatOptions = [.withInternetDateTime]
    if let date = plain.date(from: string) { return date }
    if let normalized = millisecondTruncated(string),
      let date = fractional.date(from: normalized)
    {
      return date
    }
    return nil
  }

  // A fresh instance per call: `ISO8601DateFormatter` isn't `Sendable`, so it
  // can't be a shared `static let` under Swift 6 strict concurrency. Creation is
  // cheap at our rate (one parse per timer tick).
  private static func fractionalFormatter() -> ISO8601DateFormatter {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return formatter
  }

  /// Truncate a fractional-second run longer than 3 digits to milliseconds so
  /// `ISO8601DateFormatter` accepts it (e.g. `.123456789` → `.123`). Returns
  /// `nil` when there's nothing to normalize.
  private static func millisecondTruncated(_ string: String) -> String? {
    guard let dot = string.firstIndex(of: ".") else { return nil }
    let fractionStart = string.index(after: dot)
    let fractionEnd =
      string[fractionStart...].firstIndex { !$0.isNumber } ?? string.endIndex
    let digits = string[fractionStart..<fractionEnd]
    guard digits.count > 3 else { return nil }
    return String(string[..<fractionStart]) + digits.prefix(3) + string[fractionEnd...]
  }
}
