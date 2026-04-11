import SwiftUI

/// Streamlined form for capturing a lesson — date, notes, photos.
/// Optimised for speed: notes is the hero field, everything else optional.
/// Presented as a sheet from the Library view.
struct LessonCaptureView: View {
    @Environment(IntradaCore.self) private var core
    @Environment(ToastManager.self) private var toast
    @Environment(\.dismiss) private var dismiss

    @State private var date: Date = .now
    @State private var notes: String = ""
    @State private var isSubmitting = false
    @State private var savedLessonId: String?

    /// Track lesson count to detect when core processes the create.
    @State private var lessonCountBeforeSubmit: Int?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: Spacing.card) {
                // Date picker
                VStack(alignment: .leading, spacing: 4) {
                    Text("Date")
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .foregroundStyle(Color.textLabel)

                    DatePicker(
                        "",
                        selection: $date,
                        in: ...Date.now,
                        displayedComponents: .date
                    )
                    .datePickerStyle(.compact)
                    .labelsHidden()
                }

                // Notes — the hero field
                TextAreaView(
                    label: "Notes",
                    text: $notes,
                    placeholder: "What happened in your lesson? What did your teacher say?",
                    error: nil
                )

                // Photo upload — only available after save (needs lesson ID)
                if let lessonId = savedLessonId {
                    PhotoCaptureView(lessonId: lessonId) {
                        // Refresh lesson data after photo upload
                        core.update(.lesson(.fetchLesson(id: lessonId)))
                    }
                } else {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Photos")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundStyle(Color.textLabel)

                        Text("Save the lesson first, then attach photos")
                            .font(.caption)
                            .foregroundStyle(Color.textMuted)
                    }
                }

                // Save button
                ButtonView(
                    "Save Lesson",
                    variant: .primary,
                    disabled: isSubmitting || notes.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
                    loading: isSubmitting
                ) {
                    submitForm()
                }
            }
            .padding(Spacing.card)
        }
        .navigationTitle("Log Lesson")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") { dismiss() }
            }
        }
        .onChange(of: core.viewModel.lessons.count) { _, newCount in
            guard isSubmitting, let before = lessonCountBeforeSubmit, newCount > before else { return }
            toast.show("Lesson saved", variant: .success)
            isSubmitting = false
            dismiss()
        }
        .onChange(of: core.viewModel.error) { _, newError in
            guard isSubmitting, newError != nil else { return }
            isSubmitting = false
        }
    }

    private func submitForm() {
        let trimmedNotes = notes.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedNotes.isEmpty else { return }

        isSubmitting = true
        lessonCountBeforeSubmit = core.viewModel.lessons.count

        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        let dateString = formatter.string(from: date)

        let createLesson = CreateLesson(
            date: dateString,
            notes: trimmedNotes.isEmpty ? nil : trimmedNotes
        )

        core.update(.lesson(.add(createLesson)))
    }
}

#Preview {
    NavigationStack {
        LessonCaptureView()
    }
    .environment(IntradaCore())
    .environment(ToastManager())
    .preferredColorScheme(.dark)
}
