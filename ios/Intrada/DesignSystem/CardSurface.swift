import SwiftUI

extension View {
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
