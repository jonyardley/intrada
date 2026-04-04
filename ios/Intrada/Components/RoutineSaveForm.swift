import SwiftUI

/// Collapsible save-as-routine form.
///
/// Collapsed: shows "Save as Routine" button.
/// Expanded: shows name TextField + Save/Cancel buttons.
/// Validates name (non-empty, ≤200 chars).
///
///     RoutineSaveForm { name in
///         core.update(.routine(.saveBuildingAsRoutine(name: name)))
///     }
struct RoutineSaveForm: View {
    let onSave: (String) -> Void

    @State private var isExpanded: Bool = false
    @State private var routineName: String = ""
    @State private var errorMessage: String? = nil

    var body: some View {
        VStack(spacing: 8) {
            if isExpanded {
                VStack(spacing: Spacing.cardCompact) {
                    TextField("Routine name", text: $routineName)
                        .font(.system(size: 14))
                        .foregroundStyle(Color.textPrimary)
                        .padding(.horizontal, Spacing.cardCompact)
                        .frame(height: 40)
                        .background(Color.surfaceInput)
                        .clipShape(RoundedRectangle(cornerRadius: DesignRadius.input))
                        .overlay(
                            RoundedRectangle(cornerRadius: DesignRadius.input)
                                .stroke(Color.borderInput, lineWidth: 1)
                        )
                        .onSubmit { save() }

                    if let errorMessage {
                        Text(errorMessage)
                            .font(.system(size: 12))
                            .foregroundStyle(Color.dangerText)
                    }

                    HStack(spacing: 10) {
                        ButtonView("Save", variant: .primary) { save() }
                        ButtonView("Cancel", variant: .secondary) {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                isExpanded = false
                                routineName = ""
                                errorMessage = nil
                            }
                        }
                    }
                }
            } else {
                Button {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        isExpanded = true
                    }
                } label: {
                    HStack(spacing: 6) {
                        Image(systemName: "bookmark.fill")
                            .font(.system(size: 12))
                        Text("Save as Routine")
                            .font(.system(size: 13, weight: .medium))
                    }
                    .foregroundStyle(Color.accentText)
                    .frame(maxWidth: .infinity)
                    .frame(height: 40)
                    .overlay(
                        RoundedRectangle(cornerRadius: DesignRadius.button)
                            .stroke(Color.accentText.opacity(0.3), style: StrokeStyle(lineWidth: 1, dash: [6, 3]))
                    )
                }
            }
        }
    }

    private func save() {
        let trimmed = routineName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            errorMessage = "Please enter a routine name"
            return
        }
        guard trimmed.count <= 200 else {
            errorMessage = "Name must be 200 characters or fewer"
            return
        }
        onSave(trimmed)
        withAnimation(.easeInOut(duration: 0.2)) {
            isExpanded = false
            routineName = ""
            errorMessage = nil
        }
    }
}

#Preview("RoutineSaveForm") {
    VStack(spacing: 24) {
        RoutineSaveForm { name in
            print("Saved routine: \(name)")
        }
    }
    .padding()
    .background(Color.backgroundApp)
    .preferredColorScheme(.dark)
}
