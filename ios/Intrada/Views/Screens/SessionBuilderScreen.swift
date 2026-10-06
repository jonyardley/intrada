import SharedTypes
import SwiftUI

// Dedicated "Build session" list. Interaction model per T9
// (docs/design-principles.md): every line is its own List row so the native
// long-press lift, swipe-to-remove, and Edit-mode delete apply everywhere;
// blocks render as joined card segments and move whole via their header row.
struct SessionBuilderScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @Environment(\.dynamicTypeSize) private var typeSize
  @State private var confirmingCancel = false
  // `SharedTypes`' domain `Set` (setlists) shadows `Swift.Set` here.
  @State private var collapsedGroups: Swift.Set<String> = []
  @State private var addingItems = false
  @State private var editMode: EditMode
  @State private var configuringEntry: EntrySettingsTarget?
  @State private var addingExerciseTarget: AddExerciseTarget?

  /// `startInEditMode` seeds the Edit-mode row controls for snapshot tests
  /// (they can't drive the toggle interactively).
  init(startInEditMode: Bool = false) {
    _editMode = State(initialValue: startInEditMode ? .active : .inactive)
  }

  private struct EntrySettingsTarget: Identifiable {
    let id: String
    let entry: SetlistEntryView
  }

  private struct AddExerciseTarget: Identifiable {
    let id: String
  }

  private var setlist: BuildingSetlistView? { store.viewModel?.buildingSetlist }
  private var entries: [SetlistEntryView] { setlist?.entries ?? [] }
  private var blocks: [SetlistBlockView] { setlist?.blocks ?? [] }
  private var hasGroups: Bool { blocks.contains { $0.groupId != nil } }
  private var isEditing: Bool { editMode == .active }

  // ── Row model ────────────────────────────────────────────────────────

  enum SegmentPosition {
    case single, top, middle, bottom
  }

  /// Row ids are entry ulids — stable across removals (#1024); the header and
  /// add-related rows borrow their group's id.
  enum BuilderRow: Identifiable {
    case standalone(SetlistBlockView, SetlistEntryView)
    case header(SetlistBlockView, collapsed: Bool, position: SegmentPosition)
    case nested(SetlistBlockView, SetlistEntryView, localIndex: Int, position: SegmentPosition)
    case addRelated(SetlistBlockView)

    var id: String {
      switch self {
      case .standalone(_, let entry): entry.id
      case .header(let block, _, _): "header-\(block.groupId ?? "")"
      case .nested(_, let entry, _, _): entry.id
      case .addRelated(let block): "add-\(block.groupId ?? "")"
      }
    }

    var startsUnit: Bool {
      switch self {
      case .standalone, .header: true
      case .nested, .addRelated: false
      }
    }

    var isInteractive: Bool {
      switch self {
      case .standalone, .header, .nested: true
      case .addRelated: false
      }
    }

    static func rows(
      for blocks: [SetlistBlockView], collapsed collapsedGroups: Swift.Set<String>,
      isEditing: Bool
    ) -> [BuilderRow] {
      var result: [BuilderRow] = []
      for block in blocks {
        guard let groupId = block.groupId else {
          if let entry = block.entries.first { result.append(.standalone(block, entry)) }
          continue
        }
        let collapsed = collapsedGroups.contains(groupId)
        let related = block.related
        let childRows = collapsed ? 0 : related.count + (isEditing ? 0 : 1)
        result.append(
          .header(block, collapsed: collapsed, position: childRows > 0 ? .top : .single))
        if !collapsed {
          for (index, entry) in related.enumerated() {
            let isLast = isEditing && index == related.count - 1
            result.append(
              .nested(block, entry, localIndex: index, position: isLast ? .bottom : .middle))
          }
          if !isEditing { result.append(.addRelated(block)) }
        }
      }
      return result
    }

    var ref: BuilderRowRef {
      switch self {
      case .standalone(_, let entry), .nested(_, let entry, _, _): .entry(entryId: entry.id)
      case .header(let block, _, _): .header(groupId: block.groupId ?? "")
      case .addRelated(let block): .addRelated(groupId: block.groupId ?? "")
      }
    }

    /// A List drop as the core reads it: the dragged row and its neighbours
    /// once it is lifted out (#2231).
    static func drop(in rows: [BuilderRow], from: Int, to destination: Int) -> Event? {
      guard rows.indices.contains(from) else { return nil }
      var remaining = rows
      let moved = remaining.remove(at: from)
      let slot = min(from < destination ? destination - 1 : destination, remaining.count)
      return .session(
        .moveRow(
          moved: moved.ref,
          before: slot < remaining.count ? remaining[slot].ref : nil,
          after: slot > 0 ? remaining[slot - 1].ref : nil))
    }
  }

  private var rows: [BuilderRow] {
    BuilderRow.rows(for: blocks, collapsed: collapsedGroups, isEditing: isEditing)
  }

  var body: some View {
    ScreenScaffold(
      title: "Build session",
      subtitle: isEditing ? "Editing" : summary,
      leadingContent: {
        Button("Cancel") { cancel() }
          .accessibilityIdentifier("builder.cancel")
      },
      trailingContent: { headerActions },
      content: {
        content
          .safeAreaInset(edge: .bottom, spacing: 0) {
            if !entries.isEmpty { startBar }
          }
      }
    )
    // A native back button (or its edge-swipe) would pop past `cancel()`'s
    // unsaved-plan confirmation; Cancel is the only way out (#1822).
    .navigationBarBackButtonHidden(true)
    .sheet(isPresented: $addingItems) { AddToSessionSheet().environment(store) }
    .sheet(item: $configuringEntry) { target in
      if let limits = store.viewModel?.limits {
        EntrySettingsSheet(entry: target.entry, limits: limits).environment(store)
      }
    }
    .sheet(item: $addingExerciseTarget) { target in
      AddRelatedExerciseSheet(groupId: target.id).environment(store)
    }
    .alert("Discard session plan?", isPresented: $confirmingCancel) {
      Button("Discard", role: .destructive) { dismiss() }
      Button("Keep editing", role: .cancel) {}
    } message: {
      Text("The items you've added will be cleared.")
    }
    // Emptying the list from Edit mode would strand "Editing" with no Done
    // button (it's gated on a non-empty list).
    .onChange(of: blocks.isEmpty) { _, empty in
      if empty { editMode = .inactive }
    }
  }

  @ViewBuilder private var headerActions: some View {
    if !blocks.isEmpty {
      Button(isEditing ? "Done" : "Edit") {
        withAnimation(IntradaMotion.standard) {
          editMode = isEditing ? .inactive : .active
        }
      }
    }
  }

  // Its own row rather than a header action (#1724): crowding the header
  // with three text buttons wrapped the title onto two lines and lost the
  // swipe.
  private var ungroupAllRow: some View {
    HStack {
      Spacer()
      Button("Ungroup all") { store.send(.session(.ungroupAllBlocks)) }
        .font(IntradaFont.secondary)
        .foregroundStyle(IntradaColor.accent)
        .frame(minHeight: 44)
        .contentShape(Rectangle())
    }
  }

  @ViewBuilder private var lengthCard: some View {
    if let setlist, let limits = store.viewModel?.limits {
      SessionLengthControl(
        title: "Today's length",
        lengthMins: setlist.lengthMins,
        detail: nil,
        valueLabel: setlist.lengthSummary ?? "",
        limits: limits,
        identifier: "builder.sessionLength"
      ) { store.send(.session(.setSessionLength(lengthMins: $0))) }
      .padding(IntradaSpacing.cardCompact)
      .cardSurface(cornerRadius: IntradaRadius.control)
    }
  }

  @ViewBuilder private var content: some View {
    if blocks.isEmpty {
      VStack(spacing: IntradaSpacing.card) {
        lengthCard
        Spacer()
        Text("Add pieces and exercises to build the session.")
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
        AddRowButton(title: "Add piece or exercise") { addingItems = true }
          .accessibilityIdentifier("builder.addItems")
        Spacer()
        Spacer()
      }
      .padding(IntradaSpacing.card)
      .frame(maxWidth: .infinity, maxHeight: .infinity)
    } else {
      VStack(spacing: 0) {
        // Inside the List the card was squeezed, dropping the gap between its
        // switch and stepper on the simulator (#1736), so it sits above.
        if !isEditing {
          lengthCard
            .fixedSize(horizontal: false, vertical: true)
            .padding(.horizontal, IntradaSpacing.card)
            .padding(.top, IntradaSpacing.card)
        }
        List {
          if hasGroups && !isEditing {
            ungroupAllRow
              .listRowBackground(Color.clear)
              .listRowSeparator(.hidden)
              .listRowInsets(
                EdgeInsets(
                  top: 0, leading: IntradaSpacing.card, bottom: IntradaSpacing.controlGap,
                  trailing: IntradaSpacing.card))
          }
          ForEach(rows) { row in
            rowView(row)
              .listRowBackground(Color.clear)
              .listRowSeparator(.hidden)
              .listRowInsets(insets(for: row))
              .moveDisabled(!row.isInteractive)
              .deleteDisabled(!row.isInteractive)
          }
          .onMove(perform: moveRows)
          .onDelete(perform: deleteRows)

          AddRowButton(title: "Add piece or exercise") { addingItems = true }
            .accessibilityIdentifier("builder.addItems")
            .listRowBackground(Color.clear)
            .listRowSeparator(.hidden)
            .listRowInsets(
              EdgeInsets(
                top: IntradaSpacing.controlGap, leading: IntradaSpacing.card,
                bottom: IntradaSpacing.card,
                trailing: IntradaSpacing.card)
            )
            .moveDisabled(true)
            .deleteDisabled(true)
        }
        .listStyle(.plain)
        // A block's flattened rows must butt together to read as one card: no
        // row spacing, and no 44pt minimum-height padding around short rows
        // (the add-related footer). The in-card hairlines do the separating.
        .listRowSpacing(0)
        .environment(\.defaultMinListRowHeight, 1)
        .scrollContentBackground(.hidden)
        .environment(\.editMode, $editMode)
      }
    }
  }

  // Card-to-card gap rides the unit-starting row's top inset; rows inside a
  // card butt up against each other so the segments read as one surface.
  private func insets(for row: BuilderRow) -> EdgeInsets {
    EdgeInsets(
      top: row.startsUnit ? IntradaSpacing.controlGap : 0,
      leading: IntradaSpacing.card,
      bottom: 0,
      trailing: IntradaSpacing.card)
  }

  private var gripGlyph: some View {
    Image(systemName: "line.3.horizontal")
      .font(IntradaFont.bodyMedium)
      .foregroundStyle(IntradaColor.inkFaintIcon)
      .dynamicTypeSize(...DynamicTypeSize.xxxLarge)
      .accessibilityHidden(true)
  }

  private var rowLineLimit: Int { typeSize.isAccessibilitySize ? 3 : 1 }

  private var titleLayout: AnyLayout {
    typeSize.isAccessibilitySize
      ? AnyLayout(VStackLayout(alignment: .leading, spacing: 2))
      : AnyLayout(HStackLayout(spacing: 6))
  }

  // ── Rows ─────────────────────────────────────────────────────────────

  @ViewBuilder private func rowView(_ row: BuilderRow) -> some View {
    switch row {
    case .standalone(let block, let entry):
      standaloneRow(block, entry: entry)
        .removeSwipe(named: entry.itemTitle) { removeUnit(block) }
    case .header(let block, let collapsed, let position):
      groupHeader(block, collapsed: collapsed, position: position)
        .removeSwipe(named: block.pieceTitle ?? "block") { removeUnit(block) }
    case .nested(let block, let entry, let localIndex, let position):
      nestedRow(entry, localIndex: localIndex, block: block, position: position)
        .removeSwipe(named: entry.itemTitle) { removeEntry(entry) }
    case .addRelated(let block):
      addRelatedRow(block)
    }
  }

  private func standaloneRow(_ block: SetlistBlockView, entry: SetlistEntryView) -> some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      if !isEditing { gripGlyph }
      HStack(spacing: IntradaSpacing.cardCompact) {
        entry.itemType.bar.frame(width: 4, height: 34).clipShape(Capsule())
        VStack(alignment: .leading, spacing: 2) {
          Text(entry.itemTitle).font(IntradaFont.cardTitle).foregroundStyle(IntradaColor.ink)
            .lineLimit(rowLineLimit)
          Text("\(entry.itemType.label)\(durationSuffix(block.durationDisplay))")
            .font(IntradaFont.small).foregroundStyle(IntradaColor.inkSecondary)
            .lineLimit(rowLineLimit)
          planLine(entry)
        }
        Spacer(minLength: IntradaSpacing.controlGap)
      }
      .accessibilityElement(children: .combine)
      .lastTimeAction(lastTime(entry)) { applyLastTime(entry) }
      .accessibilityAddTraits(isEditing ? [] : .isButton)
      .accessibilityHint(isEditing ? "" : "Opens settings")
      .accessibilityIdentifier("builder.row")
      .accessibilityAction(named: "Settings") {
        configuringEntry = EntrySettingsTarget(id: entry.id, entry: entry)
      }
      .accessibilityAction(named: "Move up") { moveUnit(block, by: -1) }
      .accessibilityAction(named: "Move down") { moveUnit(block, by: 1) }
      if !isEditing {
        Button {
          removeUnit(block)
        } label: {
          Image(systemName: "xmark").font(IntradaFont.secondary).foregroundStyle(
            IntradaColor.inkFaintIcon
          )
          .dynamicTypeSize(...DynamicTypeSize.xxxLarge)
          .frame(width: 44, height: 44)
          .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .hitTargetCompensation()
        .accessibilityLabel("Remove \(entry.itemTitle)")
        .accessibilityIdentifier("builder.remove")
      }
    }
    .padding(IntradaSpacing.cardCompact)
    .contentShape(Rectangle())
    .onTapGesture {
      guard !isEditing else { return }
      configuringEntry = EntrySettingsTarget(id: entry.id, entry: entry)
    }
    .cardSegment(.single)
  }

  private func groupHeader(_ block: SetlistBlockView, collapsed: Bool, position: SegmentPosition)
    -> some View
  {
    let groupId = block.groupId ?? ""
    return HStack(spacing: IntradaSpacing.cardCompact) {
      if !isEditing { gripGlyph }
      HStack(spacing: IntradaSpacing.cardCompact) {
        ItemKind.piece.bar.frame(width: 4, height: 34).clipShape(Capsule())
        VStack(alignment: .leading, spacing: 2) {
          titleLayout {
            Text(block.pieceTitle ?? "Related exercises")
              .font(IntradaFont.cardTitle).foregroundStyle(IntradaColor.ink)
              .lineLimit(rowLineLimit)
              .fixedSize(horizontal: false, vertical: true)
            if collapsed {
              Text(relatedLabel(block)).font(IntradaFont.secondary)
                .foregroundStyle(IntradaColor.inkSecondary)
            } else {
              groupPill
            }
          }
          if let subtitle = headerSubtitle(block, collapsed: collapsed) {
            Text(subtitle).font(IntradaFont.small).foregroundStyle(IntradaColor.inkSecondary)
              .lineLimit(rowLineLimit)
          }
          if let piece = block.piece { planLine(piece) }
        }
        Spacer(minLength: IntradaSpacing.controlGap)
      }
      .accessibilityElement(children: .combine)
      .accessibilityAddTraits(.isButton)
      .accessibilityLabel(
        "\(block.pieceTitle ?? "Related exercises"), \(relatedLabel(block))"
          + (block.piece.map(spokenPlan) ?? "")
      )
      .lastTimeAction(block.piece.flatMap(lastTime)) {
        if let piece = block.piece { applyLastTime(piece) }
      }
      .accessibilityHint(collapsed ? "Expands the block" : "Collapses the block")
      .accessibilityAction(named: "Move up") { moveUnit(block, by: -1) }
      .accessibilityAction(named: "Move down") { moveUnit(block, by: 1) }
      blockMenu(block, groupId: groupId)
      Image(systemName: collapsed ? "chevron.down" : "chevron.up")
        .font(IntradaFont.secondary).foregroundStyle(IntradaColor.inkFaintIcon).frame(width: 20)
        .accessibilityHidden(true)
    }
    .padding(.horizontal, IntradaSpacing.cardCompact)
    .padding(.vertical, IntradaSpacing.cardCompact)
    .contentShape(Rectangle())
    .onTapGesture {
      withAnimation(IntradaMotion.snappy) {
        if collapsed { collapsedGroups.remove(groupId) } else { collapsedGroups.insert(groupId) }
      }
    }
    .cardSegment(position, fill: collapsed ? nil : IntradaColor.surfaceSunken)
  }

  private func nestedRow(
    _ entry: SetlistEntryView, localIndex: Int, block: SetlistBlockView,
    position: SegmentPosition
  ) -> some View {
    VStack(spacing: 0) {
      HairlineDivider().padding(.leading, IntradaSpacing.card)
      HStack(spacing: IntradaSpacing.cardCompact) {
        if !isEditing { gripGlyph.frame(width: 24, height: 24) }
        HStack(spacing: IntradaSpacing.cardCompact) {
          ItemKind.exercise.bar.frame(width: 3, height: 26).clipShape(Capsule())
          VStack(alignment: .leading, spacing: 1) {
            Text(entry.itemTitle).font(IntradaFont.bodyMedium).foregroundStyle(IntradaColor.ink)
              .lineLimit(rowLineLimit)
            Text(nestedMeta(entry)).font(IntradaFont.small)
              .foregroundStyle(IntradaColor.inkSecondary)
              .lineLimit(rowLineLimit)
            planLine(entry)
          }
          Spacer(minLength: 0)
        }
        .accessibilityElement(children: .combine)
        .lastTimeAction(lastTime(entry)) { applyLastTime(entry) }
        .accessibilityAddTraits(isEditing ? [] : .isButton)
        .accessibilityHint(isEditing ? "" : "Opens settings")
        .accessibilityIdentifier("builder.row")
        .accessibilityAction(named: "Settings") {
          configuringEntry = EntrySettingsTarget(id: entry.id, entry: entry)
        }
        .accessibilityAction(named: "Move up") {
          moveExercise(entry, toLocal: localIndex - 1, in: block)
        }
        .accessibilityAction(named: "Move down") {
          moveExercise(entry, toLocal: localIndex + 1, in: block)
        }
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.vertical, IntradaSpacing.controlGap)
      .contentShape(Rectangle())
      .onTapGesture {
        guard !isEditing else { return }
        configuringEntry = EntrySettingsTarget(id: entry.id, entry: entry)
      }
    }
    .cardSegment(position)
  }

  private func addRelatedRow(_ block: SetlistBlockView) -> some View {
    VStack(spacing: 0) {
      HairlineDivider().padding(.leading, IntradaSpacing.card)
      Button {
        addingExerciseTarget = block.groupId.map(AddExerciseTarget.init)
      } label: {
        Label("Add a related exercise", systemImage: "plus")
          .font(IntradaFont.secondary).foregroundStyle(IntradaColor.accent)
          .frame(maxWidth: .infinity, minHeight: 44)
      }
      .buttonStyle(.plain)
    }
    .cardSegment(.bottom)
  }

  // ── Plan (#2249, #2303, #2315) ──────────────────────────────────────

  @ViewBuilder private func planLine(_ entry: SetlistEntryView) -> some View {
    let tags = planTags(entry)
    if !tags.isEmpty {
      FlowLayout(spacing: 6) {
        ForEach(tags, id: \.self) { TagChip($0) }
      }
      .padding(.top, 4)
    }
    if let offer = lastTime(entry) {
      OfferChip(offer.label) { applyLastTime(entry) }
        .accessibilityHidden(true)
        .padding(.top, 2)
    }
  }

  private func planTags(_ entry: SetlistEntryView) -> [String] {
    let segments = entry.record.segments
    let split = segments.count > 1
    let variations =
      setlist?.entryVariations.first { $0.entryId == entry.id }?.variations ?? []
    return segments.map {
      split && $0.plannedSecs > 0 ? "\($0.label) \($0.plannedDisplay)" : $0.label
    }
      + entry.plannedVariationIds.compactMap { id in variations.first { $0.id == id }?.label }
      + (entry.record.focus.map { ["Focus: \($0.label)"] } ?? [])
  }

  /// The header sets its own label, which drops the tags it holds.
  private func spokenPlan(_ entry: SetlistEntryView) -> String {
    planTags(entry).map { ", \($0)" }.joined()
  }

  private func lastTime(_ entry: SetlistEntryView) -> LastTimeView? {
    setlist?.lastTimes.first { $0.entryId == entry.id }
  }

  private func applyLastTime(_ entry: SetlistEntryView) {
    store.send(.session(.applyLastTime(entryId: entry.id)), onSuccess: .impact)
  }

  private var groupPill: some View {
    Text("Group")
      .font(IntradaFont.badge)
      .foregroundStyle(IntradaColor.pieceBadgeFg)
      .padding(.horizontal, 6).padding(.vertical, 2)
      .background(
        IntradaColor.pieceBadgeBg, in: RoundedRectangle(cornerRadius: IntradaRadius.badge))
  }

  private func blockMenu(_ block: SetlistBlockView, groupId: String) -> some View {
    Menu {
      if let piece = block.piece {
        Button("Piece settings") {
          configuringEntry = EntrySettingsTarget(id: piece.id, entry: piece)
        }
      }
      Button("Just the piece") { store.send(.session(.keepOnlyPiece(groupId: groupId))) }
      Button("Ungroup") { store.send(.session(.ungroupBlock(groupId: groupId))) }
      Button("Remove block", role: .destructive) {
        store.send(.session(.removeBlock(groupId: groupId)))
      }
    } label: {
      Image(systemName: "ellipsis")
        .font(IntradaFont.bodyMedium).foregroundStyle(IntradaColor.inkSecondary)
        .frame(width: 44, height: 44)
        .contentShape(Rectangle())
    }
    .hitTargetCompensation()
    .accessibilityLabel("Block actions")
  }

  // ── Sticky start bar ─────────────────────────────────────────────────

  // One-primary-action frontier. `startSession` flips the core Building → Active;
  // `buildingSetlist` goes nil (this screen auto-pops) and `activeSession` goes
  // non-nil (RootView presents the player). State-driven — no local nav flag.
  private var startBar: some View {
    BrandBarButton {
      store.send(.session(.startSession(now: SessionClock.nowRFC3339())))
    } label: {
      Image(systemName: "play.fill")
      Text(startTitle)
    }
    .accessibilityIdentifier("builder.start")
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.cardCompact)
    .padding(.bottom, IntradaSpacing.section)
    .background(IntradaColor.paperTop)
    .overlay(alignment: .top) { Rectangle().fill(IntradaColor.hairline).frame(height: 1) }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Start session")
    .accessibilityValue(
      "\(entries.count) item\(entries.count == 1 ? "" : "s")"
        + (setlist?.totalDurationSummary.map { ", \($0)" } ?? ""))
  }

  // ── Copy ─────────────────────────────────────────────────────────────

  private var summary: String {
    let items = Int(setlist?.itemCount ?? 0)
    let itemStr = "\(items) item\(items == 1 ? "" : "s")"
    var counts = itemStr
    if hasGroups {
      let n = blocks.count
      counts = "\(n) block\(n == 1 ? "" : "s") · \(itemStr)"
    }
    guard let duration = setlist?.totalDurationSummary else { return counts }
    return "\(duration) · \(counts)"
  }

  private var startTitle: String {
    guard let duration = setlist?.totalDurationSummary else { return "Start session" }
    return "Start session · \(duration)"
  }

  private func relatedLabel(_ block: SetlistBlockView) -> String {
    block.relatedCount == 0 ? "piece only" : "+\(block.relatedCount) related"
  }

  // HACK(#1101): string-matches the core's rendered sentinels ("—" for no
  // planned durations, "0s" for a not-yet-practised entry) to drop the noise;
  // the core should send empty strings instead — tracked in that issue.
  private func durationSuffix(_ display: String) -> String {
    display.isEmpty || display == "—" || display == "0s" ? "" : " · \(display)"
  }

  private func headerSubtitle(_ block: SetlistBlockView, collapsed: Bool) -> String? {
    if collapsed {
      let suffix = durationSuffix(block.durationDisplay)
      return suffix.isEmpty ? nil : block.durationDisplay
    }
    guard block.relatedCount > 0 else {
      let suffix = durationSuffix(block.durationDisplay)
      return suffix.isEmpty ? "Piece only" : block.durationDisplay
    }
    let related = block.relatedCount == 1 ? "1 related" : "\(block.relatedCount) related"
    return "\(related), then piece\(durationSuffix(block.durationDisplay))"
  }

  private func nestedMeta(_ entry: SetlistEntryView) -> String {
    "Related\(durationSuffix(entry.durationDisplay))"
  }

  // ── Actions ──────────────────────────────────────────────────────────

  private func removeUnit(_ block: SetlistBlockView) {
    if let groupId = block.groupId {
      store.send(.session(.removeBlock(groupId: groupId)), onSuccess: .impact)
    } else if let entryId = block.entries.first?.id {
      store.send(.session(.removeFromSetlist(entryId: entryId)), onSuccess: .impact)
    }
  }

  private func removeEntry(_ entry: SetlistEntryView) {
    store.send(.session(.removeFromSetlist(entryId: entry.id)), onSuccess: .impact)
  }

  private func deleteRows(at offsets: IndexSet) {
    // Snapshot first: each removal rebuilds `rows`, so live indices would
    // drift under a multi-index delete. Removing a header takes its nested
    // entries with it, so skip children of groups already gone this batch.
    let snapshot = rows
    var removedGroups: Swift.Set<String> = []
    for offset in offsets where snapshot.indices.contains(offset) {
      switch snapshot[offset] {
      case .standalone(let block, _), .header(let block, _, _):
        if let groupId = block.groupId { removedGroups.insert(groupId) }
        removeUnit(block)
      case .nested(let block, let entry, _, _):
        guard !removedGroups.contains(block.groupId ?? "") else { continue }
        removeEntry(entry)
      case .addRelated: break
      }
    }
  }

  /// VoiceOver path for a related exercise's move within its block.
  private func moveExercise(
    _ entry: SetlistEntryView, toLocal destLocal: Int, in block: SetlistBlockView
  ) {
    guard destLocal >= 0, destLocal < block.relatedCount else { return }
    send(.session(.moveRelated(entryId: entry.id, newPosition: UInt64(destLocal))))
  }

  /// VoiceOver path for unit reorder (the pointer path is the List's native
  /// long-press drag on the header/standalone row).
  private func moveUnit(_ block: SetlistBlockView, by delta: Int) {
    guard
      let from = blocks.firstIndex(where: {
        $0.entries.first?.id == block.entries.first?.id
      }),
      blocks.indices.contains(from + delta),
      let entryId = block.entries.first?.id
    else { return }
    send(.session(.moveUnit(entryId: entryId, newPosition: UInt64(from + delta))))
  }

  private func moveRows(from source: IndexSet, to destination: Int) {
    guard let from = source.first,
      let drop = BuilderRow.drop(in: rows, from: from, to: destination)
    else { return }
    // A drop home changes nothing, so it plays no tick (#1413).
    let order = setlist?.entries.map(\.id)
    store.send(drop, onSuccess: .selection) { $0.buildingSetlist?.entries.map(\.id) != order }
  }

  private func send(_ event: Event) {
    store.send(event, onSuccess: .selection)
  }

  private func cancel() {
    if entries.isEmpty { dismiss() } else { confirmingCancel = true }
  }
}

extension View {
  /// Destructive trailing swipe with the short verb on screen and the full
  /// item name for assistive tech.
  fileprivate func removeSwipe(named title: String, action: @escaping () -> Void) -> some View {
    swipeActions(edge: .trailing, allowsFullSwipe: true) {
      Button(role: .destructive, action: action) {
        Label("Remove", systemImage: "trash")
      }
      .accessibilityLabel("Remove \(title)")
    }
  }

  /// VoiceOver's path to the last-time chip. The chip itself is hidden from
  /// it, so a double tap on the row never lands on the chip.
  @ViewBuilder fileprivate func lastTimeAction(
    _ offer: LastTimeView?, perform action: @escaping () -> Void
  ) -> some View {
    if let offer {
      accessibilityAction(named: offer.label, action)
    } else {
      self
    }
  }

  /// Cancels the layout growth of a 44pt hit target wrapped around a ~24pt
  /// glyph, so small controls meet the HIG minimum without shifting the row.
  fileprivate func hitTargetCompensation() -> some View {
    padding(-10)
  }
}

// ── Card segments ──────────────────────────────────────────────────────

/// Card chrome for a flattened row: fill + per-position corner rounding, and
/// an outer hairline stroke drawn only on the card's outside edges so stacked
/// segments read as one card.
private struct CardSegmentModifier: ViewModifier {
  let position: SessionBuilderScreen.SegmentPosition
  var fill: Color?

  func body(content: Content) -> some View {
    content
      .background(fill ?? IntradaColor.cardFill)
      .clipShape(shape)
      .overlay(CardSegmentBorder(position: position).stroke(IntradaColor.hairline, lineWidth: 1))
  }

  private var shape: UnevenRoundedRectangle {
    let r = IntradaRadius.card
    return switch position {
    case .single:
      UnevenRoundedRectangle(
        cornerRadii: .init(topLeading: r, bottomLeading: r, bottomTrailing: r, topTrailing: r))
    case .top:
      UnevenRoundedRectangle(cornerRadii: .init(topLeading: r, topTrailing: r))
    case .middle:
      UnevenRoundedRectangle(cornerRadii: .init())
    case .bottom:
      UnevenRoundedRectangle(cornerRadii: .init(bottomLeading: r, bottomTrailing: r))
    }
  }
}

/// The outside-only border path for a card segment: sides always; the top arc
/// only for top/single; the bottom arc only for bottom/single. Shared edges
/// between segments carry no stroke (the in-card hairlines are separate).
private struct CardSegmentBorder: Shape {
  let position: SessionBuilderScreen.SegmentPosition

  func path(in rect: CGRect) -> Path {
    let r = IntradaRadius.card
    var path = Path()
    let roundTop = position == .top || position == .single
    let roundBottom = position == .bottom || position == .single

    path.move(to: CGPoint(x: rect.minX, y: roundBottom ? rect.maxY - r : rect.maxY))
    path.addLine(to: CGPoint(x: rect.minX, y: roundTop ? rect.minY + r : rect.minY))
    if roundTop {
      path.addArc(
        center: CGPoint(x: rect.minX + r, y: rect.minY + r), radius: r,
        startAngle: .degrees(180), endAngle: .degrees(270), clockwise: false)
      path.addLine(to: CGPoint(x: rect.maxX - r, y: rect.minY))
      path.addArc(
        center: CGPoint(x: rect.maxX - r, y: rect.minY + r), radius: r,
        startAngle: .degrees(270), endAngle: .degrees(0), clockwise: false)
    } else {
      path.move(to: CGPoint(x: rect.maxX, y: rect.minY))
    }
    path.addLine(to: CGPoint(x: rect.maxX, y: roundBottom ? rect.maxY - r : rect.maxY))
    if roundBottom {
      path.addArc(
        center: CGPoint(x: rect.maxX - r, y: rect.maxY - r), radius: r,
        startAngle: .degrees(0), endAngle: .degrees(90), clockwise: false)
      path.addLine(to: CGPoint(x: rect.minX + r, y: rect.maxY))
      path.addArc(
        center: CGPoint(x: rect.minX + r, y: rect.maxY - r), radius: r,
        startAngle: .degrees(90), endAngle: .degrees(180), clockwise: false)
    }
    return path
  }
}

extension View {
  fileprivate func cardSegment(
    _ position: SessionBuilderScreen.SegmentPosition, fill: Color? = nil
  ) -> some View {
    modifier(CardSegmentModifier(position: position, fill: fill))
  }
}

#if DEBUG
  #Preview("Populated") {
    NavigationStack { SessionBuilderScreen() }.environment(Store.previewBuilding)
  }

  #Preview("Grouped") {
    NavigationStack { SessionBuilderScreen() }.environment(Store.previewBuildingGrouped)
  }
#endif
