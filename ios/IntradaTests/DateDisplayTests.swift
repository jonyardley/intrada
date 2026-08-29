import Foundation
import Testing

@testable import Intrada

/// The pair of locales is the point (#1485): assert only the British form and a
/// hard-coded British pattern passes just as well.
struct DateDisplayTests {
  private var calendar: Calendar {
    var c = Calendar(identifier: .gregorian)
    c.timeZone = TimeZone(identifier: "UTC")!
    return c
  }

  private var friday28August2026: Date {
    calendar.date(from: DateComponents(year: 2026, month: 8, day: 28))!
  }

  private func british<T>(_ body: (Locale) -> T) -> T { body(Locale(identifier: "en_GB")) }
  private func american<T>(_ body: (Locale) -> T) -> T { body(Locale(identifier: "en_US")) }

  @Test("A British reader gets the day before the month")
  func britishDayComesFirst() {
    #expect(
      british { DateDisplay.day(friday28August2026, locale: $0, calendar: calendar) } == "28 Aug")
  }

  @Test("An American reader gets their own order from the same call")
  func americanMonthComesFirst() {
    #expect(
      american { DateDisplay.day(friday28August2026, locale: $0, calendar: calendar) } == "Aug 28")
  }

  @Test("The weekday form keeps the design's middle dot in both regions")
  func weekdayFormKeepsTheMiddleDot() {
    #expect(
      british {
        DateDisplay.weekdayAndDay(friday28August2026, locale: $0, calendar: calendar)
      } == "Fri · 28 Aug")
    #expect(
      american {
        DateDisplay.weekdayAndDay(friday28August2026, locale: $0, calendar: calendar)
      } == "Fri · Aug 28")
  }
}
