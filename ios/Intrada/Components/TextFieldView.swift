import SwiftUI

/// Form input with label, hint, and error matching the web's `TextField` component.
///
///     TextFieldView(
///         label: "Title",
///         text: $title,
///         placeholder: "e.g. Clair de Lune"
///     )
///
///     TextFieldView(
///         label: "Title",
///         text: $title,
///         hint: "The name of the piece or exercise",
///         error: titleError
///     )
struct TextFieldView: View {

    let label: String
    @Binding var text: String
    var placeholder: String = ""
    var hint: String? = nil
    var error: String? = nil

    private var hasError: Bool { error != nil && !(error?.isEmpty ?? true) }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            // Label
            Text(label)
                .formLabelStyle()

            // Hint
            if let hint {
                Text(hint)
                    .hintTextStyle()
            }

            // Input
            TextField(placeholder, text: $text)
                .inputStyle(hasError: hasError)

            // Error
            FormFieldError(message: error)
        }
    }
}

#Preview("TextFieldView") {
    VStack(spacing: 20) {
        TextFieldView(
            label: "Title",
            text: .constant(""),
            placeholder: "e.g. Clair de Lune"
        )

        TextFieldView(
            label: "Title",
            text: .constant("Clair de Lune"),
            hint: "The name of the piece"
        )

        TextFieldView(
            label: "Title",
            text: .constant(""),
            error: "Title is required"
        )

        TextFieldView(
            label: "Composer",
            text: .constant(""),
            hint: "Optional — the composer or author",
            placeholder: "e.g. Debussy"
        )
    }
    .padding()
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}
