import SwiftUI

// MARK: - Form Input Style Modifier
//
// Matches the web's `input-base`, `input-error`, and `input-success` utilities.
// Provides consistent styling for TextField and TextEditor.

struct InputStyleModifier: ViewModifier {

    var hasError: Bool = false

    func body(content: Content) -> some View {
        content
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
            .background(Color.surfaceInput)
            .clipShape(RoundedRectangle(cornerRadius: DesignRadius.input))
            .overlay(
                RoundedRectangle(cornerRadius: DesignRadius.input)
                    .stroke(
                        hasError ? Color.dangerText : Color.borderInput,
                        lineWidth: 1
                    )
            )
            .foregroundStyle(Color.textPrimary)
            .font(.system(size: 14))
    }
}

// MARK: - View Extension

extension View {
    /// Apply form input styling.
    ///
    /// - Parameter hasError: When true, shows danger-coloured border.
    func inputStyle(hasError: Bool = false) -> some View {
        modifier(InputStyleModifier(hasError: hasError))
    }
}

// MARK: - Preview

#Preview("Input Style") {
    VStack(spacing: 16) {
        TextField("Normal input", text: .constant(""))
            .inputStyle()

        TextField("With text", text: .constant("Clair de Lune"))
            .inputStyle()

        TextField("Error state", text: .constant(""))
            .inputStyle(hasError: true)
    }
    .padding()
    .background(Color.backgroundApp)
}
