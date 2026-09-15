import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// Bar visibility needs a real window: a windowless host lays out no nav bar (#1682).
@MainActor
struct TabRootNavigationBarTests {
  private static let phone = CGSize(width: 390, height: 844)
  private static let pad = CGSize(width: 1194, height: 834)

  private func barsHidden(
    _ rootView: some View, size: CGSize = phone, store: Store = .previewLibrary
  ) -> [Bool] {
    IntradaFonts.register()
    let vc = UIHostingController(
      rootView:
        rootView
        .environment(store)
        .environment(\.locale, Locale(identifier: "en_US"))
        .environment(\.calendar, PreviewCalendar.utc)
        .environment(\.intradaMotionDisabled, true))
    let window = UIWindow(frame: CGRect(origin: .zero, size: size))
    window.rootViewController = vc
    window.isHidden = false
    defer { window.rootViewController = nil }
    vc.view.layoutIfNeeded()

    var scope: UIViewController = vc
    if let tabs = descendants(of: vc).lazy.compactMap({ $0 as? UITabBarController }).first {
      scope = tabs.selectedViewController ?? tabs
    }
    return descendants(of: scope)
      .compactMap { $0 as? UINavigationController }
      .sorted {
        $0.view.convert(CGPoint.zero, to: nil).x < $1.view.convert(CGPoint.zero, to: nil).x
      }
      .map(\.isNavigationBarHidden)
  }

  private func descendants(of vc: UIViewController) -> [UIViewController] {
    [vc] + vc.children.flatMap { descendants(of: $0) }
  }

  @Test func libraryOpensWithNoBar() {
    #expect(barsHidden(RootView()) == [true])
  }

  @Test func practiceOpensWithNoBar() {
    #expect(barsHidden(RootView(previewTab: .practice)) == [true])
  }

  @Test func aPieceOpenedFromTheLibraryKeepsItsBar() {
    let pushed = NavigationStack(path: .constant(["piece-1"])) {
      LibraryScreen().navigationBarHiddenAtRoot()
    }
    #expect(barsHidden(pushed) == [false])
  }

  @Test func buildSessionOpenedFromPracticeKeepsItsBar() {
    #expect(barsHidden(RootView(previewTab: .practice), store: .previewBuilding) == [false])
  }

  @Test func bothIPadLibraryColumnsKeepTheirBars() {
    let split = LibrarySplitView(previewSelection: "piece-1")
      .environment(\.horizontalSizeClass, .regular)
    #expect(barsHidden(split, size: Self.pad) == [false, false])
  }
}
