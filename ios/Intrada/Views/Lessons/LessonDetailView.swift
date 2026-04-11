import SwiftUI

/// Full detail view of a captured lesson — date, notes, photos.
/// Supports edit mode and delete with confirmation.
struct LessonDetailView: View {
    let lessonId: String
    @Binding var selectedLessonId: String?

    @Environment(IntradaCore.self) private var core
    @Environment(ToastManager.self) private var toast

    @State private var isEditing = false
    @State private var editDate: Date = .now
    @State private var editNotes: String = ""
    @State private var showDeleteConfirmation = false
    @State private var isSubmitting = false

    private var lesson: LessonView? {
        core.viewModel.lessons.first { $0.id == lessonId }
    }

    var body: some View {
        Group {
            if let lesson {
                ScrollView {
                    VStack(alignment: .leading, spacing: Spacing.card) {
                        if isEditing {
                            editContent(lesson)
                        } else {
                            readContent(lesson)
                        }
                    }
                    .padding(Spacing.card)
                }
                .navigationTitle(formatDate(lesson.date))
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .primaryAction) {
                        if isEditing {
                            Button("Save") { saveEdits() }
                                .disabled(isSubmitting)
                        } else {
                            Menu {
                                Button { startEditing(lesson) } label: {
                                    Label("Edit", systemImage: "pencil")
                                }
                                Button(role: .destructive) {
                                    showDeleteConfirmation = true
                                } label: {
                                    Label("Delete", systemImage: "trash")
                                }
                            } label: {
                                Image(systemName: "ellipsis.circle")
                            }
                        }
                    }
                    if isEditing {
                        ToolbarItem(placement: .cancellationAction) {
                            Button("Cancel") { isEditing = false }
                        }
                    }
                }
            } else {
                EmptyStateView(
                    icon: "doc.text",
                    title: "Lesson Not Found",
                    message: "This lesson may have been deleted"
                )
            }
        }
        .onAppear {
            core.update(.lesson(.fetchLesson(id: lessonId)))
        }
        .confirmationDialog(
            "Delete Lesson",
            isPresented: $showDeleteConfirmation,
            titleVisibility: .visible
        ) {
            Button("Delete", role: .destructive) { deleteLesson() }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Are you sure you want to delete this lesson? This cannot be undone.")
        }
        .onChange(of: core.viewModel.lessons.count) { oldCount, newCount in
            // Detect deletion
            guard newCount < oldCount else { return }
            if !core.viewModel.lessons.contains(where: { $0.id == lessonId }) {
                toast.show("Lesson deleted", variant: .success)
                selectedLessonId = nil
            }
        }
    }

    // MARK: - Read Mode

    @ViewBuilder
    private func readContent(_ lesson: LessonView) -> some View {
        CardView {
            VStack(alignment: .leading, spacing: Spacing.cardCompact) {
                // Notes
                if let notes = lesson.notes, !notes.isEmpty {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("NOTES")
                            .font(.caption)
                            .fontWeight(.medium)
                            .foregroundStyle(Color.textFaint)
                            .tracking(0.8)

                        Text(notes)
                            .font(.subheadline)
                            .foregroundStyle(Color.textSecondary)
                    }
                }

                // Photos
                if !lesson.photos.isEmpty {
                    Divider()
                        .overlay(Color.borderDefault)

                    photoGallery(lesson.photos)
                }
            }
        }
    }

    // MARK: - Edit Mode

    @ViewBuilder
    private func editContent(_ lesson: LessonView) -> some View {
        CardView {
            VStack(alignment: .leading, spacing: Spacing.card) {
                // Date picker
                VStack(alignment: .leading, spacing: 4) {
                    Text("Date")
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .foregroundStyle(Color.textLabel)

                    DatePicker(
                        "",
                        selection: $editDate,
                        in: ...Date.now,
                        displayedComponents: .date
                    )
                    .datePickerStyle(.compact)
                    .labelsHidden()
                }

                // Notes
                TextAreaView(
                    label: "Notes",
                    text: $editNotes,
                    placeholder: "What happened in your lesson?",
                    error: nil
                )

                // Photo management
                PhotoCaptureView(lessonId: lessonId) {
                    core.update(.lesson(.fetchLesson(id: lessonId)))
                }

                // Existing photos
                if !lesson.photos.isEmpty {
                    photoGallery(lesson.photos)
                }
            }
        }
    }

    // MARK: - Shared Components

    @ViewBuilder
    private func photoGallery(_ photos: [LessonPhotoView]) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("PHOTOS")
                .font(.caption)
                .fontWeight(.medium)
                .foregroundStyle(Color.textFaint)
                .tracking(0.8)

            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    ForEach(photos, id: \.id) { photo in
                        AsyncImage(url: URL(string: photo.url)) { phase in
                            switch phase {
                            case .success(let image):
                                image
                                    .resizable()
                                    .scaledToFill()
                                    .frame(width: 80, height: 80)
                                    .clipShape(RoundedRectangle(cornerRadius: DesignRadius.badge))
                            case .failure:
                                photoPlaceholder
                            case .empty:
                                ProgressView()
                                    .frame(width: 80, height: 80)
                            @unknown default:
                                photoPlaceholder
                            }
                        }
                    }
                }
            }
        }
    }

    private var photoPlaceholder: some View {
        RoundedRectangle(cornerRadius: DesignRadius.badge)
            .fill(Color.surfaceSecondary)
            .frame(width: 80, height: 80)
            .overlay(
                Image(systemName: "photo")
                    .foregroundStyle(Color.textFaint)
            )
    }

    // MARK: - Actions

    private func startEditing(_ lesson: LessonView) {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        editDate = formatter.date(from: lesson.date) ?? .now
        editNotes = lesson.notes ?? ""
        isEditing = true
    }

    private func saveEdits() {
        isSubmitting = true

        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        let dateString = formatter.string(from: editDate)

        let trimmedNotes = editNotes.trimmingCharacters(in: .whitespacesAndNewlines)

        let update = UpdateLesson(
            date: dateString,
            notes: trimmedNotes.isEmpty ? nil : trimmedNotes
        )

        core.update(.lesson(.update(id: lessonId, input: update)))
        isEditing = false
        isSubmitting = false
        toast.show("Lesson updated", variant: .success)
    }

    private func deleteLesson() {
        core.update(.lesson(.delete(id: lessonId)))
    }

    private func formatDate(_ dateString: String) -> String {
        let inputFormatter = DateFormatter()
        inputFormatter.dateFormat = "yyyy-MM-dd"
        guard let date = inputFormatter.date(from: dateString) else { return dateString }

        let outputFormatter = DateFormatter()
        outputFormatter.dateStyle = .long
        return outputFormatter.string(from: date)
    }
}
