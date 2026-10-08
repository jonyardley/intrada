import Foundation
import SharedTypes

extension ItemPracticeSummary {
  func recentSessionRows(locale: Locale, calendar: Calendar) -> [RecentSession] {
    let dates = DateDisplay(locale: locale, calendar: calendar)
    return scoreHistory.map { entry in
      RecentSession(
        id: entry.sessionId,
        score: Int(entry.score),
        dateText: SessionClock.parseRFC3339(entry.sessionDate)
          .map(dates.weekdayAndDay) ?? "")
    }
  }

  /// The tempo trend ready to draw, or nil when there is nothing to draw.
  /// A point whose date will not parse takes the whole card with it, rather
  /// than being dropped: the plot places points on a time axis, and a series
  /// missing one no longer matches the `hasTrend` the core computed for it.
  func tempoTrendDisplay(locale: Locale, calendar: Calendar) -> TempoTrendDisplay? {
    var marks: [TempoTrendMark] = []
    for point in tempoTrend.points {
      guard let date = SessionClock.parseRFC3339(point.sessionDate) else { return nil }
      marks.append(TempoTrendMark(date: date, tempo: point.tempo.map(Int.init)))
    }
    guard marks.contains(where: { $0.tempo != nil }) else { return nil }

    let dates = DateDisplay(locale: locale, calendar: calendar)
    let text = { (mark: TempoTrendMark?) in mark.map { dates.day($0.date) } ?? "" }
    return TempoTrendDisplay(
      series: TempoTrendSeries(marks: marks),
      hasTrend: tempoTrend.hasTrend,
      startDateText: text(marks.first),
      endDateText: text(marks.last))
  }
}

/// Short dates, always from a localised template and never a literal pattern —
/// see `docs/tone-of-voice.md` V5 for why (#1485). Hold one across a run of
/// dates: `score_history` is uncapped, so building one per date puts hundreds
/// of ICU template lookups in a SwiftUI body.
struct DateDisplay {
  private let dayFormatter: DateFormatter
  private let weekdayFormatter: DateFormatter

  init(locale: Locale, calendar: Calendar) {
    dayFormatter = Self.formatter(template: "dMMM", locale: locale, calendar: calendar)
    weekdayFormatter = Self.formatter(template: "EEE", locale: locale, calendar: calendar)
  }

  /// "28 Aug" on a British device, "Aug 28" on an American one.
  func day(_ date: Date) -> String { dayFormatter.string(from: date) }

  /// "Fri · 28 Aug". Formatted apart and joined: one "EEEdMMM" template would
  /// bring the region's own separator with it (a comma, in en_US).
  func weekdayAndDay(_ date: Date) -> String {
    "\(weekdayFormatter.string(from: date)) · \(day(date))"
  }

  private static func formatter(template: String, locale: Locale, calendar: Calendar)
    -> DateFormatter
  {
    let formatter = DateFormatter()
    formatter.locale = locale
    formatter.calendar = calendar
    formatter.timeZone = calendar.timeZone
    formatter.setLocalizedDateFormatFromTemplate(template)
    return formatter
  }
}

extension ExerciseUsageView {
  var rowTitle: String { piece?.title ?? "On its own" }

  /// "Beethoven · 3 sessions · Jul 8", or "Removed · 1 session · Jun 28" for a
  /// since-deleted piece (#1093, 2a) — composer dropped once the piece is gone.
  /// A row with no practice says so: "0 sessions" reads as a failure rather
  /// than a fresh link (#1363).
  func metaLine(locale: Locale, calendar: Calendar) -> String {
    var parts: [String] = []
    if pieceRemoved {
      parts.append("Removed")
    } else if let subtitle = piece?.subtitle, !subtitle.isEmpty {
      parts.append(subtitle)
    }
    guard sessionCount > 0 else {
      parts.append("not practised together yet")
      return parts.joined(separator: " · ")
    }
    let n = Int(sessionCount)
    parts.append("\(n) \(n == 1 ? "session" : "sessions")")
    if let date = lastPracticedAt.flatMap(SessionClock.parseRFC3339) {
      parts.append(DateDisplay(locale: locale, calendar: calendar).day(date))
    }
    return parts.joined(separator: " · ")
  }

  /// The whole VoiceOver announcement for a "Used in" row; the ring beside it is
  /// decorative (#1468). A row with no practice stops at "not practised
  /// together yet" — the ring's unrated rest says the same thing, and "not yet
  /// rated" after it would be the fact twice.
  func spokenRow(topMark: Int) -> String {
    var parts = [rowTitle]
    if pieceRemoved { parts.append("removed from the library") }
    if let linkCaption { parts.append(linkCaption) }
    guard sessionCount > 0 else {
      parts.append("not practised together yet")
      return parts.joined(separator: ", ")
    }
    if let score = latestScore {
      parts.append("mark \(score) of \(topMark)")
    } else {
      parts.append("not yet rated")
    }
    let n = Int(sessionCount)
    parts.append("\(n) \(n == 1 ? "session" : "sessions")")
    return parts.joined(separator: ", ")
  }
}

extension LibraryItemView {
  var keyDisplay: String? { keyLabel }
}

extension LinkedExerciseView {
  var keyDisplay: String? { keyLabel }

  var metaLine: String? {
    let parts = [keyDisplay, tempoLine].compactMap { $0 }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  var metaSpoken: String? {
    let parts = [keyDisplay, tempoLineSpoken].compactMap { $0 }
    return parts.isEmpty ? nil : parts.joined(separator: ", ")
  }
}
