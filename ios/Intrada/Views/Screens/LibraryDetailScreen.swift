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
  @State private var editingSection: SectionSheetTarget?
  @State private var choosingKeys = false
  @State private var choosingVariations = false
  @State private var choosingSectionsFor: SectionLinkTarget?

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
        onApply: { ids, _ in
          applyPieceLinkChanges(ids)
          return .accepted
        }
      )
      .environment(store)
    }
    .sheet(isPresented: $editingChart) {
      ChordChartEditSheet(
        pieceId: item.id, pieceKey: item.key,
        existingChart: item.chordChart
      )
      .environment(store)
    }
    .sheet(isPresented: $choosingKeys) {
      KeysSheet(item: item)
        .environment(store)
    }
    .sheet(isPresented: $choosingVariations) {
      VariationsSheet(item: item)
        .environment(store)
    }
    .sheet(item: $choosingSectionsFor) { target in
      if let exercise = item.linkedExercises.first(where: { $0.id == target.id }) {
        LinkSectionsSheet(piece: item, exercise: exercise)
          .environment(store)
      }
    }
    .sheet(item: $editingSection) { target in
      SectionSheet(item: item, target: target)
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
      SectionTitle("Notes")
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
      sessions: item.practice?.recentSessionRows(locale: locale, calendar: calendar) ?? [],
      trend: item.practice?.scoreTrend)
  }

  // ── Actions ──

  private var library: [LibraryItemView] { store.libraryRows }

  /// The pieces that declare the link, as opposed to the ones this exercise has
  /// merely been practised alongside — only a declared link can be unticked.
  private var linkedPieceIds: [String] {
    item.usedIn.filter { $0.linked }.compactMap { $0.piece?.id }
  }

  private func linkPiece(id: String) {
    let targets = item.exerciseTargets(pieceIds: linkedPieceIds + [id])
    store.send(.item(.setExerciseLinks(exerciseId: item.id, targets: targets)), onSuccess: .success)
  }

  private func applyPieceLinkChanges(_ selected: Swift.Set<String>) {
    guard selected != Swift.Set(linkedPieceIds) else { return }
    let kept = linkedPieceIds.filter(selected.contains)
    let added = library.map(\.id).filter { selected.contains($0) && !kept.contains($0) }
    let targets = item.exerciseTargets(pieceIds: kept + added)
    store.send(.item(.setExerciseLinks(exerciseId: item.id, targets: targets)), onSuccess: .success)
  }

  // The whole set in one event (#2232): kept exercises keep their sections,
  // a newly chosen one links to the whole piece, and each draft is created and
  // linked. A refusal saves nothing, so every draft stays staged (#2224).
  private func applyLinkChanges(_ selected: Swift.Set<String>, _ drafts: [StagedExercise])
    -> LinkApplyOutcome
  {
    let row = liveRow
    let current = row.linkedExercises.map(\.id)
    let kept = row.linkedExercises.filter { selected.contains($0.id) }.flatMap(\.linkEdits)
    let added = library.map(\.id)
      .filter { selected.contains($0) && !current.contains($0) }
      .map { LinkEdit(exercise: .existing(id: $0), sectionId: nil) }
    let written = drafts.compactMap { draft -> LinkEdit? in
      guard case .new = draft.entry else { return nil }
      return LinkEdit(exercise: draft.entry, sectionId: nil)
    }
    let unchanged = Swift.Set(current) == selected && written.isEmpty
    if unchanged { return .accepted }
    let event = Event.item(.setPieceLinks(pieceId: item.id, links: kept + added + written))
    guard store.sendAccepted(event) else {
      return .refused(
        message: store.viewModel?.error ?? "Couldn't save. Try again.",
        remainingDrafts: drafts, nowLinked: Swift.Set(current))
    }
    Haptic.success.play()
    return .accepted
  }

  // `item` is the value this screen was built with; a send made a moment ago
  // has already reached the store's rows but not this copy.
  private var liveRow: LibraryItemView {
    store.libraryRows.first { $0.id == item.id } ?? item
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

        if item.itemType == .piece {
          sectionsSection
          variationsSection
        }

        if item.itemType == .exercise {
          variationsSection
          sectionsSection
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
            onShowSuggestions: { showingScaffold = true },
            onChooseSections: { choosingSectionsFor = SectionLinkTarget(id: $0.id) })
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

  private var variationsSection: some View {
    VariationsSection(
      item: item, onChooseKeys: { choosingKeys = true },
      onChooseVariations: { choosingVariations = true })
  }

  private var sectionsSection: some View {
    SectionsSection(
      item: item, onAdd: { editingSection = .new },
      onEdit: { editingSection = .existing($0) }
    )
    .id(item.id)
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
            title: item.title, kind: item.itemType, composer: nil, key: nil,
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
      FieldLabel(label)
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

private struct SectionLinkTarget: Identifiable {
  let id: String
}
