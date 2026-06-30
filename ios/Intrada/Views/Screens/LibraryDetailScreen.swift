import SharedTypes
import SwiftUI

/// Detail for a library item: type badge, key/tempo, notes, tags, and delete.
struct LibraryDetailScreen: View {
  let item: LibraryItemView

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var confirmingDelete = false
  @State private var editing = false
  @State private var editingLinks: Bool
  @State private var showingPicker = false

  init(item: LibraryItemView, startEditingLinks: Bool = false) {
    self.item = item
    _editingLinks = State(initialValue: startEditingLinks)
  }

  var body: some View {
    ScreenScaffold(title: item.title, subtitle: subtitle) {
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.card) {
          TypeBadge(kind: item.itemType)

          if !detailRows.isEmpty {
            VStack(spacing: 0) {
              ForEach(Array(detailRows.enumerated()), id: \.offset) { index, row in
                if index > 0 {
                  HairlineDivider()
                }
                DetailRow(label: row.label, value: row.value)
              }
            }
            .cardSurface()
          }

          if let notes = item.notes, !notes.isEmpty {
            Text(notes)
              .font(IntradaFont.body)
              .foregroundStyle(IntradaColor.inkSecondary)
              .frame(maxWidth: .infinity, alignment: .leading)
              .padding(IntradaSpacing.card)
              .cardSurface()
          }

          if !item.tags.isEmpty {
            tags
          }

          if item.itemType == .piece {
            linkedExercisesSection
          }

          if item.itemType == .exercise && !item.linkedFromPieces.isEmpty {
            linkedFromSection
          }

          deleteButton
            .padding(.top, IntradaSpacing.controlGap)
        }
        .padding(IntradaSpacing.card)
      }
      .scrollEdgeShadow()
    }
    .navigationBarTitleDisplayMode(.inline)
    .toolbar {
      ToolbarItem(placement: .topBarTrailing) {
        Button {
          toggleStar()
        } label: {
          Image(systemName: item.priority ? "star.fill" : "star")
            .foregroundStyle(item.priority ? IntradaColor.accent : IntradaColor.inkSecondary)
        }
        .accessibilityLabel(
          item.priority ? "Remove from priorities" : "Add to priorities")
      }
      ToolbarItem(placement: .topBarTrailing) {
        Button("Edit") { editing = true }
      }
    }
    .sheet(isPresented: $editing) {
      LibraryEditScreen(item: item)
        .environment(store)
    }
    .sheet(isPresented: $showingPicker) {
      LinkedExercisePickerSheet(
        available: availableExercises,
        pieceId: item.id,
        onAdd: { ids in linkExercises(ids) }
      )
      .environment(store)
    }
    // Alert (not confirmationDialog): always renders the Cancel button, incl.
    // iPad/regular-width where a confirmationDialog popover hides it.
    .alert("Delete \(item.title)?", isPresented: $confirmingDelete) {
      Button("Delete", role: .destructive, action: delete)
      Button("Cancel", role: .cancel) {}
    } message: {
      Text("This can't be undone.")
    }
  }

  // ── Linked exercises ──

  private var linkedExercisesSection: some View {
    VStack(spacing: 0) {
      linkedExercisesHeader
      if item.linkedExercises.isEmpty {
        linkedExercisesEmptyState
      } else {
        linkedExercisesRows
        HairlineDivider()
        linkExerciseButton
      }
    }
    .cardSurface()
    .onChange(of: item.linkedExercises.isEmpty) { _, isEmpty in
      if isEmpty { editingLinks = false }
    }
  }

  private var linkedExercisesHeader: some View {
    HStack {
      Text("Linked exercises")
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.ink)
      if !item.linkedExercises.isEmpty {
        Text("\(item.linkedExercises.count)")
          .font(IntradaFont.badge)
          .foregroundStyle(IntradaColor.inkSecondary)
          // Badge insets: 7/3 are capsule-specific — smaller than controlGap(8).
          .padding(.horizontal, 7)
          .padding(.vertical, 3)
          .background(IntradaColor.surfaceSunken, in: Capsule())
          .accessibilityHidden(true)
      }
      Spacer()
      Button(editingLinks ? "Done" : "Edit") {
        editingLinks.toggle()
      }
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.accent)
      .disabled(item.linkedExercises.isEmpty)
      .opacity(item.linkedExercises.isEmpty ? 0 : 1)
      .accessibilityLabel(editingLinks ? "Done editing linked exercises" : "Edit linked exercises")
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.card)
    .padding(.bottom, item.linkedExercises.isEmpty ? 0 : IntradaSpacing.cardCompact)
  }

  @ViewBuilder private var linkedExercisesRows: some View {
    if editingLinks {
      ForEach(Array(item.linkedExercises.enumerated()), id: \.element.id) { index, exercise in
        if index > 0 {
          HairlineDivider()
        }
        LinkedExerciseEditRow(
          exercise: exercise,
          isFirst: index == 0,
          isLast: index == item.linkedExercises.count - 1,
          onRemove: { unlink(exercise) },
          onMoveUp: { moveExercise(at: index, by: -1) },
          onMoveDown: { moveExercise(at: index, by: 1) })
      }
    } else {
      ForEach(Array(item.linkedExercises.enumerated()), id: \.element.id) { index, exercise in
        if index > 0 {
          HairlineDivider()
        }
        NavigationLink(value: exercise.id) {
          LinkedExerciseRow(exercise: exercise)
        }
        .buttonStyle(.plain)
      }
    }
  }

  private var linkedExercisesEmptyState: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      Image(systemName: "link")
        .imageScale(.large)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkFaint)
        .accessibilityHidden(true)
      Text("No exercises linked yet")
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.ink)
      Text("Link scales, arpeggios, or any exercise to track progress alongside this piece.")
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkSecondary)
        .multilineTextAlignment(.center)
      Button {
        showingPicker = true
      } label: {
        Text("Link an exercise")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.vertical, IntradaSpacing.cardCompact)
          .overlay(
            RoundedRectangle(cornerRadius: IntradaRadius.card)
              .stroke(IntradaColor.accent, lineWidth: 1)
          )
      }
      .buttonStyle(.plain)
      .padding(.top, IntradaSpacing.controlGap)
      .accessibilityLabel("Link an exercise to this piece")
    }
    .frame(maxWidth: .infinity)
    .padding(IntradaSpacing.card)
    .padding(.bottom, IntradaSpacing.cardCompact)
  }

  private var linkExerciseButton: some View {
    Button {
      showingPicker = true
    } label: {
      Label("Link an exercise", systemImage: "plus")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.accent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.cardCompact)
    }
    .buttonStyle(.plain)
    .accessibilityLabel("Link an exercise to this piece")
  }

  // ── Linked from ──

  private var linkedFromSection: some View {
    VStack(spacing: 0) {
      HStack {
        Text("Linked from")
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        Spacer()
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.card)
      .padding(.bottom, IntradaSpacing.cardCompact)
      ForEach(Array(item.linkedFromPieces.enumerated()), id: \.element.id) { index, piece in
        if index > 0 {
          HairlineDivider()
        }
        NavigationLink(value: piece.id) {
          HStack {
            Text(piece.title)
              .font(IntradaFont.body)
              .foregroundStyle(IntradaColor.ink)
              .frame(maxWidth: .infinity, alignment: .leading)
            Image(systemName: "chevron.right")
              .imageScale(.small)
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.inkFaint)
          }
          .padding(.vertical, IntradaSpacing.cardCompact)
          .padding(.horizontal, IntradaSpacing.card)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("\(piece.title), linked from")
      }
    }
    .cardSurface()
  }

  // ── Actions ──

  private var availableExercises: [LibraryItemView] {
    let linked = Swift.Set(item.linkedExercises.map(\.id))
    return (store.viewModel?.items ?? []).filter {
      $0.itemType == .exercise && !linked.contains($0.id)
    }
  }

  private func linkExercises(_ ids: [String]) {
    var allSucceeded = true
    for id in ids {
      let before = store.viewModel?.error
      store.send(.item(.linkExercise(pieceId: item.id, exerciseId: id)))
      if store.viewModel?.error != before {
        allSucceeded = false
      }
    }
    if allSucceeded {
      UINotificationFeedbackGenerator().notificationOccurred(.success)
    }
  }

  private func unlink(_ exercise: LinkedExerciseView) {
    let before = store.viewModel?.error
    store.send(.item(.unlinkExercise(pieceId: item.id, exerciseId: exercise.id)))
    if store.viewModel?.error == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  private func moveExercise(at index: Int, by delta: Int) {
    var ids = item.linkedExercises.map(\.id)
    let dest = index + delta
    guard dest >= 0, dest < ids.count else { return }
    ids.swapAt(index, dest)
    let before = store.viewModel?.error
    store.send(.item(.reorderLinkedExercises(pieceId: item.id, orderedIds: ids)))
    if store.viewModel?.error == before {
      UISelectionFeedbackGenerator().selectionChanged()
    }
  }

  private var deleteButton: some View {
    Button(role: .destructive) {
      confirmingDelete = true
    } label: {
      Label("Delete \(item.itemType.label.lowercased())", systemImage: "trash")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.danger)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.cardCompact)
    }
    .buttonStyle(.plain)
  }

  private func delete() {
    UINotificationFeedbackGenerator().notificationOccurred(.warning)
    store.send(.item(.delete(id: item.id)))
    dismiss()
  }

  // Priority-only update: every other field is "no change" (nil). A failed write
  // surfaces on the global banner, not a silent no-op (#846).
  private func toggleStar() {
    store.send(
      .item(
        .update(
          id: item.id,
          input: UpdateItem(
            title: item.title, kind: item.itemType, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: nil, priority: !item.priority))))
  }

  private var subtitle: String? {
    item.subtitle.isEmpty ? nil : item.subtitle
  }

  private var detailRows: [(label: String, value: String)] {
    var rows: [(String, String)] = []
    if let key = item.keyDisplay { rows.append(("Key", key)) }
    if let tempo = item.tempoDisplay { rows.append(("Tempo", tempo)) }
    return rows
  }

  private var tags: some View {
    ScrollView(.horizontal, showsIndicators: false) {
      HStack(spacing: IntradaSpacing.controlGap) {
        ForEach(item.tags, id: \.self) { tag in
          TagChip(tag, style: .outlined)
        }
      }
    }
  }
}

private struct DetailRow: View {
  let label: String
  let value: String

  var body: some View {
    HStack {
      Text(label)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkSecondary)
      Spacer(minLength: 16)
      Text(value)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.trailing)
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.card)
    .accessibilityElement(children: .combine)
    .accessibilityLabel("\(label), \(value)")
  }
}

/// Normal-mode row: exercise type bar + title + key/tempo meta + trailing score ring.
private struct LinkedExerciseRow: View {
  let exercise: LinkedExerciseView

  var body: some View {
    HStack(spacing: IntradaSpacing.row) {
      // spacing: 3 — tight title/meta baseline gap, below the token scale floor.
      VStack(alignment: .leading, spacing: 3) {
        Text(exercise.title)
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        if let meta = metaLine {
          Text(meta)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      ScoreRing(score: exercise.practice?.latestScore.map(Int.init), size: 44)
    }
    .padding(.vertical, IntradaSpacing.row)
    .padding(.leading, 20)
    .padding(.trailing, IntradaSpacing.card)
    .background(IntradaColor.cardFill)
    .overlay(alignment: .leading) {
      ItemKind.exercise.bar.frame(width: 4)
    }
    .accessibilityElement(children: .combine)
    .accessibilityLabel(accessibilityLabel)
  }

  private var metaLine: String? {
    let parts = [exercise.key, exercise.tempo].compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }

  private var accessibilityLabel: String {
    var parts = ["Exercise", exercise.title]
    if let meta = metaLine { parts.append(meta) }
    if let score = exercise.practice?.latestScore {
      parts.append("Score \(score) of 10")
    } else {
      parts.append("Not yet rated")
    }
    return parts.joined(separator: ", ")
  }
}

/// Edit-mode row: remove button + title + meta + up/down move buttons (VoiceOver-accessible reorder).
private struct LinkedExerciseEditRow: View {
  let exercise: LinkedExerciseView
  let isFirst: Bool
  let isLast: Bool
  let onRemove: () -> Void
  let onMoveUp: () -> Void
  let onMoveDown: () -> Void

  var body: some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      Button(action: onRemove) {
        Image(systemName: "minus.circle.fill")
          .imageScale(.medium)
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.danger)
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Remove \(exercise.title)")
      // spacing: 3 — tight title/meta baseline gap, below the token scale floor.
      VStack(alignment: .leading, spacing: 3) {
        Text(exercise.title)
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        if let meta = metaLine {
          Text(meta)
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      VStack(spacing: 0) {
        Button(action: onMoveUp) {
          Image(systemName: "chevron.up")
            .imageScale(.small)
            .font(IntradaFont.meta)
            .foregroundStyle(isFirst ? IntradaColor.inkFaint : IntradaColor.inkSecondary)
        }
        .buttonStyle(.plain)
        .disabled(isFirst)
        .accessibilityLabel("Move \(exercise.title) up")
        Button(action: onMoveDown) {
          Image(systemName: "chevron.down")
            .imageScale(.small)
            .font(IntradaFont.meta)
            .foregroundStyle(isLast ? IntradaColor.inkFaint : IntradaColor.inkSecondary)
        }
        .buttonStyle(.plain)
        .disabled(isLast)
        .accessibilityLabel("Move \(exercise.title) down")
      }
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.card)
    .background(IntradaColor.cardFill)
  }

  private var metaLine: String? {
    let parts = [exercise.key, exercise.tempo].compactMap { $0 }.filter { !$0.isEmpty }
    return parts.isEmpty ? nil : parts.joined(separator: " · ")
  }
}

#if DEBUG
  #Preview("Piece") {
    NavigationStack {
      LibraryDetailScreen(item: .previewDetail)
    }
    .environment(Store.preview)
  }

  #Preview("Minimal") {
    NavigationStack {
      LibraryDetailScreen(item: .previewMinimal)
    }
    .environment(Store.preview)
  }

  #Preview("Linked — populated") {
    NavigationStack {
      LibraryDetailScreen(item: .previewDetailWithLinkedExercises)
    }
    .environment(Store.previewDetailLinkedPopulated)
  }

  #Preview("Linked — empty") {
    NavigationStack {
      LibraryDetailScreen(item: .previewDetailLinkedEmpty)
    }
    .environment(Store.previewDetailLinkedEmpty)
  }

  #Preview("Exercise — Linked from") {
    NavigationStack {
      LibraryDetailScreen(item: .previewExerciseWithLinkedFrom)
    }
    .environment(Store.previewExerciseLinkedFrom)
  }

  /// Snapshot seed: renders the detail screen with editingLinks already on,
  /// so the test can capture the edit-mode row layout without UI interaction.
  struct EditingLinkedExercisesWrapper: View {
    let item: LibraryItemView
    var body: some View {
      NavigationStack {
        LibraryDetailScreen(item: item, startEditingLinks: true)
      }
    }
  }
#endif
