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
  @State private var creatingExercise = false
  @State private var editingChart = false
  @State private var showingScaffold = false
  @State private var showingAddSteps = false

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

          if let tempoTrend = item.practice?.tempoTrendDisplay(
            locale: locale, calendar: calendar)
          {
            TempoTrend(display: tempoTrend)
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

          PhotoCard(itemId: item.id, photoId: item.photoId)

          if item.itemType == .piece {
            chordChartSection
          }

          if item.itemType == .piece {
            linkedExercisesSection
          }

          if item.itemType == .exercise, !item.usedIn.isEmpty {
            usedInSection
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
    // The read belongs to the sheet. `onDismiss`, not the add screen's own
    // `onDisappear`, which the scanner's full-screen cover also triggers.
    .sheet(isPresented: $creatingExercise, onDismiss: { store.send(.discardPhotoDraft) }) {
      LibraryAddScreen(relatedToPieceId: item.id)
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
    .sheet(isPresented: $showingAddSteps) {
      AddStepsSheet(itemId: item.id)
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
        chartBarGrid(chart).padding(.bottom, IntradaSpacing.controlGap)
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

  private var chartEmptyState: some View {
    Text("Paste the changes to keep them with the piece.")
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
          Text("Marks shown are for this piece")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkSecondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.bottom, IntradaSpacing.cardCompact)
        }
        linkedExercisesRows
        populatedFooterActions
      }
      if item.scaffoldPreview != nil {
        HairlineDivider()
        suggestionsFromChart
      }
    }
    .cardSurface()
    .onChange(of: item.linkedExercises.isEmpty) { _, isEmpty in
      if isEmpty { editingLinks = false }
    }
  }

  // The brand bar belongs to the empty state, where there is one obvious next
  // step; once the card holds exercises both actions recede (T18).
  private var populatedFooterActions: some View {
    HStack(spacing: 0) {
      Button {
        creatingExercise = true
      } label: {
        Label("Create an exercise", systemImage: "plus")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
          .frame(maxWidth: .infinity)
          .padding(.vertical, IntradaSpacing.cardCompact)
          .contentShape(Rectangle())
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Create an exercise for this piece")
      Button {
        showingPicker = true
      } label: {
        Text("Choose one")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
          .frame(maxWidth: .infinity)
          .padding(.vertical, IntradaSpacing.cardCompact)
          .contentShape(Rectangle())
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Choose an existing exercise for this piece")
    }
    .padding(.horizontal, IntradaSpacing.controlGap)
    .padding(.vertical, IntradaSpacing.controlGap)
  }

  // Drawn as a control, not a section: read as plain content its eyebrow gets
  // mistaken for the button, and it creates several library items (T18).
  @ViewBuilder private var suggestionsFromChart: some View {
    if let preview = item.scaffoldPreview {
      VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
        Eyebrow("From the chord chart")
        Button {
          showingScaffold = true
        } label: {
          HStack(spacing: IntradaSpacing.cardCompact) {
            Image(systemName: "sparkles")
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.exerciseBadgeFg)
              .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 3) {
              Text(suggestionHeadline(preview))
                .font(IntradaFont.bodyMedium)
                .foregroundStyle(IntradaColor.ink)
                .multilineTextAlignment(.leading)
              Text(suggestionSubtitle(preview))
                .font(IntradaFont.meta)
                .foregroundStyle(IntradaColor.inkSecondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            Image(systemName: "chevron.right")
              .font(IntradaFont.meta)
              .foregroundStyle(IntradaColor.exerciseBadgeFg)
              .accessibilityHidden(true)
          }
          .padding(IntradaSpacing.cardCompact)
          .background(
            IntradaColor.surfaceSunken, in: RoundedRectangle(cornerRadius: IntradaRadius.card)
          )
          .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(suggestionAccessibilityLabel(preview))
        .accessibilityHint("Opens the exercises worked out from these changes")
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.cardCompact)
      .padding(.bottom, IntradaSpacing.card)
    }
  }

  /// Names the first two suggestions rather than counting them: what they are
  /// is the reason to tap, and a bare count says nothing about the music.
  private func suggestionHeadline(_ preview: ScaffoldPreviewView) -> String {
    let names = preview.specs.filter { !$0.alreadyLinked }.map(\.title)
    guard let first = names.first else { return "All of them are already added" }
    let shown = names.prefix(2).enumerated().map { index, title in
      index == 0 ? title : lowercasingFirstLetter(title)
    }
    let rest = names.count - shown.count
    if rest > 0 {
      return shown.joined(separator: ", ") + " and \(rest) more"
    }
    return shown.count == 2 ? shown.joined(separator: " and ") : first
  }

  /// "Shells, guide-tone lines and 3 more" — the run reads as one sentence, so
  /// every title after the first drops its capital (tone doc, sentence case).
  private func lowercasingFirstLetter(_ title: String) -> String {
    guard let first = title.first else { return title }
    return first.lowercased() + title.dropFirst()
  }

  private func suggestionSubtitle(_ preview: ScaffoldPreviewView) -> String {
    let added = preview.specs.filter(\.alreadyLinked).count
    if added > 0 {
      return "\(added) of \(preview.specs.count) already added"
    }
    return "Worked out from these changes, in \(preview.key)"
  }

  private func suggestionAccessibilityLabel(_ preview: ScaffoldPreviewView) -> String {
    "From the chord chart: \(suggestionHeadline(preview)) · \(suggestionSubtitle(preview))"
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
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      Text("Scales, arpeggios, and anything else you practise alongside this piece.")
        .font(IntradaFont.body)
        .foregroundStyle(IntradaColor.inkSecondary)
        .fixedSize(horizontal: false, vertical: true)
      BrandBarButton(action: { creatingExercise = true }) {
        Image(systemName: "plus")
        Text("Create an exercise")
      }
      .accessibilityLabel("Create an exercise for this piece")
      Button {
        showingPicker = true
      } label: {
        Text("Choose one from the library")
          .font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.accent)
          .frame(maxWidth: .infinity)
          .padding(.vertical, IntradaSpacing.controlGap)
          .contentShape(Rectangle())
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Choose an existing exercise for this piece")
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.bottom, IntradaSpacing.card)
  }

  // ── Exercise hero + provenance ──

  private var exerciseHero: some View {
    VStack(spacing: 6) {
      ScoreRing(
        score: item.practice?.latestScore.map(Int.init), size: 132, showsScale: true)
      // Names the hero as the score across every piece, so it can't be read as
      // one piece's — the distinction the "Used in" rows below make (#1087 B2).
      if !item.usedIn.isEmpty {
        Eyebrow("Overall")
      }
    }
    .frame(maxWidth: .infinity)
    .padding(.vertical, IntradaSpacing.controlGap)
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
        ScrollView(.horizontal, showsIndicators: false) {
          HStack(spacing: IntradaSpacing.card) {
            ForEach(item.variants, id: \.id) { step in
              StepRingItem(step: step)
            }
          }
          .padding(IntradaSpacing.cardCompact)
        }
        .cardSurface(cornerRadius: IntradaRadius.card)
      }
    }
    .onChange(of: item.variants.isEmpty) { _, isEmpty in
      if isEmpty { editingSteps = false }
    }
  }

  private var stepsHeader: some View {
    HStack(alignment: .firstTextBaseline) {
      Eyebrow("Steps")
      if !item.variants.isEmpty {
        Text("\(solidStepCount) of \(item.variants.count) solid")
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
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

  private var solidStepCount: Int {
    item.variants.filter(\.isSolid).count
  }

  private var stepsEmptyState: some View {
    VStack(spacing: IntradaSpacing.controlGap) {
      AddRowButton(title: "Add 12 major keys") { addKeyPreset(KeyHelper.circleMajor) }
        .accessibilityLabel("Add 12 major keys as this exercise's step ladder")
      AddRowButton(title: "Add 12 minor keys") { addKeyPreset(KeyHelper.circleMinor) }
        .accessibilityLabel("Add 12 minor keys as this exercise's step ladder")
      AddRowButton(title: "Add custom steps", style: .plain) {
        showingAddSteps = true
      }
      .accessibilityLabel("Add custom steps to this exercise")
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

  // ── Used in (pieces this exercise serves) ──

  // Gated on non-empty upstream until the empty state lands (#1363).
  private var usedInSection: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
      SectionHeader(title: "Used in")
      VStack(spacing: 0) {
        ForEach(Array(item.usedIn.enumerated()), id: \.offset) { index, usage in
          if index > 0 {
            HairlineDivider()
          }
          usedInRow(usage)
        }
      }
      .cardSurface()
    }
  }

  @ViewBuilder private func usedInRow(_ usage: ExerciseUsageView) -> some View {
    // A live piece taps through; the "On its own" bucket and since-removed pieces
    // (#1093, 2a) are inert rows — nowhere to navigate.
    if let piece = usage.piece, !usage.pieceRemoved {
      NavigationLink(value: piece.id) {
        UsedInRow(usage: usage, locale: locale, calendar: calendar, discloses: true)
      }
      .buttonStyle(.plain)
    } else {
      UsedInRow(usage: usage, locale: locale, calendar: calendar, discloses: false)
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

  // A laddered exercise drops the item-level Key/Tempo rows: each step carries
  // its own target, so a single value here would be misleading (#1083 C2).
  private var detailRows: [(label: String, value: String)] {
    guard item.itemType != .exercise || item.variants.isEmpty else { return [] }
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
      parts.append("Mark \(score) of 10 on this piece")
    } else {
      parts.append("Not yet rated on this piece")
    }
    return parts.joined(separator: ", ")
  }
}

/// One column in the Steps horizontal scroller: a ring (letter + progress arc)
/// and a state caption below — Solid (accent), Current (badge gold, calm and
/// static — no pulse; `breathe`/`metro` are retired per `design/CLAUDE.md`
/// "Motion"), or a dash for not yet reached.
private struct StepRingItem: View {
  let step: VariantView

  var body: some View {
    VStack(spacing: 6) {
      ScoreRing(
        score: step.latestScore.map(Int.init), size: 44, solid: step.isSolid,
        labelOverride: step.label)
      Text(captionText)
        .font(IntradaFont.meta)
        .foregroundStyle(captionColor)
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
  }

  private var captionText: String {
    if step.isSolid { return "Solid" }
    if step.isCurrent { return "Current" }
    return "—"
  }

  private var captionColor: Color {
    if step.isSolid { return IntradaColor.accent }
    if step.isCurrent { return IntradaColor.exerciseBadgeFg }
    return IntradaColor.inkFaint
  }

  private var accessibilityLabel: String {
    if step.isCurrent { return "\(step.label), current step" }
    guard let score = step.latestScore else { return "\(step.label), not yet attempted" }
    return step.isSolid
      ? "\(step.label), solid, \(score) of 10" : "\(step.label), \(score) of 10"
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

private struct UsedInRow: View {
  let usage: ExerciseUsageView
  let locale: Locale
  let calendar: Calendar
  let discloses: Bool

  private var isStandalone: Bool { usage.piece == nil }

  var body: some View {
    HStack(spacing: IntradaSpacing.row) {
      ScoreRing(score: usage.latestScore.map(Int.init), size: 44)
      VStack(alignment: .leading, spacing: 3) {
        Text(usage.rowTitle)
          .font(isStandalone ? IntradaFont.bodyMedium : IntradaFont.cardTitle())
          .foregroundStyle(usage.pieceRemoved ? IntradaColor.inkSecondary : IntradaColor.ink)
        Text(usage.metaLine(locale: locale, calendar: calendar))
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
    var parts = [usage.rowTitle]
    if usage.pieceRemoved { parts.append("removed from the library") }
    guard usage.sessionCount > 0 else {
      parts.append("not practised together yet")
      return parts.joined(separator: ", ")
    }
    if let score = usage.latestScore {
      parts.append("mark \(score) of 10")
    } else {
      parts.append("not yet rated")
    }
    let n = Int(usage.sessionCount)
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
