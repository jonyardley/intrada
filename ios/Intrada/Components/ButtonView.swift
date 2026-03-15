import SwiftUI

/// Button variant matching the web's `ButtonVariant` enum.
enum ButtonVariant {
    case primary
    case secondary
    case danger
    case dangerOutline
}

/// Styled button matching the web's `Button` component.
///
/// Supports Primary, Secondary, Danger, and DangerOutline variants
/// with loading spinner and disabled state. Minimum 44pt touch target.
///
///     ButtonView("Save", variant: .primary) { save() }
///     ButtonView("Delete", variant: .danger, loading: true) { }
struct ButtonView: View {

    let label: String
    var variant: ButtonVariant = .primary
    var disabled: Bool = false
    var loading: Bool = false
    let action: () -> Void

    init(
        _ label: String,
        variant: ButtonVariant = .primary,
        disabled: Bool = false,
        loading: Bool = false,
        action: @escaping () -> Void
    ) {
        self.label = label
        self.variant = variant
        self.disabled = disabled
        self.loading = loading
        self.action = action
    }

    private var backgroundColor: Color {
        switch variant {
        case .primary: Color.accent
        case .secondary: Color.surfaceSecondary
        case .danger: Color.danger
        case .dangerOutline: Color.dangerSurface
        }
    }

    private var foregroundColor: Color {
        switch variant {
        case .primary: Color.textPrimary
        case .secondary: Color.textSecondary
        case .danger: Color.textPrimary
        case .dangerOutline: Color.dangerText
        }
    }

    private var borderColor: Color? {
        switch variant {
        case .secondary: Color.borderDefault
        case .dangerOutline: Color.dangerText.opacity(0.3)
        default: nil
        }
    }

    var body: some View {
        Button(action: action) {
            HStack(spacing: 8) {
                if loading {
                    ProgressView()
                        .tint(foregroundColor)
                        .controlSize(.small)
                }
                Text(label)
                    .font(.system(size: 14, weight: .semibold))
            }
            .foregroundStyle(foregroundColor)
            .frame(maxWidth: .infinity)
            .frame(minHeight: 44)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: DesignRadius.button))
            .overlay {
                if let borderColor {
                    RoundedRectangle(cornerRadius: DesignRadius.button)
                        .stroke(borderColor, lineWidth: 1)
                }
            }
        }
        .disabled(disabled || loading)
        .opacity(disabled ? 0.5 : 1.0)
    }
}

#Preview("ButtonView") {
    VStack(spacing: 12) {
        ButtonView("Primary Action", variant: .primary) { }
        ButtonView("Secondary Action", variant: .secondary) { }
        ButtonView("Danger Action", variant: .danger) { }
        ButtonView("Danger Outline", variant: .dangerOutline) { }
        ButtonView("Loading...", variant: .primary, loading: true) { }
        ButtonView("Disabled", variant: .primary, disabled: true) { }
    }
    .padding()
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}
