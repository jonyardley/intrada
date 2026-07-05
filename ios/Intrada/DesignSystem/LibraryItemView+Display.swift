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
}

/// Shell-side presentation formatting for a library item. The core exposes
/// structured `tempoMarking` / `tempoBpm`; how iOS renders them ("Allegro · ♩ =
/// 132") is the shell's call, shared here so the card and detail agree.
extension LibraryItemView {
  var keyDisplay: String? {
    KeyHelper.display(key: key, modality: modality)
  }

  /// Visual tempo: "Allegro · ♩ = 132". ♩ is U+2669 (no SF Symbol equivalent).
  var tempoDisplay: String? {
    let parts = [tempoMarking, tempoBpm.map { "♩ = \($0)" }]
      .compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  /// Spoken tempo for VoiceOver — spells the BPM out instead of the ♩ glyph.
  var tempoSpoken: String? {
    let parts = [
      tempoMarking.flatMap { $0.isEmpty ? nil : $0 }, tempoBpm.map { "\($0) beats per minute" },
    ]
    .compactMap { $0 }
    return parts.isEmpty ? nil : parts.joined(separator: ", ")
  }
}
