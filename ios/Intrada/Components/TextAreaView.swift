import SwiftUI

/// Multi-line form input matching the web's `TextArea` component.
///
///     TextAreaView(
///         label: "Notes",
///         text: $notes,
///         hint: "Practice observations"
///     )
struct TextAreaView: View {

    let label: String
    @Binding var text: String
    var placeholder: String = ""
    var hint: String? = nil
    var error: String? = nil
    var minHeight: CGFloat = 100

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

            // Text editor with placeholder overlay
            ZStack(alignment: .topLeading) {
                TextEditor(text: $text)
                    .scrollContentBackground(.hidden)
                    .frame(minHeight: minHeight)
                    .inputStyle(hasError: hasError)

                if text.isEmpty && !placeholder.isEmpty {
                    Text(placeholder)
                        .font(.system(size: 14))
                        .foregroundStyle(Color.textMuted)
                        .padding(.horizontal, 16)
                        .padding(.vertical, 14)
                        .allowsHitTesting(false)
                }
            }

            // Error
            FormFieldError(message: error)
        }
    }
}

#Preview("TextAreaView") {
    VStack(spacing: 20) {
        TextAreaView(
            label: "Notes",
            text: .constant(""),
            placeholder: "Practice observations..."
        )

        TextAreaView(
            label: "Notes",
            text: .constant("Focus on legato phrasing in the arpeggiated section."),
            hint: "What to focus on next time"
        )

        TextAreaView(
            label: "Notes",
            text: .constant(""),
            error: "Notes cannot be empty"
        )
    }
    .padding()
    .background(Color(red: 0.05, green: 0.05, blue: 0.10))
}
