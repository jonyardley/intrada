import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class ComponentSnapshotTests: SnapshotTestCase {
  func testScoreRing() {
    let rings = ZStack {
      PaperBackground()
      HStack(spacing: 18) {
        ScoreRing(score: nil)
        ForEach([1, 4, 7, 10], id: \.self) { ScoreRing(score: $0) }
      }
      .padding(16)
    }
    assertSnapshot(of: host(rings), as: config)
  }

  func testScoreRingHero() {
    let hero = ZStack {
      PaperBackground()
      HStack(spacing: 24) {
        ScoreRing(score: 7, size: 132, showsScale: true)
        ScoreRing(score: nil, size: 132, showsScale: true)
      }
      .padding(16)
    }
    assertSnapshot(of: host(hero), as: config)
  }

  func testAddRowButtonVariants() {
    let buttons = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        AddRowButton(title: "Add a related exercise") {}
        AddRowButton(title: "Add a related exercise", style: .plain) {}
      }
      .padding(16)
    }
    assertSnapshot(of: host(buttons), as: config)
  }

  // The eyebrow must wrap between words beside the Edit button, never inside one (#1781).
  func testSectionHeaderWithActionAccessibilitySize() {
    let headers = ZStack {
      PaperBackground()
      VStack(alignment: .leading, spacing: IntradaSpacing.section) {
        SectionHeader(
          title: "Chord chart",
          action: .init(title: "Edit", accessibilityLabel: "Edit chord chart", perform: {}))
        SectionHeader(
          title: "Related exercises", caption: "3", captionAccessibilityHidden: true,
          action: .init(title: "Edit", accessibilityLabel: "Edit related exercises", perform: {}))
      }
      .padding(IntradaSpacing.card)
    }
    .dynamicTypeSize(.accessibility5)
    assertSnapshot(of: host(headers), as: config)
  }
}
