import SwiftUI

/// Toast notification overlay matching the web's `Toast` component.
///
/// Reads from `ToastManager` in the environment. Apply via the
/// `.toastOverlay()` modifier at the app root.
///
///     ContentView()
///         .toastOverlay()
///         .environment(toastManager)
struct ToastView: View {

    let message: String
    let variant: ToastVariant
    let onDismiss: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: variant.iconName)
                .font(.system(size: 18))
                .foregroundStyle(variant.textColor)

            Text(message)
                .font(.system(size: 14))
                .foregroundStyle(variant.textColor)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(Spacing.card)
        .background(variant.backgroundColor)
        .overlay(
            Rectangle()
                .fill(variant.borderColor)
                .frame(width: 4),
            alignment: .leading
        )
        .clipShape(RoundedRectangle(cornerRadius: DesignRadius.badge))
        .shadow(color: .shadowDefault, radius: 8, y: 4)
        .accessibilityElement(children: .combine)
        .accessibilityLabel(message)
        .accessibilityAddTraits(.isStaticText)
    }
}

/// ViewModifier that overlays the toast at the top of the view.
struct ToastOverlayModifier: ViewModifier {

    @Environment(ToastManager.self) private var toast

    func body(content: Content) -> some View {
        content
            .overlay(alignment: .top) {
                if toast.isShowing {
                    ToastView(
                        message: toast.message,
                        variant: toast.variant,
                        onDismiss: { toast.dismiss() }
                    )
                    .padding(.horizontal, Spacing.card)
                    .padding(.top, 8)
                    .transition(.move(edge: .top).combined(with: .opacity))
                    .onTapGesture { toast.dismiss() }
                }
            }
    }
}

extension View {
    /// Attach the toast overlay to this view.
    /// Requires `ToastManager` in the environment.
    func toastOverlay() -> some View {
        modifier(ToastOverlayModifier())
    }
}

// MARK: - Preview

#Preview("Toast Variants") {
    VStack(spacing: 16) {
        ToastView(message: "Session saved successfully", variant: .success, onDismiss: {})
        ToastView(message: "Check your connection", variant: .warning, onDismiss: {})
        ToastView(message: "Failed to save session", variant: .danger, onDismiss: {})
        ToastView(message: "New update available", variant: .info, onDismiss: {})
    }
    .padding()
    .background(Color.backgroundApp)
}
