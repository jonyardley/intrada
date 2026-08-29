import Foundation
import SharedTypes
import Testing

@testable import Intrada

/// Since #1363 a "Used in" row can exist with no practice behind it, so the
/// zero case has to read as a fresh link rather than a bad result.
struct ExerciseUsageDisplayTests {
  private let locale = Locale(identifier: "en_GB")
  private var calendar: Calendar {
    var c = Calendar(identifier: .gregorian)
    c.timeZone = TimeZone(identifier: "UTC")!
    return c
  }

  private func usage(
    title: String = "Clair de Lune",
    subtitle: String? = "Debussy",
    linked: Bool = true,
    latestScore: UInt8? = nil,
    sessionCount: UInt64 = 0,
    lastPracticedAt: String? = nil,
    pieceRemoved: Bool = false
  ) -> ExerciseUsageView {
    ExerciseUsageView(
      piece: PieceRefView(id: "piece-1", title: title, subtitle: subtitle),
      linked: linked, latestScore: latestScore, sessionCount: sessionCount,
      lastPracticedAt: lastPracticedAt, pieceRemoved: pieceRemoved)
  }

  @Test("A linked piece with no practice says so rather than counting to zero")
  func linkedWithoutPracticeReadsAsFresh() {
    let line = usage().metaLine(locale: locale, calendar: calendar)
    #expect(line == "Debussy · not practised together yet")
    #expect(!line.contains("0"))
  }

  @Test("The zero case keeps the removed-piece prefix instead of the composer")
  func removedPieceWithoutPracticeKeepsItsPrefix() {
    let line = usage(subtitle: nil, pieceRemoved: true)
      .metaLine(locale: locale, calendar: calendar)
    #expect(line == "Removed · not practised together yet")
  }

  @Test("A practised piece still counts its sessions and dates them")
  func practisedPieceCountsSessions() {
    let line = usage(
      latestScore: 7, sessionCount: 3, lastPracticedAt: "2026-06-24T09:00:00Z"
    ).metaLine(locale: locale, calendar: calendar)
    #expect(line == "Debussy · 3 sessions · Jun 24")
  }

  @Test("One session is singular")
  func oneSessionIsSingular() {
    let line = usage(latestScore: 5, sessionCount: 1, lastPracticedAt: "2026-06-24T09:00:00Z")
      .metaLine(locale: locale, calendar: calendar)
    #expect(line == "Debussy · 1 session · Jun 24")
  }

  // #1468: the ring is decorative, so the announcement carries the mark itself
  // — and stops at "not practised together yet" when there is no mark.
  @Test("A practised row speaks its mark, since the ring is decorative")
  func practisedRowSpeaksItsMark() {
    #expect(
      usage(latestScore: 7, sessionCount: 3).spokenRow
        == "Clair de Lune, mark 7 of 10, 3 sessions")
  }

  @Test("A practised but unrated row says so")
  func practisedButUnratedRowSaysSo() {
    #expect(usage(sessionCount: 2).spokenRow == "Clair de Lune, not yet rated, 2 sessions")
  }

  @Test("A row with no practice stops at the meta line, saying nothing of the ring")
  func zeroPracticeRowStopsAtTheMetaLine() {
    let spoken = usage().spokenRow
    #expect(spoken == "Clair de Lune, not practised together yet")
    #expect(!spoken.contains("rated"))
  }

  @Test("A removed piece is announced as removed before its practice")
  func removedPieceIsAnnouncedAsRemoved() {
    #expect(
      usage(latestScore: 4, sessionCount: 1, pieceRemoved: true).spokenRow
        == "Clair de Lune, removed from the library, mark 4 of 10, 1 session")
  }
}
