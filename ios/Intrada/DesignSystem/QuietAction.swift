import SwiftUI

/// The declinable second action under a primary one. Quiet by weight, never by
/// being hard to find — every resolution screen has an honest way out.
struct QuietAction: ButtonStyle {
  func makeBody(configuration: Configuration) -> some View {
    configuration.label
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.inkSecondary)
      .frame(maxWidth: .infinity, minHeight: 50)
      .contentShape(Rectangle())
      .opacity(configuration.isPressed ? 0.6 : 1)
  }
}
