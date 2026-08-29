import Foundation
import SharedTypes

extension ItemPracticeSummary {
  /// Score history (newest first) mapped to `RecentSessions` rows, formatting
  /// each RFC3339 `sessionDate` as the compact `EEE · MMM d` the design uses.
  /// Locale/calendar come from the SwiftUI environment so snapshot hosts stay
  /// deterministic (see `SessionCard.dateDisplay`).
  func recentSessionRows(locale: Locale, calendar: Calendar) -> [RecentSession] {
    let formatter = DateFormatter()
    formatter.locale = locale
    formatter.calendar = calendar
    formatter.timeZone = calendar.timeZone
    formatter.dateFormat = "EEE '·' MMM d"
    return scoreHistory.map { entry in
      RecentSession(
        id: entry.sessionId,
        score: Int(entry.score),
        dateText: SessionClock.parseRFC3339(entry.sessionDate)
          .map { formatter.string(from: $0) } ?? "")
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

    let formatter = DateFormatter()
    formatter.locale = locale
    formatter.calendar = calendar
    formatter.timeZone = calendar.timeZone
    formatter.dateFormat = "d MMM"
    let text = { (mark: TempoTrendMark?) in mark.map { formatter.string(from: $0.date) } ?? "" }
    return TempoTrendDisplay(
      series: TempoTrendSeries(marks: marks),
      hasTrend: tempoTrend.hasTrend,
      startDateText: text(marks.first),
      endDateText: text(marks.last))
  }
}

/// Shell-side presentation formatting shared by any screen with a structured
/// `tempoMarking` / `tempoBpm` pair (the core's call, not iOS's — see
/// `LibraryItemView`/`ActiveSessionView`), so the card, detail, and
/// focus-player screens all agree on "Allegro · ♩ = 132".
enum TempoFormatting {
  /// Visual tempo: "Allegro · ♩ = 132". ♩ is U+2669 (no SF Symbol equivalent).
  static func display(marking: String?, bpm: UInt16?) -> String? {
    let parts = [marking, bpm.map { "♩ = \($0)" }].compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  /// Spoken tempo for VoiceOver — spells the BPM out instead of the ♩ glyph.
  static func spoken(marking: String?, bpm: UInt16?) -> String? {
    let parts = [
      marking.flatMap { $0.isEmpty ? nil : $0 }, bpm.map { "\($0) beats per minute" },
    ]
    .compactMap { $0 }
    return parts.isEmpty ? nil : parts.joined(separator: ", ")
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
      let formatter = DateFormatter()
      formatter.locale = locale
      formatter.calendar = calendar
      formatter.timeZone = calendar.timeZone
      formatter.dateFormat = "d MMM"
      parts.append(formatter.string(from: date))
    }
    return parts.joined(separator: " · ")
  }

  /// The whole VoiceOver announcement for a "Used in" row; the ring beside it is
  /// decorative (#1468). A row with no practice stops at "not practised
  /// together yet" — the ring's unrated rest says the same thing, and "not yet
  /// rated" after it would be the fact twice.
  var spokenRow: String {
    var parts = [rowTitle]
    if pieceRemoved { parts.append("removed from the library") }
    guard sessionCount > 0 else {
      parts.append("not practised together yet")
      return parts.joined(separator: ", ")
    }
    if let score = latestScore {
      parts.append("mark \(score) of 10")
    } else {
      parts.append("not yet rated")
    }
    let n = Int(sessionCount)
    parts.append("\(n) \(n == 1 ? "session" : "sessions")")
    return parts.joined(separator: ", ")
  }
}

extension LibraryItemView {
  var keyDisplay: String? {
    KeyHelper.display(key: key, modality: modality)
  }

  var tempoDisplay: String? { TempoFormatting.display(marking: tempoMarking, bpm: tempoBpm) }

  var tempoSpoken: String? { TempoFormatting.spoken(marking: tempoMarking, bpm: tempoBpm) }
}

extension ActiveSessionView {
  /// The current item's own declared tempo (the practice target) — distinct
  /// from `achievedTempo` on a `SetlistEntryView`, which is logged after the
  /// fact. "Allegro · ♩ = 132".
  var currentItemTempoDisplay: String? {
    TempoFormatting.display(marking: currentItemTempoMarking, bpm: currentItemTempoBpm)
  }

  var currentItemTempoSpoken: String? {
    TempoFormatting.spoken(marking: currentItemTempoMarking, bpm: currentItemTempoBpm)
  }
}
