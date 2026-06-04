import SwiftUI

extension View {
  /// The app's single soft elevation recipe (previously inline in KeyPicker).
  /// Lifts a card or layer off the paper so stacked surfaces read as separate.
  func cardShadow() -> some View {
    shadow(color: IntradaColor.shadow, radius: 5, x: 0, y: 2)
  }
}
