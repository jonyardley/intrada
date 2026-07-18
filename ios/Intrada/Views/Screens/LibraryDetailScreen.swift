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
  @State private var editingSteps: Bool
  @State private var showingPicker = false
  @State private var editingChart = false
  @State private var showingScaffold = false

  init(item: LibraryItemView, startEditingLinks: Bool = false, startEditingSteps: Bool = false) {
    self.item = item
    _editingLinks = State(initialValue: startEditingLinks)
    _editingSteps = State(initialValue: startEditingSteps)
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

          if item.itemType == .exercise {
            stepsSection
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
            chordChartSection
          }

          if item.itemType == .piece {
            linkedExercisesSection
          }

          if item.itemType == .exercise, !item.exerciseContexts.isEmpty {
            byPieceSection
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

  // ── Chord chart ──

  private var chordChartSection: some View {
    VStack(spacing: 0) {
      HStack {
        Text("Chord chart")
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        Spacer()
        Button(item.chordChart == nil ? "Add" : "Edit") { editingChart = true }
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
          .accessibilityLabel(item.chordChart == nil ? "Add a chord chart" : "Edit chord chart")
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.card)
      .padding(.bottom, item.chordChart == nil ? IntradaSpacing.card : IntradaSpacing.cardCompact)

      if let chart = item.chordChart {
        chartSubtitle(chart)
        chartBarGrid(chart)
        seeCurriculumButton
      } else {
        chartEmptyState
      }
    }
    .cardSurface()
  }

  private func chartSubtitle(_ chart: ChordChart) -> some View {
    let bars = chart.sections.reduce(0) { $0 + $1.bars.count }
    let changes = chart.sections.reduce(0) { $0 + $1.bars.reduce(0) { $0 + $1.chords.count } }
    let key = item.keyDisplay ?? chart.key
    return Text("\(key) · \(bars) \(bars == 1 ? "bar" : "bars") · \(changes) changes")
      .font(IntradaFont.meta)
      .foregroundStyle(IntradaColor.inkSecondary)
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.bottom, IntradaSpacing.cardCompact)
  }

  private func chartBarGrid(_ chart: ChordChart) -> some View {
    let columns = Array(repeating: GridItem(.flexible(), spacing: 6), count: 4)
    return VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      ForEach(Array(chart.sections.enumerated()), id: \.offset) { _, section in
        if let label = section.label, !label.isEmpty {
          Eyebrow(label)
        }
        LazyVGrid(columns: columns, spacing: 6) {
          ForEach(Array(sectionChords(section).enumerated()), id: \.offset) { _, raw in
            Text(raw)
              .font(IntradaFont.cardTitle())
              .foregroundStyle(IntradaColor.ink)
              .lineLimit(1)
              .minimumScaleFactor(0.7)
              .frame(maxWidth: .infinity)
              .padding(.vertical, IntradaSpacing.controlGap)
              .padding(.horizontal, 4)
              .background(
                RoundedRectangle(cornerRadius: IntradaRadius.badge)
                  .fill(IntradaColor.paperTop)
                  .stroke(IntradaColor.divider, lineWidth: 1)
              )
          }
        }
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.bottom, IntradaSpacing.cardCompact)
    .accessibilityElement(children: .combine)
    .accessibilityLabel(
      "Chord chart: " + chart.sections.flatMap { sectionChords($0) }.joined(separator: ", "))
  }

  private func sectionChords(_ section: ChartSection) -> [String] {
    section.bars.flatMap { $0.chords.map { $0.symbol.raw } }
  }

  private var seeCurriculumButton: some View {
    BrandBarButton(action: { showingScaffold = true }) {
      Image(systemName: "sparkles")
      Text("See the curriculum")
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.bottom, IntradaSpacing.card)
    .accessibilityLabel("See the derived curriculum")
    .accessibilityHint("Shows the exercises derived from these changes")
  }

  private var chartEmptyState: some View {
    Text("Paste the changes to see the exercises they imply.")
      .font(IntradaFont.body)
      .foregroundStyle(IntradaColor.inkSecondary)
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.bottom, IntradaSpacing.card)
  }

  // ── Related exercises ──

  private var linkedExercisesSection: some View {
    VStack(spacing: 0) {
      linkedExercisesHeader
      if item.linkedExercises.isEmpty {
        linkedExercisesEmptyState
      } else {
        if !editingLinks {
          // The rings below are each exercise's score *on this piece*, not its
          // overall — say so, mirroring the exercise hero's "Overall" (#1087 B2).
          Text("Scores shown are for this piece")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.bottom, IntradaSpacing.cardCompact)
        }
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
          onMoveDown: { moveExercise(at: index, by: 1) },
          onRemove: { removeExercise(id: exercise.id) })
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
      VStack(spacing: 6) {
        ScoreRing(
          score: item.practice?.latestScore.map(Int.init), size: 132, showsScale: true)
        // Names the hero as the score across every piece, so it can't be read as
        // one piece's — the distinction the "By piece" rows below make (#1087 B2).
        if !item.exerciseContexts.isEmpty {
          Eyebrow("Overall")
        }
      }
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

  // ── Steps (exercise step ladder) ──

  private var stepsSection: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      stepsHeader
      if item.variants.isEmpty {
        stepsEmptyState
      } else if editingSteps {
        VStack(spacing: 0) {
          ForEach(Array(item.variants.enumerated()), id: \.element.id) { index, step in
            if index > 0 {
              HairlineDivider()
            }
            StepEditRow(
              step: step,
              onRename: { renameStep(id: step.id, to: $0) },
              onMoveUp: { moveStep(id: step.id, by: -1) },
              onMoveDown: { moveStep(id: step.id, by: 1) },
              onRemove: { removeStep(id: step.id) },
              onDrop: { droppedId in moveStep(id: droppedId, before: step.id) })
          }
        }
        .cardSurface()
      } else {
        VStack(spacing: 0) {
          ForEach(Array(item.variants.enumerated()), id: \.offset) { index, step in
            if index > 0 {
              HairlineDivider()
            }
            StepRow(step: step)
          }
        }
        .cardSurface()
      }
    }
    .onChange(of: item.variants.isEmpty) { _, isEmpty in
      if isEmpty { editingSteps = false }
    }
  }

  private var stepsHeader: some View {
    HStack(alignment: .firstTextBaseline) {
      Eyebrow("Steps")
      Spacer()
      if !item.variants.isEmpty {
        Button(editingSteps ? "Done" : "Edit") {
          editingSteps.toggle()
        }
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.accent)
        .accessibilityLabel(editingSteps ? "Done editing steps" : "Edit steps")
      }
    }
  }

  private var stepsEmptyState: some View {
    VStack(spacing: IntradaSpacing.controlGap) {
      AddRowButton(title: "Add 12 major keys") { addKeyPreset(KeyHelper.circleMajor) }
        .accessibilityLabel("Add 12 major keys as this exercise's step ladder")
      AddRowButton(title: "Add 12 minor keys") { addKeyPreset(KeyHelper.circleMinor) }
        .accessibilityLabel("Add 12 minor keys as this exercise's step ladder")
    }
    .padding(IntradaSpacing.card)
    .cardSurface()
  }

  private func addKeyPreset(_ labels: [String]) {
    let before = store.viewModel?.errorSeq
    store.send(.item(.setVariants(id: item.id, labels: labels)))
    if store.viewModel?.errorSeq == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  private func renameStep(id: String, to newLabel: String) {
    store.send(.item(.renameVariant(itemId: item.id, variantId: id, newLabel: newLabel)))
  }

  private func moveStep(id: String, by delta: Int) {
    var ids = item.variants.map(\.id)
    guard let index = ids.firstIndex(of: id) else { return }
    let dest = index + delta
    guard dest >= 0, dest < ids.count else { return }
    ids.swapAt(index, dest)
    reorderSteps(ids)
  }

  // Drop-onto-a-row reorder: move the dragged step to just before the target.
  private func moveStep(id: String, before targetId: String) {
    guard id != targetId else { return }
    var ids = item.variants.map(\.id)
    guard ids.contains(id), let sourceIndex = ids.firstIndex(of: id) else { return }
    ids.remove(at: sourceIndex)
    let insertIndex = ids.firstIndex(of: targetId) ?? ids.count
    ids.insert(id, at: insertIndex)
    reorderSteps(ids)
  }

  // Reordering existing labels resolves by label match (core-tested), so no
  // labels actually change — only position — and every id/score history is
  // preserved.
  private func reorderSteps(_ orderedIds: [String]) {
    let labelsById = Dictionary(uniqueKeysWithValues: item.variants.map { ($0.id, $0.label) })
    let labels = orderedIds.compactMap { labelsById[$0] }
    let before = store.viewModel?.errorSeq
    store.send(.item(.setVariants(id: item.id, labels: labels)))
    if store.viewModel?.errorSeq == before {
      UISelectionFeedbackGenerator().selectionChanged()
    }
  }

  private func removeStep(id: String) {
    let labels = item.variants.filter { $0.id != id }.map(\.label)
    let before = store.viewModel?.errorSeq
    store.send(.item(.setVariants(id: item.id, labels: labels)))
    if store.viewModel?.errorSeq == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  // ── By piece (exercise contexts) ──

  // Per-piece score breakdown derived from session blocks (#1087 B2): where this
  // drill has done its work and how it scores there. Gated on non-empty upstream.
  private var byPieceSection: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: "By piece")
      VStack(spacing: 0) {
        ForEach(Array(item.exerciseContexts.enumerated()), id: \.offset) { index, context in
          if index > 0 {
            HairlineDivider()
          }
          byPieceRow(context)
        }
      }
      .cardSurface()
    }
  }

  @ViewBuilder private func byPieceRow(_ context: ExerciseContextView) -> some View {
    // A live piece taps through; the "On its own" bucket and since-removed pieces
    // (#1093, 2a) are inert rows — nowhere to navigate.
    if let piece = context.piece, !context.pieceRemoved {
      NavigationLink(value: piece.id) {
        ByPieceRow(context: context, locale: locale, calendar: calendar, discloses: true)
      }
      .buttonStyle(.plain)
    } else {
      ByPieceRow(context: context, locale: locale, calendar: calendar, discloses: false)
    }
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

  private func commitScaffold(_ kinds: Swift.Set<ScaffoldKind>) {
    guard !kinds.isEmpty else { return }
    // Optimistic UI reconciles with the core's confirmed outcome — only fire the
    // success haptic when no error was surfaced (surface-don't-swallow).
    let before = store.viewModel?.errorSeq
    store.send(.item(.commitScaffold(pieceId: item.id, kinds: Array(kinds))))
    if store.viewModel?.errorSeq == before {
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

  private func removeExercise(id: String) {
    let before = store.viewModel?.errorSeq
    store.send(.item(.unlinkExercise(pieceId: item.id, exerciseId: id)))
    if store.viewModel?.errorSeq == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
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
      // The score *on this piece* (#1087 B2), not the exercise's flat overall —
      // the section caption tells the reader which. Unrated until practised here.
      ScoreRing(score: exercise.pieceContextScore.map(Int.init), size: 44)
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
    if let score = exercise.pieceContextScore {
      parts.append("Score \(score) of 10 on this piece")
    } else {
      parts.append("Not yet rated on this piece")
    }
    return parts.joined(separator: ", ")
  }
}

private struct StepRow: View {
  let step: VariantView

  var body: some View {
    HStack(spacing: IntradaSpacing.row) {
      ScoreRing(score: step.latestScore.map(Int.init), size: 44, solid: step.isSolid)
      VStack(alignment: .leading, spacing: 3) {
        Text(step.label)
          .font(IntradaFont.cardTitle())
          .foregroundStyle(IntradaColor.ink)
        Text("Step \(step.position + 1)")
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .frame(maxWidth: .infinity, alignment: .leading)
    }
    .padding(.vertical, IntradaSpacing.row)
    .padding(.horizontal, IntradaSpacing.card)
    .background(IntradaColor.cardFill)
    .accessibilityElement(children: .combine)
    .accessibilityLabel(accessibilityLabel)
  }

  private var accessibilityLabel: String {
    var parts = [step.label]
    if let score = step.latestScore {
      parts.append("score \(score) of 10")
    } else {
      parts.append("not yet rated")
    }
    if step.isSolid { parts.append("Solid") }
    return parts.joined(separator: ", ")
  }
}

/// Edit-mode row: drag handle (native drag reorder) + inline rename field +
/// remove button. VoiceOver gets move-up/move-down actions since a drag
/// gesture alone isn't screen-reader-operable.
private struct StepEditRow: View {
  let step: VariantView
  let onRename: (String) -> Void
  let onMoveUp: () -> Void
  let onMoveDown: () -> Void
  let onRemove: () -> Void
  let onDrop: (String) -> Void

  @State private var label: String

  init(
    step: VariantView, onRename: @escaping (String) -> Void, onMoveUp: @escaping () -> Void,
    onMoveDown: @escaping () -> Void, onRemove: @escaping () -> Void,
    onDrop: @escaping (String) -> Void
  ) {
    self.step = step
    self.onRename = onRename
    self.onMoveUp = onMoveUp
    self.onMoveDown = onMoveDown
    self.onRemove = onRemove
    self.onDrop = onDrop
    _label = State(initialValue: step.label)
  }

  var body: some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      Image(systemName: "line.3.horizontal")
        .imageScale(.small)
        .foregroundStyle(IntradaColor.inkFaint)
        .accessibilityLabel("Reorder \(step.label)")
        .accessibilityHint("Drag to change this step's position")
        .accessibilityAction(named: "Move up", onMoveUp)
        .accessibilityAction(named: "Move down", onMoveDown)
        .draggable(step.id)
      TextField("Step label", text: $label)
        .font(IntradaFont.cardTitle())
        .foregroundStyle(IntradaColor.ink)
        .onChange(of: label) { _, value in
          let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
          guard !trimmed.isEmpty, trimmed != step.label else { return }
          onRename(trimmed)
        }
      Button(action: onRemove) {
        Image(systemName: "minus.circle")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.danger)
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Remove \(step.label) from steps")
    }
    .padding(.vertical, IntradaSpacing.cardCompact)
    .padding(.horizontal, IntradaSpacing.card)
    .background(IntradaColor.cardFill)
    .dropDestination(for: String.self) { items, _ in
      guard let droppedId = items.first else { return false }
      onDrop(droppedId)
      return true
    }
  }
}

private struct ByPieceRow: View {
  let context: ExerciseContextView
  let locale: Locale
  let calendar: Calendar
  let discloses: Bool

  private var isStandalone: Bool { context.piece == nil }

  var body: some View {
    HStack(spacing: IntradaSpacing.row) {
      ScoreRing(score: context.latestScore.map(Int.init), size: 44)
      VStack(alignment: .leading, spacing: 3) {
        Text(context.contextTitle)
          .font(isStandalone ? IntradaFont.bodyMedium : IntradaFont.cardTitle())
          .foregroundStyle(context.pieceRemoved ? IntradaColor.inkSecondary : IntradaColor.ink)
        Text(context.metaLine(locale: locale, calendar: calendar))
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      if discloses {
        Image(systemName: "chevron.right")
          .imageScale(.small)
          .foregroundStyle(IntradaColor.inkFaint)
          .accessibilityHidden(true)
      }
    }
    .padding(.vertical, IntradaSpacing.row)
    .padding(.horizontal, IntradaSpacing.card)
    .background(IntradaColor.cardFill)
    .accessibilityElement(children: .combine)
    .accessibilityLabel(accessibilityLabel)
    .accessibilityAddTraits(discloses ? [.isButton] : [])
  }

  private var accessibilityLabel: String {
    var parts = [context.contextTitle]
    if context.pieceRemoved { parts.append("removed from your library") }
    if let score = context.latestScore {
      parts.append("score \(score) of 10")
    } else {
      parts.append("not yet rated")
    }
    let n = Int(context.sessionCount)
    parts.append("\(n) \(n == 1 ? "session" : "sessions")")
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
  let onRemove: () -> Void

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
      HStack(spacing: IntradaSpacing.controlGap) {
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
        Button(action: onRemove) {
          Image(systemName: "minus.circle")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.danger)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Remove \(exercise.title) from related exercises")
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

  /// Snapshot seed: renders the detail screen with editingSteps already on,
  /// so the test can capture the Steps edit-mode row layout without UI
  /// interaction.
  struct EditingStepsWrapper: View {
    let item: LibraryItemView
    var body: some View {
      NavigationStack {
        LibraryDetailScreen(item: item, startEditingSteps: true)
      }
    }
  }
#endif
