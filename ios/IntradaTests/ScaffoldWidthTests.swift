import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// Measured, not walked: any wrapper around an over-wide view centres it, so a
/// traversal scores a plain `VStack` like a broken scaffold (#1470, #1481).
@MainActor
struct ScaffoldWidthTests {
  private static let offered = CGSize(width: 390, height: 844)

  private func reportedWidth(of view: some View) -> CGFloat {
    IntradaFonts.register()
    let vc = UIHostingController(rootView: view.dynamicTypeSize(.accessibility5))
    return vc.sizeThatFits(in: Self.offered).width
  }

  @Test("The scaffold reports the width it was offered, not its content's")
  func scaffoldContainsAnOverWideChild() {
    let scaffold = ScreenScaffold(title: "Library", subtitle: "2 pieces") {
      Color.clear.frame(width: 900, height: 200)
    }
    let width = reportedWidth(of: scaffold)
    #expect(
      width <= Self.offered.width + 1,
      "the scaffold grew to \(width)pt against a \(Self.offered.width)pt offer, so an ancestor centres it and clips both edges"
    )
  }

  @Test("The scaffold reports the offered width for content that fits")
  func scaffoldReportsOfferedWidthWhenContentFits() {
    let scaffold = ScreenScaffold(title: "Library", subtitle: "2 pieces") {
      Color.clear.frame(height: 200)
    }
    #expect(abs(reportedWidth(of: scaffold) - Self.offered.width) <= 1)
  }
}
