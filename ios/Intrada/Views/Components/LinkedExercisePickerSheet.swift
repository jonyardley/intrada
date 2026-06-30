import SharedTypes
import SwiftUI

/// Multi-select sheet for linking exercises to a piece. Lists all exercises
/// not already linked; the user picks one or more then confirms with "Add N".
struct LinkedExercisePickerSheet: View {
  let available: [LibraryItemView]
  let pieceId: String
  let onAdd: ([String]) -> Void

  @Environment(\.dismiss) private var dismiss
  @State private var selected: Swift.Set<String>

  init(available: [LibraryItemView], pieceId: String, onAdd: @escaping ([String]) -> Void) {
    self.available = available
    self.pieceId = pieceId
    self.onAdd = onAdd
    _selected = State(initialValue: [])
  }

  #if DEBUG
    init(
      available: [LibraryItemView], pieceId: String, preselected: [String],
      onAdd: @escaping ([String]) -> Void
    ) {
      self.available = available
      self.pieceId = pieceId
      self.onAdd = onAdd
      _selected = State(initialValue: Swift.Set(preselected))
    }
  #endif

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        if available.isEmpty {
          PlaceholderContent(
            systemImage: "music.note.list",
            message: "No exercises to link. Add an exercise to your library first.")
        } else {
          ScrollView {
            VStack(spacing: 0) {
              createNewRow
              HairlineDivider().padding(.leading, IntradaSpacing.card)
              ForEach(available, id: \.id) { exercise in
                let isOn = selected.contains(exercise.id)
                Button {
                  toggle(exercise.id, isOn: isOn)
                } label: {
                  exerciseRow(exercise, isOn: isOn)
                }
                .buttonStyle(.plain)
                .accessibilityLabel(rowAccessibilityLabel(exercise, isOn: isOn))
                .accessibilityAddTraits(isOn ? [.isButton, .isSelected] : .isButton)

                if exercise.id != available.last?.id {
                  HairlineDivider().padding(.leading, IntradaSpacing.card)
                }
              }
            }
            .cardSurface()
            .padding(IntradaSpacing.card)
          }
        }
      }
      .navigationTitle("Link an exercise")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .cancellationAction) {
          Button("Cancel") { dismiss() }
        }
        ToolbarItem(placement: .confirmationAction) {
          Button(addLabel) {
            onAdd(Array(selected))
            dismiss()
          }
          .disabled(selected.isEmpty)
          .accessibilityLabel(addAccessibilityLabel)
        }
      }
    }
    .presentationDetents([.medium, .large])
  }

  // ── Rows ──

  private var createNewRow: some View {
    NavigationLink(destination: LibraryAddScreen(defaultKind: .exercise)) {
      HStack(spacing: IntradaSpacing.cardCompact) {
        Image(systemName: "plus.circle.fill")
          .imageScale(.medium)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.accent)
          .accessibilityHidden(true)
        Text("Create new exercise")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
        Spacer(minLength: 0)
      }
      .padding(.vertical, IntradaSpacing.row)
      .padding(.horizontal, IntradaSpacing.card)
      .frame(maxWidth: .infinity, alignment: .leading)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Create a new exercise")
  }

  @ViewBuilder
  private func exerciseRow(_ exercise: LibraryItemView, isOn: Bool) -> some View {
    HStack(spacing: IntradaSpacing.row) {
      VStack(alignment: .leading, spacing: 3) {
        Text(exercise.title)
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        if let meta = metaLine(exercise) {
          Text(meta)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      if isOn {
        Image(systemName: "checkmark")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
      }
    }
    .padding(.vertical, IntradaSpacing.row)
    .padding(.horizontal, IntradaSpacing.card)
    .frame(maxWidth: .infinity, alignment: .leading)
    .contentShape(Rectangle())
  }

  // ── Helpers ──

  private func metaLine(_ exercise: LibraryItemView) -> String? {
    let parts = [exercise.keyDisplay, exercise.tempoDisplay]
      .compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  private func toggle(_ id: String, isOn: Bool) {
    if isOn {
      selected.remove(id)
    } else {
      selected.insert(id)
    }
    UISelectionFeedbackGenerator().selectionChanged()
  }

  private var addLabel: String {
    selected.isEmpty ? "Add" : "Add \(selected.count)"
  }

  private var addAccessibilityLabel: String {
    if selected.isEmpty { return "Add exercises, disabled" }
    return "Add \(selected.count) \(selected.count == 1 ? "exercise" : "exercises")"
  }

  private func rowAccessibilityLabel(_ exercise: LibraryItemView, isOn: Bool) -> String {
    var parts = [exercise.title]
    if let meta = metaLine(exercise) { parts.append(meta) }
    parts.append(isOn ? "selected" : "not selected")
    return parts.joined(separator: ", ")
  }
}

#if DEBUG
  /// Snapshot seam: exposes the picker with a pre-seeded selection so the test
  /// captures "some selected + Add N enabled" without UI interaction.
  struct LinkedExercisePickerSheetWrapper: View {
    let available: [LibraryItemView]
    let pieceId: String
    let preselected: [String]

    var body: some View {
      LinkedExercisePickerSheet(
        available: available, pieceId: pieceId, preselected: preselected, onAdd: { _ in })
    }
  }

  #Preview("With exercises — some selected") {
    LinkedExercisePickerSheetWrapper(
      available: [
        .previewExercise,
        LibraryItemView(
          id: "exercise-2", itemType: .exercise, title: "Db Major Scale", subtitle: "",
          key: "Db", modality: .major, tempo: nil, tempoMarking: nil, tempoBpm: nil,
          notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
          latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: []),
        LibraryItemView(
          id: "exercise-3", itemType: .exercise, title: "Arpeggios in Db", subtitle: "",
          key: "Db", modality: .major, tempo: nil, tempoMarking: nil, tempoBpm: nil,
          notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
          latestAchievedTempo: nil, priority: false, linkedExercises: [], linkedFromPieces: []),
      ],
      pieceId: "piece-1",
      preselected: ["exercise-1"])
  }

  #Preview("Empty") {
    LinkedExercisePickerSheet(
      available: [],
      pieceId: "piece-1",
      onAdd: { _ in })
  }
#endif
