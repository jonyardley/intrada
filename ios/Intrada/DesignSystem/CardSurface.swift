import SwiftUI

extension View {
  /// The standard cream card surface: fill, rounded corners, hairline stroke.
  func cardSurface(cornerRadius: CGFloat = 12) -> some View {
    self
      .background(IntradaColor.cardFill)
      .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
      .overlay(
        RoundedRectangle(cornerRadius: cornerRadius)
          .stroke(IntradaColor.hairline, lineWidth: 1)
      )
  }
}
