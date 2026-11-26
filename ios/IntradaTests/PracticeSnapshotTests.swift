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

  // A day still to come reads "Yet to come", not a rest day (#2144).
  func testPracticeScreenDayStillToCome() {
    let week = PracticeWeekView.previewMidWeek
    assertSnapshot(
      of: host(
        PracticeScreen(
          referenceDate: PracticeSessionView.previewReferenceDate,
          selectedDay: week.days[5].date),
        store: Store(bridge: PreviewBridge(practiceWeeks: [week]))), as: config)
  }

  /// No item marked for the warm-up yet: the card that leads to the Library (#3007).
  func testPracticeScreenPlanEmptyWarmUp() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPractice(plan: .previewEmptyWarmUp)), as: config)
  }

  /// Profile turned the warm-up off, so the section goes altogether (#3007).
  func testPracticeScreenPlanWarmUpOff() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPractice(plan: .previewWarmUpOff)), as: config)
  }

  /// The stepper, the rows and the start bar at the largest text we allow (#1411).
  func testPracticeScreenPlanLargestText() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPractice),
      as: .image(
        on: .iPhone13, perceptualPrecision: 0.98,
        traits: UITraitCollection { traits in
          traits.displayScale = 2
          traits.preferredContentSizeCategory = .extraExtraExtraLarge
        }))
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

  /// The greeting leads the subtitle and the badge is the way in (#1694, T25).
  func testPracticeScreenGreeting() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewPracticeProfile), as: config)
  }
}
