import SwiftUI

extension View {
  /// A label that sits over the user's own photo or a live camera feed, where
  /// there is no background token to rely on. At 0.8 the worst case (a white
  /// page) is 8:1; 0.55 was 3.4:1, under the AA floor.
  func scrimCapsule() -> some View {
    font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.onAccent)
      .padding(.horizontal, IntradaSpacing.row)
      .padding(.vertical, IntradaSpacing.cardCompact)
      .background(IntradaColor.viewerBackdrop.opacity(0.8), in: Capsule())
      .contentShape(Capsule())
  }
}
