import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// `ScreenScaffold` used to report its *content's* width rather than the width
/// it was offered, so one un-shrinkable row made the whole screen over-large
/// and every ancestor then centred it — which is how the Library lost the first
/// characters of its title and every card at accessibility sizes (#1470).
///
/// Asserted as a measurement, not by walking the view hierarchy: any wrapper
/// placed around an over-wide view centres it, so a traversal reports an offset
/// the view under test is not responsible for. A plain leading `VStack` scores
/// identically to a broken scaffold under that method, which is why #1481 could
/// not be closed with it.
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

  /// The baseline the test above is meaningless without: an offered width that
  /// the content fits inside must come back unchanged.
  @Test("The scaffold reports the offered width for content that fits")
  func scaffoldReportsOfferedWidthWhenContentFits() {
    let scaffold = ScreenScaffold(title: "Library", subtitle: "2 pieces") {
      Color.clear.frame(height: 200)
    }
    #expect(abs(reportedWidth(of: scaffold) - Self.offered.width) <= 1)
  }
}
