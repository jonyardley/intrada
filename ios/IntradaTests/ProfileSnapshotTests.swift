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
