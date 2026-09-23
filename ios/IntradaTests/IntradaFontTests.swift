import Testing
import UIKit

@testable import Intrada

/// An unresolved PostScript name falls back to the system font silently (#1676).
struct IntradaFontTests {
  @Test(arguments: [
    IntradaFont.Hanken.regular, IntradaFont.Hanken.medium, IntradaFont.Hanken.semibold,
    IntradaFont.Hanken.bold, IntradaFont.Mono.regular,
  ])
  func bundledFaceResolves(_ name: String) {
    IntradaFonts.register()
    #expect(UIFont(name: name, size: 16) != nil)
  }

  @Test(arguments: [
    (IntradaFont.Hanken.regular, UIFont.Weight.regular.rawValue),
    (IntradaFont.Hanken.medium, UIFont.Weight.medium.rawValue),
    (IntradaFont.Hanken.semibold, UIFont.Weight.semibold.rawValue),
    (IntradaFont.Hanken.bold, UIFont.Weight.bold.rawValue),
  ])
  func faceCarriesItsWeight(_ name: String, weight: CGFloat) throws {
    IntradaFonts.register()
    let font = try #require(UIFont(name: name, size: 16))
    let traits = font.fontDescriptor.object(forKey: .traits) as? [UIFontDescriptor.TraitKey: Any]
    let actual = try #require(traits?[.weight] as? CGFloat)
    let nearest = Self.namedWeights.min { abs($0 - actual) < abs($1 - actual) }
    #expect(nearest == weight)
  }

  /// A bundled face reports its OS/2 class, which sits near a named weight
  /// rather than on it.
  private static let namedWeights = [
    UIFont.Weight.light, .regular, .medium, .semibold, .bold, .heavy,
  ].map(\.rawValue)
}
