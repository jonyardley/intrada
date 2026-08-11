import SwiftUI

/// The flat key `TapVerdict` and `FeelChips` are both cut from, in two columns
/// and three. `CoachAction`'s `.key` is the seated variant of the same shape.
private struct CoachKeySurface: ViewModifier {
  let height: CGFloat
  let fill: Color
  let border: Color

  @Environment(\.coachScale) private var scale

  func body(content: Content) -> some View {
    let shape = RoundedRectangle(cornerRadius: scale.targetRadius, style: .continuous)
    return
      content
      .frame(maxWidth: .infinity)
      .frame(height: height)
      .padding(.horizontal, IntradaSpacing.controlGap)
      .background(fill, in: shape)
      .overlay(shape.strokeBorder(border, lineWidth: 1))
  }
}

extension View {
  func coachKeySurface(height: CGFloat, fill: Color, border: Color) -> some View {
    modifier(CoachKeySurface(height: height, fill: fill, border: border))
  }
}
