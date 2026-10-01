import SharedTypes
import SwiftUI

private struct MarkerKey: EnvironmentKey {
  static let defaultValue: Color = IntradaColor.marker
}

private struct HeroGradientKey: EnvironmentKey {
  static let defaultValue: LinearGradient = .practiceHero(.butter)
}

extension EnvironmentValues {
  /// The highlighter the musician chose (#1677), set once at the root from the
  /// core's profile view so every marker surface follows the swatch.
  var marker: Color {
    get { self[MarkerKey.self] }
    set { self[MarkerKey.self] = newValue }
  }

  /// The Practice hero's ground, deepened from the same swatch (#1877).
  var heroGradient: LinearGradient {
    get { self[HeroGradientKey.self] }
    set { self[HeroGradientKey.self] = newValue }
  }
}

extension View {
  func marker(_ colour: HighlighterColour) -> some View {
    environment(\.marker, IntradaColor.marker(colour))
      .environment(\.heroGradient, .practiceHero(colour))
  }
}
