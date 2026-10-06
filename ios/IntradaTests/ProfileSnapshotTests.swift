import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class ProfileSnapshotTests: SnapshotTestCase {
  /// Every instrument icon at its three sizes (#1693): a missing or empty
  /// asset renders blank here rather than on a device.
  func testInstrumentIcons() {
    let sheet = VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      ForEach(InstrumentIcon.all, id: \.self) { icon in
        HStack(spacing: IntradaSpacing.card) {
          InstrumentGlyph(icon: icon, size: IntradaGlyph.bar)
          InstrumentGlyph(icon: icon, size: IntradaGlyph.tile)
          InstrumentGlyph(icon: icon, size: IntradaGlyph.hero)
          Text(icon.tileLabel).font(IntradaFont.body)
        }
      }
    }
    .foregroundStyle(IntradaColor.ink)
    .padding(IntradaSpacing.card)
    assertSnapshot(of: host(sheet), as: tallFormConfig)
  }

  func testProfileScreen() {
    assertSnapshot(
      of: host(NavigationStack { ProfileScreen() }, store: .previewProfile), as: config)
  }

  /// No name yet: a prompt to add one, not blank rows (#1692).
  func testProfileScreenEmpty() {
    assertSnapshot(of: host(NavigationStack { ProfileScreen() }), as: config)
  }

  func testProfileEditSheet() {
    assertSnapshot(of: host(ProfileEditSheet(), store: .previewProfile), as: config)
  }

  func testProfileEditSheetWithError() {
    assertSnapshot(
      of: host(
        ProfileEditSheet(
          previewError: "Name must be 100 characters or fewer", field: .name),
        store: .previewProfile),
      as: config)
  }

  /// The Beta row sits below the fold of the phone-height snapshots.
  func testProfileScreenFull() {
    assertSnapshot(
      of: host(NavigationStack { ProfileScreen() }, store: .previewProfile), as: tallFormConfig)
  }

  func testFeedbackSheet() {
    assertSnapshot(of: host(FeedbackSheet()), as: config)
  }

  func testFeedbackSheetFromAShake() {
    assertSnapshot(of: host(FeedbackSheet(screenshot: Self.screenshot)), as: config)
  }

  private static var screenshot: Data? {
    UIGraphicsImageRenderer(size: CGSize(width: 390, height: 844)).image { context in
      UIColor(IntradaColor.paperTop).setFill()
      context.fill(CGRect(x: 0, y: 0, width: 390, height: 844))
      UIColor(IntradaColor.ink).setFill()
      context.fill(CGRect(x: 24, y: 120, width: 342, height: 64))
    }.pngData()
  }

  func testInstrumentIconPicker() {
    assertSnapshot(
      of: host(InstrumentIconPicker(suggested: .cello, choice: .constant(.harp))), as: config)
  }

  /// The eight highlighters through the environment (#1677): a marker surface
  /// wears whichever swatch the root sets, not the butter token.
  func testHighlighterColours() {
    let colours: [HighlighterColour] = [
      .butter, .coral, .mint, .sky, .lavender, .sage, .peach, .powder,
    ]
    let sheet = VStack(spacing: IntradaSpacing.cardCompact) {
      ForEach(colours, id: \.self) { colour in
        BrandBarButton(action: {}) { Text(String(describing: colour)) }
          .environment(\.marker, IntradaColor.marker(colour))
      }
    }
    .padding(IntradaSpacing.card)
    assertSnapshot(of: host(sheet), as: config)
  }
}
