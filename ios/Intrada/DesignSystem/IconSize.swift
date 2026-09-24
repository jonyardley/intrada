import SwiftUI

extension View {
  func iconSize(_ size: IntradaIconSize, weight: Font.Weight = .regular) -> some View {
    modifier(IconSize(size, weight: weight))
  }
}

private struct IconSize: ViewModifier {
  @ScaledMetric private var points: CGFloat
  let maxPoints: CGFloat
  let weight: Font.Weight

  init(_ size: IntradaIconSize, weight: Font.Weight) {
    _points = ScaledMetric(wrappedValue: size.points, relativeTo: size.textStyle)
    maxPoints = size.maxPoints
    self.weight = weight
  }

  func body(content: Content) -> some View {
    content.font(.system(size: min(points, maxPoints), weight: weight))
  }
}
