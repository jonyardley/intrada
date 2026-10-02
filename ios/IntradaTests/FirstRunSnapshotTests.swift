import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

/// The welcome's three steps (#2117) and the Start here card (#2118).
final class FirstRunSnapshotTests: SnapshotTestCase {
  func testWelcome() {
    assertSnapshot(of: host(FirstRunScreen { _ in }), as: config)
  }

  func testWelcomeAccessibilitySize() {
    assertSnapshot(of: host(FirstRunScreen { _ in }), as: axConfig)
  }

  /// A phone's width in the middle of the iPad, not stretched across it.
  func testWelcomeOnIPad() {
    assertSnapshot(of: host(FirstRunScreen { _ in }), as: splitConfig)
  }

  /// The splash with its swipe drawn, before the handoff (#2277).
  func testLaunchSplash() {
    assertSnapshot(of: host(FirstRunScreen(intro: .frozen(at: 2.8)) { _ in }), as: config)
  }

  /// The returning splash settled, with its swipe still drawing (#2278).
  func testReturningSplash() {
    assertSnapshot(
      of: host(ReturningSplashLayer(frame: .returning(at: 0.4, reduceMotion: false))), as: config)
  }

  func testProfileStep() {
    assertSnapshot(of: host(FirstRunScreen(step: .profile) { _ in }), as: config)
  }

  func testProfileStepAccessibilitySize() {
    assertSnapshot(of: host(FirstRunScreen(step: .profile) { _ in }), as: axConfig)
  }

  func testFirstPieceStep() {
    assertSnapshot(of: host(FirstRunScreen(step: .firstPiece) { _ in }), as: config)
  }

  func testFirstPieceStepAccessibilitySize() {
    assertSnapshot(of: host(FirstRunScreen(step: .firstPiece) { _ in }), as: axConfig)
  }

  func testPracticeScreenStartHere() {
    assertSnapshot(
      of: host(
        PracticeScreen(referenceDate: PracticeSessionView.previewReferenceDate),
        store: .previewStartHere), as: config)
  }

  func testStartHereCardAccessibilitySize() {
    let card = StartHereCard(
      progress: FirstRunView(
        showsWelcome: false, showsStartHere: true, added: true, built: true, played: false,
        marked: false, firstItemTitle: "Clair de Lune"),
      onAdd: {}, onStartSession: {}
    )
    .padding(IntradaSpacing.card)
    assertSnapshot(of: host(card), as: tallAxConfig(height: 900))
  }
}
