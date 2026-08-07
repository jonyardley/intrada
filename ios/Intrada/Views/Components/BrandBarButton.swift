import SwiftUI

/// The brand-gradient primary action bar (builder start, reflection save,
/// practise-this). Generic label content so icons can lead or trail.
struct BrandBarButton<Label: View>: View {
  private let action: () -> Void
  private let prominent: Bool
  private let label: Label

  /// `prominent` is the one-action-per-screen weight the built-session flow
  /// uses (#1256): taller, rounder, and it rebounds under the thumb. The
  /// default is the inline bar every other surface already draws.
  init(prominent: Bool = false, action: @escaping () -> Void, @ViewBuilder label: () -> Label) {
    self.prominent = prominent
    self.action = action
    self.label = label()
  }

  @ViewBuilder var body: some View {
    if prominent {
      Button(action: action) { bar }.buttonStyle(PressRebound(scale: 0.97))
    } else {
      Button(action: action) { bar }.buttonStyle(.plain)
    }
  }

  private var bar: some View {
    HStack(spacing: IntradaSpacing.controlGap) { label }
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.onAccent)
      .frame(maxWidth: .infinity, minHeight: prominent ? 56 : nil)
      .padding(.vertical, prominent ? 0 : IntradaSpacing.row)
      .background(
        LinearGradient.brandBar,
        in: RoundedRectangle(
          cornerRadius: prominent ? IntradaRadius.panel : IntradaRadius.control))
  }
}
