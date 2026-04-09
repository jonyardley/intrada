import SwiftUI

/// Text field with dropdown autocomplete suggestions.
/// Used for composer and tag inputs where existing values should be suggested.
struct AutocompleteField: View {
    let label: String
    @Binding var text: String
    var placeholder: String = ""
    var hint: String? = nil
    var error: String? = nil
    var suggestions: [String] = []
    var minChars: Int = 2
    var maxSuggestions: Int = 8
    var onCommit: (() -> Void)? = nil

    @State private var showSuggestions: Bool = false
    @FocusState private var isFocused: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .formLabelStyle()

            if let hint {
                Text(hint)
                    .hintTextStyle()
            }

            ZStack(alignment: .topLeading) {
                TextField(placeholder, text: $text)
                    .inputStyle(hasError: error.hasContent)
                    .focused($isFocused)
                    .onChange(of: text) { _, newValue in
                        showSuggestions = isFocused && newValue.count >= minChars && !filteredSuggestions.isEmpty
                    }
                    .onChange(of: isFocused) { _, focused in
                        if !focused {
                            // Delay hiding to allow tap on suggestion
                            Task {
                                try? await Task.sleep(for: .milliseconds(200))
                                showSuggestions = false
                            }
                        } else {
                            showSuggestions = text.count >= minChars && !filteredSuggestions.isEmpty
                        }
                    }
                    .onSubmit {
                        showSuggestions = false
                        onCommit?()
                    }

                if showSuggestions {
                    suggestionsDropdown
                        .offset(y: 44) // Below the text field
                }
            }
            .zIndex(1)

            FormFieldError(message: error)
        }
    }

    private var filteredSuggestions: [String] {
        let query = text.lowercased()
        let filtered = suggestions.filter { suggestion in
            suggestion.lowercased().contains(query) && suggestion.lowercased() != query
        }
        // Prefix matches first, then contains matches
        let prefixMatches = filtered.filter { $0.lowercased().hasPrefix(query) }
        let containsMatches = filtered.filter { !$0.lowercased().hasPrefix(query) }
        return Array((prefixMatches + containsMatches).prefix(maxSuggestions))
    }

    private var suggestionsDropdown: some View {
        VStack(spacing: 0) {
            ForEach(filteredSuggestions, id: \.self) { suggestion in
                Button {
                    text = suggestion
                    showSuggestions = false
                    isFocused = false
                } label: {
                    Text(suggestion)
                        .font(.body)
                        .foregroundStyle(Color.textPrimary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 10)
                }
                .buttonStyle(.plain)

                if suggestion != filteredSuggestions.last {
                    Divider()
                        .overlay(Color.borderDefault)
                }
            }
        }
        .background(Color.surfacePrimary)
        .clipShape(RoundedRectangle(cornerRadius: DesignRadius.input))
        .overlay(
            RoundedRectangle(cornerRadius: DesignRadius.input)
                .strokeBorder(Color.borderInput, lineWidth: 1)
        )
        .shadow(color: .shadowDefault, radius: 8, y: 4)
    }
}

#Preview {
    struct Preview: View {
        @State private var text = ""
        var body: some View {
            AutocompleteField(
                label: "Composer",
                text: $text,
                placeholder: "e.g. Claude Debussy",
                suggestions: ["Claude Debussy", "Frédéric Chopin", "Johann Sebastian Bach"]
            )
            .padding()
            .background(Color.backgroundApp)
        }
    }
    return Preview()
        .preferredColorScheme(.dark)
}
