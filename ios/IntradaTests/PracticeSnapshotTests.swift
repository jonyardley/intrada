import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class PracticeSnapshotTests: SnapshotTestCase {
  func testPracticeScreen() {
    // Pin the week: the empty state still shows the strip, and the live core's
    // week would shift day-to-day and flake.
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPracticeEmpty), as: config)
  }

  func testPracticeScreenPopulated() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPractice), as: config)
  }

  func testPracticeScreenSuggestion() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPracticeSuggestion), as: config)
  }

  /// Dismissed but the core still has one to offer: the plain hero carries a
  /// way back to it (#1618).
  func testPracticeScreenSuggestionDismissed() {
    assertSnapshot(
      of: host(
        PracticeScreen(
          referenceDate: PracticeSessionView.previewReferenceDate, suggestionDismissed: true),
        store: .previewPracticeSuggestion), as: config)
  }

  func testPracticeScreenPriorities() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPracticePriorities), as: config)
  }

  func testPracticeScreenSuggestionPriorities() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPracticeSuggestionPriorities), as: config)
  }

  /// Dismissed with something starred too: two secondaries (restore, then
  /// priorities) stack under the plain hero (#1618).
  func testPracticeScreenSuggestionDismissedPriorities() {
    assertSnapshot(
      of: host(
        PracticeScreen(
          referenceDate: PracticeSessionView.previewReferenceDate, suggestionDismissed: true),
        store: .previewPracticeSuggestionPriorities), as: config)
  }

  /// No star, no ladder variation, never-marked wording: conditionals that can
  /// regress without the full screen moving.
  func testUpNextHeroNeverMarked() {
    assertSnapshot(
      of: host(upNextHeroCard(.previewFresh)),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 370),
        traits: .init(displayScale: 2)))
  }

  private func upNextHeroCard(_ suggestion: SuggestedSession) -> some View {
    UpNextHero(suggestion: suggestion, onStart: {}, onBuildOwn: {})
      .padding(IntradaSpacing.card)
      .background(IntradaColor.paperTop)
      .frame(width: 390)
  }

  func testRecoveryPromptCard() throws {
    // Component-level (not full-screen): the card is the load-bearing state and
    // the flat crop keeps the reference PNG well under the size ceiling (#840).
    let session = try XCTUnwrap(Store.previewPracticeRecovery.recoverableSession)
    let card = RecoveryPromptCard(
      session: session, referenceDate: PracticeSessionView.previewReferenceDate,
      onResume: {}, onDiscard: {}
    )
    .padding(IntradaSpacing.card)
    .background(IntradaColor.paperTop)
    .frame(width: 390)
    assertSnapshot(
      of: host(card),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 240),
        traits: .init(displayScale: 2)))
  }

  /// The card's "what was played" line: pieces named plainly, an exercise's
  /// keys spelled out when there are few (#1785).
  func testSessionCardsPlayedSummary() {
    let cards = VStack(spacing: IntradaSpacing.card) {
      SessionCard(session: .previewCompleted)
      SessionCard(session: .previewWithVariations)
      SessionCard(session: .previewEndedEarly)
    }
    .padding(IntradaSpacing.card)
    .background(IntradaColor.paperTop)
    .frame(width: 390)
    assertSnapshot(
      of: host(cards),
      as: .image(
        perceptualPrecision: 0.98, size: CGSize(width: 390, height: 460),
        traits: .init(displayScale: 2)))
  }

  /// The real `PracticeScreen`/`TabView` path, not a bare component (#1730).
  func testPracticeScreenWeekStripAccessibilitySize() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPractice), as: tallAxConfig())
  }

  func testPracticeScreenQuietDay() {
    // Open on Monday, a day with no practice, to lock the per-day empty state.
    assertSnapshot(
      of: host(
        PracticeScreen(
          referenceDate: PracticeSessionView.previewReferenceDate,
          selectedDay: PracticeWeekView.previewWeek.days[0].date),
        store: .previewPractice), as: config)
  }

  /// The read-only record of a past session (#1371), including the session
  /// mark #1012 asked for.
  func testPracticeSessionDetail() {
    assertSnapshot(
      of: host(
        NavigationStack {
          PracticeSessionDetailScreen(session: .previewCompleted)
        }, store: .previewPractice), as: config)
  }

  /// Skipped and not-played items, and no session mark to draw.
  func testPracticeSessionDetailEndedEarly() {
    assertSnapshot(
      of: host(
        NavigationStack {
          PracticeSessionDetailScreen(session: .previewEndedEarly)
        }, store: .previewPractice), as: config)
  }

  /// A line per variation below the entry line, rather than only the last
  /// play's numbers (#1739).
  func testPracticeSessionDetailWithVariations() {
    assertSnapshot(
      of: host(
        NavigationStack {
          PracticeSessionDetailScreen(session: .previewWithVariations)
        }, store: .previewPractice), as: config)
  }

  /// A single variation still gets named, highlighted rather than folded
  /// into the "type and time" line (#1785).
  func testPracticeSessionDetailWithOneVariation() {
    assertSnapshot(
      of: host(
        NavigationStack {
          PracticeSessionDetailScreen(session: .previewWithOneVariation)
        }, store: .previewPractice), as: config)
  }

  /// Pins the accessibility-size branch: the ring drops below the text so the
  /// meta line cannot break mid-word (#1471's shape, one screen over).
  func testPracticeSessionDetailAccessibilitySize() {
    let detail = NavigationStack {
      PracticeSessionDetailScreen(session: .previewCompleted)
    }
    .dynamicTypeSize(.accessibility3)
    assertSnapshot(of: host(detail, store: .previewPractice), as: config)
  }

  /// The greeting leads the subtitle and the badge is the way in (#1694, T25).
  func testPracticeScreenGreeting() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPracticeProfile), as: config)
  }
}
