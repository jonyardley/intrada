import SwiftUI

extension View {
  /// A soft shadow pinned at a scroll view's top edge, so content slides
  /// *under* the header/filters above it with depth instead of hard-cutting at
  /// the boundary. Apply to the `ScrollView`.
  func scrollEdgeShadow(height: CGFloat = 5) -> some View {
    overlay(alignment: .top) {
      LinearGradient(colors: [IntradaColor.shadow, .clear], startPoint: .top, endPoint: .bottom)
        .frame(height: height)
        .allowsHitTesting(false)
    }
  }
}
