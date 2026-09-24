import SharedTypes
import SwiftUI

/// Detail for a library item: type badge, key/tempo, notes, tags, and delete.
struct LibraryDetailScreen: View {
  let item: LibraryItemView
  /// False only for the iPad split view's own detail pane (#1724): selection
  /// there replaces the pane in place rather than pushing, so there is
  /// nothing to pop back from. A related item pushed within that same pane
  /// (`LibrarySplitView`'s own `navigationDestination`) still gets one.
  var showsBackButton: Bool

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @Environment(\.locale) private var locale
  @Environment(\.calendar) private var calendar
  @State private var confirmingDelete = false
  @State private var editing = false
  @State private var editingLinks: Bool
  @State private var showingPicker = false
  @State private var showingPiecePicker = false
  @State private var editingChart = false
  @State private var showingScaffold = false

  init(item: LibraryItemView, showsBackButton: Bool = true, startEditingLinks: Bool = false) {
    self.item = item
    self.showsBackButton = showsBackButton
    _editingLinks = State(initialValue: startEditingLinks)
  }

  var body: some View {
    ScreenScaffold(
      title: item.title, subtitle: subtitle,
      trailingContent: { starAndEditActions },
      content: { detailContent }
    )
    .navigationBarBackButtonHidden(!showsBackButton)
    .sheet(isPresented: $editing) {
      LibraryEditScreen(item: item)
        .environment(store)
    }
    .sheet(isPresented: $showingPicker) {
      LinkedItemPickerSheet(
        kind: .exercise,
        library: library,
        linkedIds: item.linkedExercises.map(\.id),
        onApply: applyLinkChanges
      )
      .environment(store)
    }
    .sheet(isPresented: $showingPiecePicker) {
      LinkedItemPickerSheet(
        kind: .piece,
        library: library,
        linkedIds: linkedPieceIds,
        onApply: { ids, _ in applyPieceLinkChanges(ids) }
      )
      .environment(store)
    }
    .sheet(isPresented: $editingChart) {
      ChordChartEditSheet(
        pieceId: item.id, pieceKey: item.key, pieceModality: item.modality,
        existingChart: item.chordChart
      )
      .environment(store)
    }
    .sheet(isPresented: $showingScaffold) {
      if let preview = item.scaffoldPreview {
        ScaffoldPreviewSheet(preview: preview, onCommit: commitScaffold)
      }
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

  // ── Notes ──

  private func notesSection(_ notes: String) -> some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Eyebrow("Notes")
      Text(notes)
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(maxWidth: .infinity, alignment: .leading)
    }
    .padding(IntradaSpacing.card)
    .cardSurface()
  }

  // ── Used in (pieces this exercise serves) ──

  private var usedInSection: some View {
    UsedInCard(
      usage: item.usedIn, locale: locale, calendar: calendar,
      onLink: { linkPiece(id: $0) },
      onLinkAPiece: { showingPiecePicker = true })
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
    store.send(.session(.startBuildingWith(itemId: item.id)), onSuccess: .impact)
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

  private var library: [LibraryItemView] { store.viewModel?.items ?? [] }

  /// The pieces that declare the link, as opposed to the ones this exercise has
  /// merely been practised alongside — only a declared link can be unticked.
  private var linkedPieceIds: [String] {
    item.usedIn.filter { $0.linked }.compactMap { $0.piece?.id }
  }

  private func linkPiece(id: String) {
    store.send(.item(.linkExercise(pieceId: id, exerciseId: item.id)), onSuccess: .success)
  }

  private func applyPieceLinkChanges(_ selected: Swift.Set<String>) {
    let current = Swift.Set(linkedPieceIds)
    let toLink = selected.subtracting(current)
    let toUnlink = current.subtracting(selected)
    var ok = true
    for pieceId in toLink {
      if !store.sendAccepted(.item(.linkExercise(pieceId: pieceId, exerciseId: item.id))) {
        ok = false
      }
    }
    for pieceId in toUnlink {
      if !store.sendAccepted(.item(.unlinkExercise(pieceId: pieceId, exerciseId: item.id))) {
        ok = false
      }
    }
    if ok && !(toLink.isEmpty && toUnlink.isEmpty) {
      Haptic.success.play()
    }
  }

  // Links, unlinks and creates+links each draft (#1431); the haptic fires once
  // the core accepts them all, and a failed disk write arrives later on the
  // banner (#846, #2004).
  private func applyLinkChanges(_ selected: Swift.Set<String>, _ drafts: [StagedExercise]) {
    let current = Swift.Set(item.linkedExercises.map(\.id))
    let toLink = selected.subtracting(current)
    let toUnlink = current.subtracting(selected)
    var ok = true
    for id in toLink {
      if !store.sendAccepted(.item(.linkExercise(pieceId: item.id, exerciseId: id))) { ok = false }
    }
    for id in toUnlink {
      if !store.sendAccepted(.item(.unlinkExercise(pieceId: item.id, exerciseId: id))) {
        ok = false
      }
    }
    for draft in drafts {
      guard case .new(let input) = draft.entry else { continue }
      if !store.sendAccepted(.item(.addLinkedExercise(pieceId: item.id, input: input))) {
        ok = false
      }
    }
    if ok && !(toLink.isEmpty && toUnlink.isEmpty && drafts.isEmpty) {
      Haptic.success.play()
    }
  }

  private func commitScaffold(_ kinds: Swift.Set<ScaffoldKind>) {
    guard !kinds.isEmpty else { return }
    store.send(.item(.commitScaffold(pieceId: item.id, kinds: Array(kinds))), onSuccess: .success)
  }

  private var deleteButton: some View {
    DeleteButton(title: "Delete \(item.itemType.label.lowercased())") {
      confirmingDelete = true
    }
  }

  private var detailContent: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: IntradaSpacing.card) {
        if item.itemType == .exercise {
          ExerciseHero(item: item)
        }
        badgeRow

        if let notes = item.notes, !notes.isEmpty {
          notesSection(notes)
        }

        if item.itemType == .exercise {
          VariationsSection(item: item, onAddVariations: { editing = true })
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

        if let tempoTrend = item.practice?.tempoTrendDisplay(
          locale: locale, calendar: calendar)
        {
          TempoTrend(display: tempoTrend)
        }

        if item.itemType == .exercise {
          practiseButton
            .padding(.top, IntradaSpacing.controlGap)
        }

        PhotoCard(itemId: item.id, photoId: item.photoId)

        if item.itemType == .piece {
          ChordChartCard(item: item, onEdit: { editingChart = true })
        }

        if item.itemType == .piece {
          RelatedExercisesCard(
            item: item, editing: $editingLinks,
            onAdd: { showingPicker = true },
            onShowSuggestions: { showingScaffold = true })
        }

        if item.itemType == .exercise {
          usedInSection
        }

        if hasRecentSessions {
          recentSessionsSection
        }

        deleteButton
          .padding(.top, IntradaSpacing.controlGap)
      }
      .padding(IntradaSpacing.card)
    }
    .scrollEdgeShadow()
  }

  private var starAndEditActions: some View {
    HStack(spacing: IntradaSpacing.card) {
      Button {
        toggleStar()
      } label: {
        Image(systemName: item.priority ? "star.fill" : "star")
      }
      .tint(item.priority ? IntradaColor.accent : IntradaColor.inkSecondary)
      .accessibilityLabel(item.priority ? "Remove from priorities" : "Add to priorities")

      Button("Edit") { editing = true }
        .accessibilityIdentifier("libraryDetail.edit")
    }
  }

  private func delete() {
    Haptic.warning.play()
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

  // An exercise with variations drops the item-level Key/Tempo rows: each one carries
  // its own target, so a single value here would be misleading (#1083 C2).
  private var detailRows: [(label: String, value: String)] {
    guard item.itemType != .exercise || item.variants.isEmpty else { return [] }
    var rows: [(String, String)] = []
    if let key = item.keyDisplay { rows.append(("Key", key)) }
    if let tempo = item.tempoDisplay { rows.append(("Tempo", tempo)) }
    return rows
  }

  private var badgeRow: some View {
    ScrollView(.horizontal, showsIndicators: false) {
      HStack(spacing: IntradaSpacing.controlGap) {
        TypeBadge(kind: item.itemType)
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
        .font(IntradaFont.metaMedium)
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
      LibraryDetailScreen(item: .previewExerciseLinkedOnly)
    }
    .environment(Store.previewExerciseLinkedOnlyStore)
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
