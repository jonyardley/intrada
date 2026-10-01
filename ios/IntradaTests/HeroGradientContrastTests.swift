import SharedTypes
import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// The hero's lower stop follows the marker (#1877); every swatch must keep the
/// light text and the marker button readable on its lightest corner.
@MainActor
struct HeroGradientContrastTests {
  @Test func titleHoldsLargeTextContrast() {
    for colour in HighlighterColour.all {
      let ground = rgb(IntradaColor.heroGradientBottom(colour))
      #expect(contrast(rgb(IntradaColor.paperTop), ground) >= 3, "\(colour.label)")
    }
  }

  /// The reason and meta lines sit at 75% white on the 12% paper wash of the
  /// item card: the tightest pairing on either hero.
  @Test func smallTextOnTheItemCardHoldsBodyContrast() {
    for colour in HighlighterColour.all {
      let card = over(
        rgb(IntradaColor.paperTop), IntradaOpacity.wash,
        rgb(IntradaColor.heroGradientBottom(colour)))
      let text = over(rgb(IntradaColor.onAccent), IntradaOpacity.secondary, card)
      #expect(contrast(text, card) >= 4.5, "\(colour.label)")
    }
  }

  @Test func startButtonStandsOffTheGround() {
    for colour in HighlighterColour.all {
      let ground = rgb(IntradaColor.heroGradientBottom(colour))
      #expect(contrast(rgb(IntradaColor.marker(colour)), ground) >= 3, "\(colour.label)")
    }
  }

  @Test func eachSwatchGetsItsOwnStop() {
    let stops = HighlighterColour.all.map { rgb(IntradaColor.heroGradientBottom($0)) }
    #expect(Set(stops.map { "\($0)" }).count == HighlighterColour.all.count)
  }

  // ── Contrast (WCAG 2.x relative luminance) ──

  private typealias RGB = [Double]

  private func rgb(_ color: Color) -> RGB {
    var r: CGFloat = 0
    var g: CGFloat = 0
    var b: CGFloat = 0
    var a: CGFloat = 0
    UIColor(color).getRed(&r, green: &g, blue: &b, alpha: &a)
    return [r, g, b].map(Double.init)
  }

  private func over(_ top: RGB, _ alpha: Double, _ base: RGB) -> RGB {
    zip(top, base).map { $0 * alpha + $1 * (1 - alpha) }
  }

  private func luminance(_ c: RGB) -> Double {
    let linear = c.map { $0 <= 0.03928 ? $0 / 12.92 : pow(($0 + 0.055) / 1.055, 2.4) }
    return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]
  }

  private func contrast(_ a: RGB, _ b: RGB) -> Double {
    let (hi, lo) = (max(luminance(a), luminance(b)), min(luminance(a), luminance(b)))
    return (hi + 0.05) / (lo + 0.05)
  }
}
