import Foundation
import SharedTypes

extension PracticeSessionView {
  /// Parsed start instant. chrono's `to_rfc3339` emits fractional seconds, so
  /// fall back to the plain internet-date form.
  var startedDate: Date? {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    if let date = formatter.date(from: startedAt) { return date }
    formatter.formatOptions = [.withInternetDateTime]
    return formatter.date(from: startedAt)
  }
}

/// Pure date math for the Practice week strip. Shell-side per the dumb-pipe
/// call (#904 tracks lifting this into the core once Track needs the same
/// derived data). Every function takes an explicit `calendar` so callers pin
/// the timezone/locale for deterministic tests.
enum PracticeWeek {
  /// Monday-anchored week (7 start-of-day dates, ascending) containing `date`.
  static func days(containing date: Date, calendar: Calendar) -> [Date] {
    var calendar = calendar
    calendar.firstWeekday = 2  // Monday
    let start = calendar.startOfDay(for: date)
    guard let week = calendar.dateInterval(of: .weekOfYear, for: start) else { return [start] }
    return (0..<7).compactMap { calendar.date(byAdding: .day, value: $0, to: week.start) }
  }

  static func practiceDays(from sessions: [PracticeSessionView], calendar: Calendar)
    -> Swift.Set<Date>
  {
    // Swift.Set: the domain `Set` entity (SharedTypes) shadows Swift.Set here.
    Swift.Set(sessions.compactMap { $0.startedDate.map(calendar.startOfDay(for:)) })
  }

  /// Sorted newest-first.
  static func sessions(on day: Date, from sessions: [PracticeSessionView], calendar: Calendar)
    -> [PracticeSessionView]
  {
    sessions
      .filter { $0.startedDate.map { calendar.isDate($0, inSameDayAs: day) } ?? false }
      .sorted { ($0.startedDate ?? .distantPast) > ($1.startedDate ?? .distantPast) }
  }

  /// Which day to open on: today if it has practice, else the most recent
  /// earlier day this week with practice, else today.
  static func autoSelectedDay(
    in week: [Date], today: Date, practiceDays: Swift.Set<Date>, calendar: Calendar
  ) -> Date {
    let todayStart = calendar.startOfDay(for: today)
    if practiceDays.contains(todayStart) { return todayStart }
    return week.filter { $0 <= todayStart && practiceDays.contains($0) }.max() ?? todayStart
  }
}
