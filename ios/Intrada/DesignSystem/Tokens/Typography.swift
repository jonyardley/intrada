import SwiftUI

// MARK: - Design Tokens: Typography
//
// Font extensions and ViewModifiers matching the web's typography utilities.
// Heading font uses Georgia (closest built-in serif to Source Serif 4).
// All body text uses the system font (San Francisco).

extension Font {

    /// Serif heading font for page titles.
    /// Maps to the web's `font-heading` (Source Serif 4 → Georgia on iOS).
    static func heading(size: CGFloat = 28) -> Font {
        .custom("Georgia", size: size, relativeTo: .largeTitle)
    }
}

// MARK: - Typography View Modifiers

/// Section heading inside cards — web's `section-title` utility.
/// 18pt semibold, textPrimary.
struct SectionTitleStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.system(size: 18, weight: .semibold))
            .foregroundStyle(Color.textPrimary)
    }
}

/// Card subsection heading — web's `card-title` utility.
/// 14pt semibold, textSecondary.
struct CardTitleStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.system(size: 14, weight: .semibold))
            .foregroundStyle(Color.textSecondary)
    }
}

/// Data label / stat card title — web's `field-label` utility.
/// 12pt medium, uppercase, textMuted, wider tracking.
struct FieldLabelStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.system(size: 12, weight: .medium))
            .foregroundStyle(Color.textMuted)
            .textCase(.uppercase)
            .tracking(0.8)
    }
}

/// Form field label — web's `form-label` utility.
/// 14pt medium, textLabel.
struct FormLabelStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.system(size: 14, weight: .medium))
            .foregroundStyle(Color.textLabel)
    }
}

/// Helper text below fields — web's `hint-text` utility.
/// 12pt regular, textMuted.
struct HintTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.system(size: 12))
            .foregroundStyle(Color.textMuted)
    }
}

// MARK: - View Extension (convenience)

extension View {
    func sectionTitleStyle() -> some View { modifier(SectionTitleStyle()) }
    func cardTitleStyle() -> some View { modifier(CardTitleStyle()) }
    func fieldLabelStyle() -> some View { modifier(FieldLabelStyle()) }
    func formLabelStyle() -> some View { modifier(FormLabelStyle()) }
    func hintTextStyle() -> some View { modifier(HintTextStyle()) }
}

// MARK: - Preview

#Preview("Typography") {
    VStack(alignment: .leading, spacing: 16) {
        Text("Page Heading")
            .font(.heading())
            .foregroundStyle(Color.textPrimary)

        Text("Section Title")
            .sectionTitleStyle()

        Text("Card Title")
            .cardTitleStyle()

        Text("FIELD LABEL")
            .fieldLabelStyle()

        Text("Form Label")
            .formLabelStyle()

        Text("Hint text appears below form fields")
            .hintTextStyle()

        Text("Body text in textSecondary")
            .font(.body)
            .foregroundStyle(Color.textSecondary)

        Text("Muted caption text")
            .font(.caption)
            .foregroundStyle(Color.textMuted)

        Text("Faint timestamp")
            .font(.caption2)
            .foregroundStyle(Color.textFaint)
    }
    .padding()
    .frame(maxWidth: .infinity, alignment: .leading)
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}
