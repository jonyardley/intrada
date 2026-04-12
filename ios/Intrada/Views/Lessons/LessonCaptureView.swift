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

    @State private var lessonIdsBeforeSubmit: Set<String>?

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
                .disabled(savedLessonId != nil)

                // Notes — the hero field
                TextAreaView(
                    label: "Notes",
                    text: $notes,
                    placeholder: "What happened in your lesson? What did your teacher say?",
                    error: nil
                )
                .disabled(savedLessonId != nil)

                // Photo upload — only available after save (needs lesson ID)
                if let lessonId = savedLessonId {
                    PhotoCaptureView(lessonId: lessonId) {
                        core.update(.lesson(.fetchLesson(id: lessonId)))
                    }
                }

                if savedLessonId == nil {
                    ButtonView(
                        "Save Lesson",
                        variant: .primary,
                        disabled: isSubmitting || notes.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
                        loading: isSubmitting
                    ) {
                        submitForm()
                    }
                }
            }
            .padding(Spacing.card)
        }
        .navigationTitle("Log Lesson")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                if savedLessonId == nil {
                    Button("Cancel") { dismiss() }
                }
            }
            ToolbarItem(placement: .confirmationAction) {
                if savedLessonId != nil {
                    Button("Done") { dismiss() }
                }
            }
        }
        .onChange(of: core.viewModel.lessons.count) { _, _ in
            guard isSubmitting, let oldIds = lessonIdsBeforeSubmit else { return }

            if let newLesson = core.viewModel.lessons.first(where: { !oldIds.contains($0.id) }) {
                savedLessonId = newLesson.id
                toast.show("Lesson saved — add photos or tap Done", variant: .success)
                isSubmitting = false
            }
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
        lessonIdsBeforeSubmit = Set(core.viewModel.lessons.map(\.id))

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
