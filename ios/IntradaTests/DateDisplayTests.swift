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

  private var british: DateDisplay {
    DateDisplay(locale: Locale(identifier: "en_GB"), calendar: calendar)
  }
  private var american: DateDisplay {
    DateDisplay(locale: Locale(identifier: "en_US"), calendar: calendar)
  }

  @Test("A British reader gets the day before the month")
  func britishDayComesFirst() {
    #expect(british.day(friday28August2026) == "28 Aug")
  }

  @Test("An American reader gets their own order from the same call")
  func americanMonthComesFirst() {
    #expect(american.day(friday28August2026) == "Aug 28")
  }

  @Test("The weekday form keeps the design's middle dot in both regions")
  func weekdayFormKeepsTheMiddleDot() {
    #expect(british.weekdayAndDay(friday28August2026) == "Fri · 28 Aug")
    #expect(american.weekdayAndDay(friday28August2026) == "Fri · Aug 28")
  }
}
