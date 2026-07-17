import SharedTypes
import SwiftUI

// Dedicated "Build session" list. Interaction model per T9
// (docs/design-principles.md): every line is its own List row so the native
// long-press lift, swipe-to-remove, and Edit-mode delete apply everywhere;
// blocks render as joined card segments and move whole via their header row.
struct SessionBuilderScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var confirmingCancel = false
  // `SharedTypes`' domain `Set` (setlists) shadows `Swift.Set` here.
  @State private var collapsedGroups: Swift.Set<String> = []
  @State private var addingItems = false
  @State private var editMode: EditMode
  @State private var configuringEntry: EntrySettingsTarget?
  @State private var addingExerciseTarget: AddExerciseTarget?

  /// `startInEditMode` seeds the Edit-mode row controls for snapshot tests
  /// (they can't drive the Edit/Done toggle interactively).
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

  fileprivate enum SegmentPosition {
    case single, top, middle, bottom
  }

  /// Row ids are entry ulids — stable across removals (#1024); the header and
  /// add-related rows borrow their group's id.
  private enum BuilderRow: Identifiable {
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

    func belongs(to block: SetlistBlockView) -> Bool {
      switch self {
      case .nested(let b, _, _, _), .addRelated(let b):
        b.groupId != nil && b.groupId == block.groupId
      case .standalone, .header: false
      }
    }
  }

  private var rows: [BuilderRow] {
    var result: [BuilderRow] = []
    for block in blocks {
      guard let groupId = block.groupId else {
        if let entry = block.entries.first { result.append(.standalone(block, entry)) }
        continue
      }
      let collapsed = collapsedGroups.contains(groupId)
      let related = block.entries.filter { $0.itemType == .exercise }
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

  var body: some View {
    ZStack(alignment: .bottom) {
      PaperBackground()
      VStack(alignment: .leading, spacing: 0) {
        header
        content
      }
      if !entries.isEmpty { startBar }
    }
    .navigationBarTitleDisplayMode(.inline)
    .navigationBarBackButtonHidden(true)
    .toolbar {
      ToolbarItem(placement: .topBarLeading) { Button("Cancel") { cancel() } }
      if hasGroups && !isEditing {
        ToolbarItem(placement: .topBarTrailing) {
          Button("Ungroup all") { store.send(.session(.ungroupAllBlocks)) }
            .font(IntradaFont.meta)
        }
      }
      if !blocks.isEmpty {
        ToolbarItem(placement: .topBarTrailing) {
          Button(isEditing ? "Done" : "Edit") {
            withAnimation(IntradaMotion.standard) {
              editMode = isEditing ? .inactive : .active
            }
          }
        }
      }
    }
    .sheet(isPresented: $addingItems) { AddToSessionSheet().environment(store) }
    .sheet(item: $configuringEntry) { target in
      EntrySettingsSheet(entry: target.entry).environment(store)
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

  private var header: some View {
    VStack(alignment: .leading, spacing: 4) {
      Text("Build session")
        .font(IntradaFont.pageTitle(28))
        .foregroundStyle(IntradaColor.ink)
      Text(isEditing ? "Editing" : summary)
        .font(IntradaFont.subtitle)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.top, IntradaSpacing.controlGap)
    .padding(.bottom, IntradaSpacing.cardCompact)
  }

  @ViewBuilder private var content: some View {
    if blocks.isEmpty {
      VStack(spacing: IntradaSpacing.card) {
        Spacer()
        Text("Add pieces and exercises to build your session.")
          .font(IntradaFont.body)
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
        AddRowButton(title: "Add piece or exercise") { addingItems = true }
        Spacer()
        Spacer()
      }
      .padding(IntradaSpacing.card)
      .frame(maxWidth: .infinity, maxHeight: .infinity)
    } else {
      List {
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
          .listRowBackground(Color.clear)
          .listRowSeparator(.hidden)
          .listRowInsets(
            EdgeInsets(
              top: IntradaSpacing.controlGap, leading: IntradaSpacing.card, bottom: 100,
              trailing: IntradaSpacing.card)
          )
          .moveDisabled(true)
          .deleteDisabled(true)
      }
      .listStyle(.plain)
      // A block's flattened rows must butt together to read as one card: no
      // row spacing, and no 44pt minimum-height padding around short rows
      // (the add-related footer) — the in-card hairlines do the separating.
      .listRowSpacing(0)
      .environment(\.defaultMinListRowHeight, 1)
      .scrollContentBackground(.hidden)
      .environment(\.editMode, $editMode)
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
      .foregroundStyle(IntradaColor.inkFaint)
      .accessibilityHidden(true)
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
          Text(entry.itemTitle).font(IntradaFont.cardTitle(16)).foregroundStyle(IntradaColor.ink)
            .lineLimit(1)
          Text(
            "Standalone \(entry.itemType.label.lowercased())\(durationSuffix(block.durationDisplay))"
          )
          .font(IntradaFont.micro).foregroundStyle(IntradaColor.inkSecondary)
          .lineLimit(1)
        }
        Spacer(minLength: IntradaSpacing.controlGap)
      }
      .accessibilityElement(children: .combine)
      .accessibilityAddTraits(isEditing ? [] : .isButton)
      .accessibilityHint(isEditing ? "" : "Opens settings")
      .accessibilityAction(named: "Settings") {
        configuringEntry = EntrySettingsTarget(id: entry.id, entry: entry)
      }
      .accessibilityAction(named: "Move up") { moveUnit(block, by: -1) }
      .accessibilityAction(named: "Move down") { moveUnit(block, by: 1) }
      if !isEditing {
        Button {
          removeUnit(block)
        } label: {
          Image(systemName: "xmark").font(IntradaFont.meta).foregroundStyle(IntradaColor.inkFaint)
            .frame(width: 44, height: 44)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .hitTargetCompensation()
        .accessibilityLabel("Remove \(entry.itemTitle)")
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
          HStack(spacing: 6) {
            Text(block.pieceTitle ?? "Related exercises")
              .font(IntradaFont.cardTitle(16)).foregroundStyle(IntradaColor.ink).lineLimit(1)
            if collapsed {
              Text(relatedLabel(block)).font(IntradaFont.meta)
                .foregroundStyle(IntradaColor.inkSecondary)
            } else {
              groupPill
            }
          }
          if let subtitle = headerSubtitle(block, collapsed: collapsed) {
            Text(subtitle).font(IntradaFont.micro).foregroundStyle(IntradaColor.inkSecondary)
              .lineLimit(1)
          }
        }
        Spacer(minLength: IntradaSpacing.controlGap)
      }
      .accessibilityElement(children: .combine)
      .accessibilityAddTraits(.isButton)
      .accessibilityLabel(
        "\(block.pieceTitle ?? "Related exercises"), \(relatedLabel(block))"
      )
      .accessibilityHint(collapsed ? "Expands the block" : "Collapses the block")
      .accessibilityAction(named: "Move up") { moveUnit(block, by: -1) }
      .accessibilityAction(named: "Move down") { moveUnit(block, by: 1) }
      blockMenu(block, groupId: groupId)
      Image(systemName: collapsed ? "chevron.down" : "chevron.up")
        .font(IntradaFont.meta).foregroundStyle(IntradaColor.inkFaint).frame(width: 20)
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
              .lineLimit(1)
            Text(nestedMeta(entry)).font(IntradaFont.micro)
              .foregroundStyle(IntradaColor.inkSecondary)
              .lineLimit(1)
          }
          Spacer(minLength: 0)
        }
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits(isEditing ? [] : .isButton)
        .accessibilityHint(isEditing ? "" : "Opens settings")
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
          .font(IntradaFont.meta).foregroundStyle(IntradaColor.accent)
          .frame(maxWidth: .infinity, minHeight: 44)
      }
      .buttonStyle(.plain)
    }
    .cardSegment(.bottom)
  }

  private var groupPill: some View {
    Text("Group")
      .font(IntradaFont.micro).textCase(.uppercase).kerning(0.4)
      .foregroundStyle(IntradaColor.pieceBadgeFg)
      .padding(.horizontal, 6).padding(.vertical, 2)
      .background(
        IntradaColor.pieceBadgeBg, in: RoundedRectangle(cornerRadius: IntradaRadius.badge))
  }

  private func blockMenu(_ block: SetlistBlockView, groupId: String) -> some View {
    Menu {
      if let piece = block.entries.first(where: { $0.itemType == .piece }) {
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

  /// Moves `entry` to `toLocal` within `block`'s related-exercise run (never
  /// past the anchor piece, which is always last in `block.entries`).
  private func moveExercise(
    _ entry: SetlistEntryView, toLocal destLocal: Int, in block: SetlistBlockView
  ) {
    let related = block.entries.filter { $0.itemType == .exercise }
    guard related.indices.contains(destLocal),
      let localIndex = related.firstIndex(where: { $0.id == entry.id }),
      destLocal != localIndex,
      let blockStart = entries.firstIndex(where: { $0.id == related[0].id })
    else { return }
    store.send(
      .session(.reorderSetlist(entryId: entry.id, newPosition: UInt64(blockStart + destLocal))),
      onSuccess: .selection)
  }

  /// VoiceOver path for unit reorder (the pointer path is the List's native
  /// long-press drag on the header/standalone row).
  private func moveUnit(_ block: SetlistBlockView, by delta: Int) {
    let unitBlocks = blocks
    guard
      let from = unitBlocks.firstIndex(where: {
        $0.entries.first?.id == block.entries.first?.id
      }),
      unitBlocks.indices.contains(from + delta)
    else { return }
    sendUnitMove(block, toUnitIndex: from + delta)
  }

  /// A block moves whole via `reorderBlock`; a standalone via `reorderSetlist`
  /// aimed at the flat-entry index its new unit slot implies.
  private func sendUnitMove(_ block: SetlistBlockView, toUnitIndex target: Int) {
    if let groupId = block.groupId {
      store.send(
        .session(.reorderBlock(groupId: groupId, newPosition: UInt64(target))),
        onSuccess: .selection)
    } else if let entryId = block.entries.first?.id {
      var reordered = blocks
      guard let from = reordered.firstIndex(where: { $0.entries.first?.id == entryId }) else {
        return
      }
      let moved = reordered.remove(at: from)
      reordered.insert(moved, at: min(target, reordered.count))
      let flatIds = reordered.flatMap { $0.entries.map(\.id) }
      if let newIndex = flatIds.firstIndex(of: entryId) {
        store.send(
          .session(.reorderSetlist(entryId: entryId, newPosition: UInt64(newIndex))),
          onSuccess: .selection)
      }
    }
  }

  /// Interprets the List's single-row move on the flattened rows:
  /// - a header/standalone row moves its whole unit to the unit slot the drop
  ///   position implies (a drop inside another block clamps to its boundary);
  /// - a nested exercise clamps to its own block's related run.
  private func moveRows(from source: IndexSet, to destination: Int) {
    let currentRows = rows
    guard let from = source.first, currentRows.indices.contains(from) else { return }
    let moved = currentRows[from]
    var remaining = currentRows
    remaining.remove(at: from)
    let slot = (from < destination ? destination - 1 : destination)
      .clamped(to: 0...remaining.count)

    switch moved {
    case .standalone(let block, _), .header(let block, _, _):
      // Dropping back among the unit's own remaining rows means "stay put".
      if slot < remaining.count, remaining[slot].belongs(to: block) { return }
      var target = remaining[..<slot].filter(\.startsUnit).count
      // A drop INSIDE a foreign unit's span counts that unit's header as
      // passed; when dragging upward the intent is "before that unit", so
      // step back one — otherwise swapping with the block above needs a
      // pixel-precise drop on its header row.
      if destination <= from, slot < remaining.count, !remaining[slot].startsUnit {
        target = max(0, target - 1)
      }
      let currentUnit = blocks.firstIndex { $0.entries.first?.id == block.entries.first?.id }
      guard target != currentUnit else { return }
      sendUnitMove(block, toUnitIndex: target)
    case .nested(let block, let entry, let localIndex, _):
      // A drop outside the source block's own nested run is a no-op (the row
      // snaps home) — silently converting it into a within-block move would
      // reorder siblings the user never touched.
      let nestedIndices = remaining.indices.filter { index in
        if case .nested(let b, _, _, _) = remaining[index] {
          return b.groupId == block.groupId
        }
        return false
      }
      guard let first = nestedIndices.first, let last = nestedIndices.last,
        (first...(last + 1)).contains(slot)
      else { return }
      let target = slot - first
      guard target != localIndex else { return }
      moveExercise(entry, toLocal: target, in: block)
    case .addRelated:
      break
    }
  }

  private func cancel() {
    if entries.isEmpty { dismiss() } else { confirmingCancel = true }
  }
}

extension Comparable {
  fileprivate func clamped(to range: ClosedRange<Self>) -> Self {
    min(max(self, range.lowerBound), range.upperBound)
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
