import SwiftUI

/// Inline validation error text below a form field.
/// Matches the web's `FormFieldError` component.
///
///     FormFieldError(message: "Title is required")
struct FormFieldError: View {

    let message: String?

    var body: some View {
        if let message, !message.isEmpty {
            Text(message)
                .font(.caption)
                .foregroundStyle(Color.dangerText)
                .frame(maxWidth: .infinity, alignment: .leading)
                .accessibilityLabel("Error: \(message)")
        }
    }
}

#Preview("FormFieldError") {
    VStack(alignment: .leading, spacing: 16) {
        FormFieldError(message: "Title is required")
        FormFieldError(message: "Must be at least 2 characters")
        FormFieldError(message: nil)
    }
    .padding()
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}
