import SwiftUI

extension View {
  /// The feathered highlighter under a page title (#1676), drawn left to right
  /// up to `progress`.
  func markerSwipe(progress: CGFloat = 1) -> some View {
    modifier(MarkerSwipe(progress: progress))
  }
}

private struct MarkerSwipe: ViewModifier {
  let progress: CGFloat
  @Environment(\.marker) private var marker

  func body(content: Content) -> some View {
    content.background {
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .fill(
          LinearGradient(
            stops: [
              .init(color: .clear, location: 0.50),
              .init(color: marker.opacity(IntradaOpacity.soft), location: 0.53),
              .init(color: marker, location: 0.58),
              .init(color: marker, location: 0.89),
              .init(color: marker.opacity(IntradaOpacity.soft), location: 0.93),
              .init(color: .clear, location: 0.96),
            ],
            startPoint: .top, endPoint: .bottom
          )
        )
        .scaleEffect(x: progress, anchor: .leading)
    }
  }
}
