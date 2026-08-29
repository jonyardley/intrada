import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// The scaffold sizes every top-level screen, so it must report the width it
/// was offered rather than its content's: growing is what let ancestors centre
/// it and clip both edges (#1470). `ScreenEdgeTests` covers the symptom; this
/// covers the cause, and that the background it now leans on fills the offer.
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
      abs(width - Self.offered.width) <= 1,
      "the scaffold reported \(width)pt against a \(Self.offered.width)pt offer"
    )
  }

  /// The background is now the scaffold's sole sizing anchor, so a background
  /// that stopped filling the offer would shrink every screen.
  @Test("The background fills the offered width")
  func backgroundFillsTheOfferedWidth() {
    let scaffold = ScreenScaffold(title: "Library", subtitle: "2 pieces") {
      Color.clear.frame(height: 200)
    }
    #expect(abs(reportedWidth(of: scaffold) - Self.offered.width) <= 1)
  }
}
