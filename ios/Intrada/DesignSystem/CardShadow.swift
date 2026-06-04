import SwiftUI

extension View {
  /// The app's single soft elevation recipe (previously inline in KeyPicker).
  /// Lifts a card or layer off the paper so stacked surfaces read as separate.
  /// `above: true` casts the shadow upward — for a bottom panel the content
  /// slides *under*, e.g. the builder's queue tray.
  func cardShadow(above: Bool = false) -> some View {
    shadow(color: IntradaColor.shadow, radius: 5, x: 0, y: above ? -2 : 2)
  }
}
