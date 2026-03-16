import SwiftUI

/// Chip-based multi-tag input with autocomplete suggestions.
/// Tags are displayed as removable pills, with an inline text field for adding new tags.
struct TagInputView: View {
    @Binding var tags: [String]
    var availableTags: [String] = []
    var error: String? = nil

    @State private var inputText: String = ""
    @State private var showSuggestions: Bool = false
    @FocusState private var isFocused: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Tags")
                .formLabelStyle()

            Text("Press return to add a tag")
                .hintTextStyle()

            ZStack(alignment: .topLeading) {
                VStack(alignment: .leading, spacing: 8) {
                    // Tag chips + input field
                    FlowLayout(spacing: 6) {
                        ForEach(tags, id: \.self) { tag in
                            tagChip(tag)
                        }

                        TextField("Add tag...", text: $inputText)
                            .focused($isFocused)
                            .frame(minWidth: 80)
                            .onSubmit {
                                commitTag()
                            }
                            .onChange(of: inputText) { _, newValue in
                                showSuggestions = isFocused && newValue.count >= 1 && !filteredSuggestions.isEmpty
                            }
                            .onChange(of: isFocused) { _, focused in
                                if !focused {
                                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
                                        showSuggestions = false
                                    }
                                }
                            }
                    }
                    .padding(10)
                    .background(Color.surfaceInput)
                    .clipShape(RoundedRectangle(cornerRadius: DesignRadius.input))
                    .overlay(
                        RoundedRectangle(cornerRadius: DesignRadius.input)
                            .strokeBorder(
                                error.hasContent ? Color.dangerText : Color.borderInput,
                                lineWidth: 1
                            )
                    )
                }

                if showSuggestions {
                    suggestionsDropdown
                        .offset(y: chipAreaHeight + 8)
                }
            }
            .zIndex(1)

            FormFieldError(message: error)
        }
    }

    // Approximate height for positioning dropdown
    private var chipAreaHeight: CGFloat {
        let chipCount = tags.count
        if chipCount == 0 { return 44 }
        // Rough estimate: each row ~32px, ~4 chips per row
        let rows = max(1, (chipCount + 3) / 4)
        return CGFloat(rows) * 36 + 16
    }

    private func tagChip(_ tag: String) -> some View {
        HStack(spacing: 4) {
            Text(tag)
                .font(.caption)
                .foregroundStyle(Color.textSecondary)

            Button {
                tags.removeAll { $0.lowercased() == tag.lowercased() }
            } label: {
                Image(systemName: "xmark")
                    .font(.caption2)
                    .foregroundStyle(Color.textMuted)
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Remove \(tag)")
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 5)
        .background(Color.surfaceSecondary)
        .clipShape(Capsule())
        .overlay(
            Capsule()
                .strokeBorder(Color.borderDefault, lineWidth: 1)
        )
    }

    private func commitTag() {
        let trimmed = inputText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }

        // Case-insensitive deduplication
        let isDuplicate = tags.contains { $0.lowercased() == trimmed.lowercased() }
        if !isDuplicate {
            tags.append(trimmed)
        }
        inputText = ""
        showSuggestions = false
    }

    private var filteredSuggestions: [String] {
        let query = inputText.lowercased()
        return Array(availableTags
            .filter { suggestion in
                suggestion.lowercased().contains(query)
                    && !tags.contains(where: { $0.lowercased() == suggestion.lowercased() })
            }
            .prefix(6))
    }

    private var suggestionsDropdown: some View {
        VStack(spacing: 0) {
            ForEach(filteredSuggestions, id: \.self) { suggestion in
                Button {
                    tags.append(suggestion)
                    inputText = ""
                    showSuggestions = false
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
        @State private var tags = ["romantic", "exam"]
        var body: some View {
            TagInputView(
                tags: $tags,
                availableTags: ["romantic", "exam", "grade 8", "recital", "scales"]
            )
            .padding()
            .background(Color.backgroundApp)
        }
    }
    return Preview()
        .preferredColorScheme(.dark)
}
