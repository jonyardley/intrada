import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// Growing to its content's width is what let ancestors centre the scaffold and
/// clip both edges (#1470). Two-sided: the background it leans on must fill too.
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
}
