import SharedTypes
import XCTest

@testable import Intrada

final class PracticeWeekTests: XCTestCase {
  private let calendar = PreviewCalendar.utc
  // Sunday 31 May 2026, noon UTC.
  private let reference = PracticeSessionView.previewReferenceDate

  private func date(_ year: Int, _ month: Int, _ day: Int) -> Date {
    var c = DateComponents()
    (c.year, c.month, c.day, c.hour) = (year, month, day, 12)
    return calendar.date(from: c) ?? .distantPast
  }

  func testWeekIsMondayThroughSunday() {
    let week = PracticeWeek.days(containing: reference, calendar: calendar)
    XCTAssertEqual(week.count, 7)
    // Monday 25th → Sunday 31st.
    XCTAssertEqual(calendar.component(.day, from: week.first ?? .distantPast), 25)
    XCTAssertEqual(calendar.component(.day, from: week.last ?? .distantPast), 31)
    // Each entry is start-of-day and strictly ascending by one day.
    for (a, b) in zip(week, week.dropFirst()) {
      XCTAssertEqual(calendar.date(byAdding: .day, value: 1, to: a), b)
    }
  }

  func testPracticeDaysBucketsToStartOfDay() {
    let sessions = [PracticeSessionView.previewCompleted, .previewEndedEarly]  // Sat 30, Thu 28
    let days = PracticeWeek.practiceDays(from: sessions, calendar: calendar)
    XCTAssertEqual(days.count, 2)
    XCTAssertTrue(days.contains(calendar.startOfDay(for: date(2026, 5, 30))))
    XCTAssertTrue(days.contains(calendar.startOfDay(for: date(2026, 5, 28))))
    XCTAssertFalse(days.contains(calendar.startOfDay(for: date(2026, 5, 31))))
  }

  func testSessionsOnDayFiltersAndSortsNewestFirst() {
    let early = PracticeSessionView.previewEndedEarly  // Thu 28
    let onThursday = PracticeWeek.sessions(
      on: date(2026, 5, 28), from: [.previewCompleted, early], calendar: calendar)
    XCTAssertEqual(onThursday.map(\.id), [early.id])
    XCTAssertTrue(
      PracticeWeek.sessions(
        on: date(2026, 5, 27), from: [.previewCompleted, early], calendar: calendar
      )
      .isEmpty)
  }

  func testAutoSelectPrefersTodayWhenItHasPractice() {
    let week = PracticeWeek.days(containing: date(2026, 5, 30), calendar: calendar)
    let practice = PracticeWeek.practiceDays(
      from: [.previewCompleted, .previewEndedEarly], calendar: calendar)
    let selected = PracticeWeek.selectedDay(
      forWeek: week, today: date(2026, 5, 30), practiceDays: practice, calendar: calendar)
    XCTAssertEqual(calendar.component(.day, from: selected), 30)
  }

  func testAutoSelectFallsBackToMostRecentEarlierPracticeDay() {
    let week = PracticeWeek.days(containing: reference, calendar: calendar)
    let practice = PracticeWeek.practiceDays(
      from: [.previewCompleted, .previewEndedEarly], calendar: calendar)
    // Today = Sun 31 (no practice) → most recent earlier practice day = Sat 30.
    let selected = PracticeWeek.selectedDay(
      forWeek: week, today: reference, practiceDays: practice, calendar: calendar)
    XCTAssertEqual(calendar.component(.day, from: selected), 30)
  }

  func testAutoSelectFallsBackToTodayWhenNoPractice() {
    let week = PracticeWeek.days(containing: reference, calendar: calendar)
    let selected = PracticeWeek.selectedDay(
      forWeek: week, today: reference, practiceDays: [], calendar: calendar)
    XCTAssertEqual(calendar.startOfDay(for: selected), calendar.startOfDay(for: reference))
  }

  func testWeeksSpanEarliestSessionToReference() {
    // Sessions in the week of 25–31 May; reference Wed 10 June → 3 weeks.
    let weeks = PracticeWeek.weeks(
      forSessions: [.previewCompleted, .previewEndedEarly],
      referenceDate: date(2026, 6, 10), calendar: calendar)
    XCTAssertEqual(weeks.count, 3)
    XCTAssertEqual(calendar.component(.day, from: weeks.first?.first ?? .distantPast), 25)
    XCTAssertTrue(
      weeks.last?.contains { calendar.isDate($0, inSameDayAs: date(2026, 6, 10)) } ?? false)
  }

  func testWeeksIsJustCurrentWeekWithNoSessions() {
    let weeks = PracticeWeek.weeks(forSessions: [], referenceDate: reference, calendar: calendar)
    XCTAssertEqual(weeks.count, 1)
    XCTAssertEqual(calendar.component(.day, from: weeks[0].first ?? .distantPast), 25)
  }

  func testSelectPastWeekPicksMostRecentPracticeDay() {
    let week = PracticeWeek.days(containing: date(2026, 5, 28), calendar: calendar)
    let practice = PracticeWeek.practiceDays(
      from: [.previewCompleted, .previewEndedEarly], calendar: calendar)
    // Reference (10 June) isn't in this week → most recent practice day = Sat 30.
    let selected = PracticeWeek.selectedDay(
      forWeek: week, today: date(2026, 6, 10), practiceDays: practice, calendar: calendar)
    XCTAssertEqual(calendar.component(.day, from: selected), 30)
  }

  func testSelectPastWeekWithNoPracticeFallsBackToLastDay() {
    let week = PracticeWeek.days(containing: date(2026, 5, 28), calendar: calendar)
    let selected = PracticeWeek.selectedDay(
      forWeek: week, today: date(2026, 6, 10), practiceDays: [], calendar: calendar)
    // Sunday 31 May — the week's last day.
    XCTAssertEqual(calendar.component(.day, from: selected), 31)
  }
}
