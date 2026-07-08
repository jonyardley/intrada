import SharedTypes
import SwiftUI

/// Detail for a library item: type badge, key/tempo, notes, tags, and delete.
struct LibraryDetailScreen: View {
  let item: LibraryItemView

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @Environment(\.locale) private var locale
  @Environment(\.calendar) private var calendar
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
          if item.itemType == .exercise {
            exerciseHero
          } else {
            TypeBadge(kind: item.itemType)
          }

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

          if hasRecentSessions {
            recentSessionsSection
          }

          if item.itemType == .exercise {
            practiseButton
              .padding(.top, IntradaSpacing.controlGap)
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
        available: allExercises,
        linkedIds: item.linkedExercises.map(\.id),
        onApply: { applyLinkChanges($0) }
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

  // ── Related exercises ──

  private var linkedExercisesSection: some View {
    VStack(spacing: 0) {
      linkedExercisesHeader
      if item.linkedExercises.isEmpty {
        linkedExercisesEmptyState
      } else {
        linkedExercisesRows
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
      Text("Related exercises")
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
      .accessibilityLabel(
        editingLinks ? "Done editing related exercises" : "Edit related exercises")
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
      Text("No related exercises yet")
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.ink)
      Text("Add scales, arpeggios, or any exercise you practise alongside this piece.")
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkSecondary)
        .multilineTextAlignment(.center)
      AddRowButton(title: "Add a related exercise") {
        showingPicker = true
      }
      .padding(.top, IntradaSpacing.controlGap)
      .accessibilityLabel("Add a related exercise to this piece")
    }
    .frame(maxWidth: .infinity)
    .padding(IntradaSpacing.card)
    .padding(.bottom, IntradaSpacing.cardCompact)
  }

  private var linkExerciseButton: some View {
    AddRowButton(title: "Add a related exercise") {
      showingPicker = true
    }
    .accessibilityLabel("Add a related exercise to this piece")
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.vertical, IntradaSpacing.cardCompact)
  }

  // ── Exercise hero + provenance ──

  private var exerciseHero: some View {
    VStack(spacing: IntradaSpacing.cardCompact) {
      ScoreRing(
        score: item.practice?.latestScore.map(Int.init), size: 132, showsScale: true)
      relatedBreadcrumb
    }
    .frame(maxWidth: .infinity)
    .padding(.vertical, IntradaSpacing.controlGap)
  }

  @ViewBuilder private var relatedBreadcrumb: some View {
    if let first = item.linkedFromPieces.first {
      if item.linkedFromPieces.count == 1 {
        NavigationLink(value: first.id) {
          breadcrumbRow(first, discloses: false)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(breadcrumbAccessibility(first))
      } else {
        Menu {
          ForEach(item.linkedFromPieces, id: \.id) { piece in
            NavigationLink(value: piece.id) {
              Text(piece.title)
              if let subtitle = piece.subtitle {
                Text(subtitle)
              }
            }
          }
        } label: {
          breadcrumbRow(first, discloses: true)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(breadcrumbAccessibility(first, discloses: true))
        .accessibilityHint("Choose a piece to open")
      }
    }
  }

  private func breadcrumbRow(_ piece: PieceRefView, discloses: Bool) -> some View {
    HStack(spacing: 5) {
      Image(systemName: "arrow.turn.down.right")
        .imageScale(.small)
        .accessibilityHidden(true)
      breadcrumbLabel(piece)
      if discloses {
        Image(systemName: "chevron.down")
          .imageScale(.small)
          .accessibilityHidden(true)
      }
    }
    .foregroundStyle(IntradaColor.exerciseBadgeFg)
  }

  private func breadcrumbLabel(_ piece: PieceRefView) -> Text {
    let extra = item.linkedFromPieces.count - 1
    let base =
      Text("Related to ").font(IntradaFont.metaMedium) + Text(piece.title).font(IntradaFont.badge)
    return extra > 0 ? base + Text(" · +\(extra) more").font(IntradaFont.metaMedium) : base
  }

  private func breadcrumbAccessibility(_ piece: PieceRefView, discloses: Bool = false) -> String {
    let extra = item.linkedFromPieces.count - 1
    let others = extra > 0 ? " and \(extra) more \(extra == 1 ? "piece" : "pieces")" : ""
    let role = discloses ? "" : ", related piece"
    return "Related to \(piece.title)\(others)\(role)"
  }

  // One-tap into the session builder seeded with this exercise (core
  // StartBuildingWith); RootView switches to the Practice tab when
  // `buildingSetlist` goes non-nil.
  private var practiseButton: some View {
    BrandBarButton(action: practiseThis) {
      Image(systemName: "timer")
      Text("Practise this")
    }
    .accessibilityLabel("Practise this exercise")
    .accessibilityHint("Starts a session plan with this exercise")
  }

  private func practiseThis() {
    let before = store.viewModel?.errorSeq
    store.send(.session(.startBuildingWith(itemId: item.id)))
    if store.viewModel?.errorSeq == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  // ── Recent sessions ──

  private var hasRecentSessions: Bool {
    !(item.practice?.scoreHistory.isEmpty ?? true)
  }

  private var recentSessionsSection: some View {
    RecentSessions(
      sessions: item.practice?.recentSessionRows(locale: locale, calendar: calendar) ?? [])
  }

  // ── Actions ──

  private var allExercises: [LibraryItemView] {
    (store.viewModel?.items ?? []).filter { $0.itemType == .exercise }
  }

  // Reconcile the picker's final set against what's linked now: link the added,
  // unlink the removed. A failed write surfaces on the global banner (#846), so
  // the success haptic only fires when every write landed.
  private func applyLinkChanges(_ selected: Swift.Set<String>) {
    let current = Swift.Set(item.linkedExercises.map(\.id))
    let toLink = selected.subtracting(current)
    let toUnlink = current.subtracting(selected)
    var ok = true
    for id in toLink {
      let before = store.viewModel?.errorSeq
      store.send(.item(.linkExercise(pieceId: item.id, exerciseId: id)))
      if store.viewModel?.errorSeq != before { ok = false }
    }
    for id in toUnlink {
      let before = store.viewModel?.errorSeq
      store.send(.item(.unlinkExercise(pieceId: item.id, exerciseId: id)))
      if store.viewModel?.errorSeq != before { ok = false }
    }
    if ok && !(toLink.isEmpty && toUnlink.isEmpty) {
      UINotificationFeedbackGenerator().notificationOccurred(.success)
    }
  }

  private func moveExercise(at index: Int, by delta: Int) {
    var ids = item.linkedExercises.map(\.id)
    let dest = index + delta
    guard dest >= 0, dest < ids.count else { return }
    ids.swapAt(index, dest)
    let before = store.viewModel?.errorSeq
    store.send(.item(.reorderLinkedExercises(pieceId: item.id, orderedIds: ids)))
    if store.viewModel?.errorSeq == before {
      UISelectionFeedbackGenerator().selectionChanged()
    }
  }

  private var deleteButton: some View {
    DeleteButton(title: "Delete \(item.itemType.label.lowercased())") {
      confirmingDelete = true
    }
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
  let onMoveUp: () -> Void
  let onMoveDown: () -> Void

  var body: some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
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

  #Preview("Related — populated") {
    NavigationStack {
      LibraryDetailScreen(item: .previewDetailWithLinkedExercises)
    }
    .environment(Store.previewDetailLinkedPopulated)
  }

  #Preview("Related — empty") {
    NavigationStack {
      LibraryDetailScreen(item: .previewDetailLinkedEmpty)
    }
    .environment(Store.previewDetailLinkedEmpty)
  }

  #Preview("Exercise — Related pieces") {
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
