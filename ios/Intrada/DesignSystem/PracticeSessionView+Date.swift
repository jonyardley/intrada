import Foundation
import SharedTypes

extension PracticeSessionView {
  /// Parsed start instant. Routed through `SessionClock` so chrono's
  /// micro/nanosecond fractions parse (the bare `.withFractionalSeconds`
  /// formatter only handles milliseconds).
  var startedDate: Date? { SessionClock.parseRFC3339(startedAt) }

  /// Locale and calendar come from the caller's SwiftUI environment, never
  /// `Locale.current`: the template reorders by region and the day bucket
  /// shifts by timezone, which snapshot hosts pin and production must not.
  func dateDisplay(locale: Locale, calendar: Calendar) -> String {
    guard let date = startedDate else { return "" }
    if calendar.isDateInToday(date) { return "Today" }
    if calendar.isDateInYesterday(date) { return "Yesterday" }
    let formatter = DateFormatter()
    formatter.locale = locale
    formatter.calendar = calendar
    formatter.timeZone = calendar.timeZone
    formatter.setLocalizedDateFormatFromTemplate("EEEdMMM")
    return formatter.string(from: date)
  }

  /// "3 pieces", "2 exercises", or "5 items" when the session spans both.
  var itemCountDisplay: String {
    let count = entries.count
    let noun: String
    if entries.allSatisfy({ $0.itemType == .piece }) {
      noun = count == 1 ? "piece" : "pieces"
    } else if entries.allSatisfy({ $0.itemType == .exercise }) {
      noun = count == 1 ? "exercise" : "exercises"
    } else {
      noun = count == 1 ? "item" : "items"
    }
    return "\(count) \(noun)"
  }
}
