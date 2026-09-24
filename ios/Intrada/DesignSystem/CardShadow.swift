import SwiftUI

extension View {
  func dropShadow(_ token: IntradaShadow) -> some View {
    shadow(color: token.color, radius: token.radius, y: token.y)
  }

  /// Lifts a card or layer off the paper so stacked surfaces read as separate.
  func cardShadow() -> some View {
    dropShadow(.card)
  }
}
