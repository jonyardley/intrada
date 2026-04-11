import SwiftUI

/// Reverse chronological list of lessons with NavigationSplitView on iPad.
struct LessonListView: View {
    @Environment(IntradaCore.self) private var core
    @State private var selectedLessonId: String?

    var body: some View {
        NavigationSplitView {
            Group {
                if core.viewModel.lessons.isEmpty {
                    EmptyStateView(
                        icon: "pencil.and.list.clipboard",
                        title: "No Lessons Yet",
                        message: "Log your first lesson from the Library tab"
                    )
                } else {
                    List(core.viewModel.lessons, id: \.id, selection: $selectedLessonId) { lesson in
                        LessonRow(lesson: lesson)
                    }
                    .listStyle(.plain)
                }
            }
            .navigationTitle("Lessons")
            .onAppear {
                core.update(.lesson(.fetchLessons))
            }
        } detail: {
            if let lessonId = selectedLessonId {
                LessonDetailView(
                    lessonId: lessonId,
                    selectedLessonId: $selectedLessonId
                )
            } else {
                EmptyStateView(
                    icon: "doc.text",
                    title: "Select a Lesson",
                    message: "Choose a lesson to view its details"
                )
            }
        }
    }
}

/// A single row in the lesson list.
private struct LessonRow: View {
    let lesson: LessonView

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(formatDate(lesson.date))
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundStyle(Color.textPrimary)

                Spacer()

                if lesson.hasPhotos {
                    Image(systemName: "photo")
                        .font(.caption)
                        .foregroundStyle(Color.textMuted)
                }
            }

            if !lesson.notesPreview.isEmpty {
                Text(lesson.notesPreview)
                    .font(.caption)
                    .foregroundStyle(Color.textSecondary)
                    .lineLimit(2)
            }
        }
        .padding(.vertical, 4)
        .listRowBackground(Color.clear)
    }

    private func formatDate(_ dateString: String) -> String {
        let inputFormatter = DateFormatter()
        inputFormatter.dateFormat = "yyyy-MM-dd"
        guard let date = inputFormatter.date(from: dateString) else { return dateString }

        let outputFormatter = DateFormatter()
        outputFormatter.dateStyle = .medium
        return outputFormatter.string(from: date)
    }
}

#Preview {
    LessonListView()
        .environment(IntradaCore())
        .preferredColorScheme(.dark)
}
