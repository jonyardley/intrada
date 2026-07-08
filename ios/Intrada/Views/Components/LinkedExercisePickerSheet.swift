import SharedTypes
import SwiftUI

/// Add/remove manager for a piece's related exercises. Lists every exercise
/// with the already-related ones pre-selected; tapping toggles membership and
/// "Done" hands the final set back — the caller links the added and unlinks the
/// removed. (Reorder stays in the detail card's Edit mode.)
struct LinkedExercisePickerSheet: View {
  let available: [LibraryItemView]
  let linkedIds: [String]
  let onApply: (Swift.Set<String>) -> Void

  @Environment(\.dismiss) private var dismiss
  @State private var selected: Swift.Set<String>

  init(
    available: [LibraryItemView], linkedIds: [String],
    onApply: @escaping (Swift.Set<String>) -> Void
  ) {
    self.available = available
    self.linkedIds = linkedIds
    self.onApply = onApply
    _selected = State(initialValue: Swift.Set(linkedIds))
  }

  var body: some View {
    BottomSheet(
      title: "Add exercises",
      onDone: { onApply(selected) },
      leadingAction: { Button("Cancel") { dismiss() } }
    ) {
      if available.isEmpty {
        PlaceholderContent(
          systemImage: "music.note.list",
          message: "No exercises yet. Create an exercise to relate it to this piece.")
      } else {
        list
      }
    }
  }

  private var list: some View {
    ScrollView {
      VStack(spacing: 0) {
        createNewRow
        HairlineDivider().padding(.leading, IntradaSpacing.card)
        Text("Your exercises")
          .font(IntradaFont.eyebrow)
          .textCase(.uppercase)
          .kerning(1.2)
          .foregroundStyle(IntradaColor.inkFaint)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.top, IntradaSpacing.cardCompact)
          .padding(.bottom, IntradaSpacing.controlGap)
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

  // ── Rows ──

  private var createNewRow: some View {
    NavigationLink(destination: LibraryAddScreen(defaultKind: .exercise)) {
      HStack(spacing: IntradaSpacing.cardCompact) {
        Image(systemName: "plus")
          .font(.system(size: 16, weight: .semibold))
          .foregroundStyle(IntradaColor.accent)
          .frame(width: 28, height: 28)
          .background(Circle().fill(IntradaColor.pieceBadgeBg))
          .accessibilityHidden(true)
        Text("Create new exercise")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
        Spacer(minLength: 0)
      }
      .padding(.vertical, IntradaSpacing.cardCompact)
      .padding(.horizontal, IntradaSpacing.card)
      .frame(maxWidth: .infinity, alignment: .leading)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Create a new exercise")
  }

  private func exerciseRow(_ exercise: LibraryItemView, isOn: Bool) -> some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      ItemKind.exercise.bar
        .frame(width: 4, height: 30)
        .clipShape(Capsule())
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
      membershipControl(isOn: isOn)
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.card)
    .frame(maxWidth: .infinity, alignment: .leading)
    .contentShape(Rectangle())
  }

  private func membershipControl(isOn: Bool) -> some View {
    ZStack {
      Circle()
        .fill(isOn ? AnyShapeStyle(IntradaColor.exerciseAccent) : AnyShapeStyle(Color.clear))
        .overlay(
          Circle()
            .strokeBorder(IntradaColor.exerciseAccent, lineWidth: 2)
            .opacity(isOn ? 0 : 1))
      Image(systemName: isOn ? "checkmark" : "plus")
        .font(.system(size: 14, weight: .semibold))
        .foregroundStyle(isOn ? IntradaColor.onExercise : IntradaColor.exerciseAccent)
    }
    .frame(width: 28, height: 28)
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

  private func rowAccessibilityLabel(_ exercise: LibraryItemView, isOn: Bool) -> String {
    var parts = [exercise.title]
    if let meta = metaLine(exercise) { parts.append(meta) }
    parts.append(isOn ? "related, tap to remove" : "not related, tap to add")
    return parts.joined(separator: ", ")
  }
}

#if DEBUG
  #Preview("Add or remove — one related") {
    LinkedExercisePickerSheet(
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
      linkedIds: ["exercise-1"],
      onApply: { _ in })
  }

  #Preview("Empty") {
    LinkedExercisePickerSheet(available: [], linkedIds: [], onApply: { _ in })
  }
#endif
