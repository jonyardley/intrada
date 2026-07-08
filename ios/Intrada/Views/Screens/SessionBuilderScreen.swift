import SharedTypes
import SwiftUI

// Dedicated "Build session" list: grouped blocks + standalone items, added via
// the "Add to session" sheet and started from a sticky bar. A piece's related
// exercises travel as one draggable block.
struct SessionBuilderScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var confirmingCancel = false
  // `SharedTypes`' domain `Set` (setlists) shadows `Swift.Set` here.
  @State private var collapsedGroups: Swift.Set<String> = []
  @State private var addingItems = false
  @State private var editMode: EditMode
  @State private var configuringEntry: EntrySettingsTarget?
  @State private var addingExerciseGroupId: String?

  /// `startInEditMode` seeds the Edit-mode nested-row controls for snapshot
  /// tests (they can't drive the Edit/Done toggle interactively).
  init(startInEditMode: Bool = false) {
    _editMode = State(initialValue: startInEditMode ? .active : .inactive)
  }

  private struct EntrySettingsTarget: Identifiable {
    let id: String
    let entry: SetlistEntryView
  }

  private var setlist: BuildingSetlistView? { store.viewModel?.buildingSetlist }
  private var entries: [SetlistEntryView] { setlist?.entries ?? [] }
  private var blocks: [SetlistBlockView] { setlist?.blocks ?? [] }
  private var hasGroups: Bool { blocks.contains { $0.groupId != nil } }

  /// Stable-identity units for the queue `ForEach` — keying on the first entry's
  /// ulid (not the array index) avoids the SwiftUI out-of-range crash when a unit
  /// is removed and `blocks` shrinks mid-render (#1024).
  private struct BuilderUnit: Identifiable {
    let id: String
    let block: SetlistBlockView
  }
  private var units: [BuilderUnit] {
    blocks.enumerated().map { offset, block in
      BuilderUnit(id: block.entries.first?.id ?? "unit-\(offset)", block: block)
    }
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
      if !blocks.isEmpty {
        ToolbarItem(placement: .topBarTrailing) {
          Button(editMode == .active ? "Done" : "Edit") {
            withAnimation(IntradaMotion.standard) {
              editMode = editMode == .active ? .inactive : .active
            }
          }
        }
      }
      if hasGroups && editMode == .active {
        ToolbarItem(placement: .topBarTrailing) {
          Button("Ungroup all") { store.send(.session(.ungroupAllBlocks)) }
            .font(IntradaFont.meta)
        }
      }
    }
    .sheet(isPresented: $addingItems) { AddToSessionSheet().environment(store) }
    .sheet(item: $configuringEntry) { target in
      EntrySettingsSheet(entry: target.entry).environment(store)
    }
    .sheet(
      isPresented: Binding(
        get: { addingExerciseGroupId != nil }, set: { if !$0 { addingExerciseGroupId = nil } })
    ) {
      if let groupId = addingExerciseGroupId {
        AddRelatedExerciseSheet(groupId: groupId).environment(store)
      }
    }
    .alert("Discard session plan?", isPresented: $confirmingCancel) {
      Button("Discard", role: .destructive) { dismiss() }
      Button("Keep editing", role: .cancel) {}
    } message: {
      Text("The items you've added will be cleared.")
    }
  }

  private var header: some View {
    VStack(alignment: .leading, spacing: 4) {
      Text("Build session")
        .font(IntradaFont.pageTitle(28))
        .foregroundStyle(IntradaColor.ink)
      Text(editMode == .active ? "Editing" : summary)
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
        ForEach(units) { unit in
          unitView(unit.block)
            .listRowBackground(Color.clear)
            .listRowSeparator(.hidden)
            .listRowInsets(rowInsets)
            .swipeActions(edge: .trailing, allowsFullSwipe: true) {
              Button(role: .destructive) {
                removeUnit(unit.block)
              } label: {
                Label(removeLabel(unit.block), systemImage: "trash")
              }
            }
        }
        .onMove(perform: moveUnits)

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
      .scrollContentBackground(.hidden)
      .environment(\.editMode, $editMode)
    }
  }

  private var rowInsets: EdgeInsets {
    EdgeInsets(
      top: IntradaSpacing.controlGap / 2, leading: IntradaSpacing.card,
      bottom: IntradaSpacing.controlGap / 2, trailing: IntradaSpacing.card)
  }

  // ── Units ────────────────────────────────────────────────────────────

  @ViewBuilder private func unitView(_ block: SetlistBlockView) -> some View {
    if block.groupId == nil, let entry = block.entries.first {
      standaloneRow(block, entry: entry)
    } else {
      blockCard(block)
    }
  }

  private func standaloneRow(_ block: SetlistBlockView, entry: SetlistEntryView) -> some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      entry.itemType.bar.frame(width: 4, height: 34).clipShape(Capsule())
      VStack(alignment: .leading, spacing: 2) {
        Text(entry.itemTitle).font(IntradaFont.cardTitle(16)).foregroundStyle(IntradaColor.ink)
          .lineLimit(1)
        Text("Standalone \(entry.itemType.label.lowercased()) · \(block.durationDisplay)")
          .font(IntradaFont.micro).foregroundStyle(IntradaColor.inkSecondary)
      }
      Spacer(minLength: IntradaSpacing.controlGap)
      if editMode == .active {
        Button {
          configuringEntry = EntrySettingsTarget(id: entry.id, entry: entry)
        } label: {
          Image(systemName: "slider.horizontal.3").font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        .buttonStyle(.plain)
        .accessibilityLabel("\(entry.itemTitle) settings")
      }
      Button {
        removeUnit(block)
      } label: {
        Image(systemName: "xmark").font(IntradaFont.meta).foregroundStyle(IntradaColor.inkFaint)
          .frame(width: 24, height: 24)
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Remove \(entry.itemTitle)")
    }
    .padding(IntradaSpacing.cardCompact)
    .cardSurface(cornerRadius: IntradaRadius.card)
  }

  private func blockCard(_ block: SetlistBlockView) -> some View {
    let groupId = block.groupId ?? ""
    let collapsed = collapsedGroups.contains(groupId)
    return VStack(spacing: 0) {
      groupHeader(block, groupId: groupId, collapsed: collapsed)
      if !collapsed {
        // The anchor piece is the header; nested rows are its related exercises
        // (the core orders a block as [related exercises…, piece]).
        let related = block.entries.filter { $0.itemType == .exercise }
        ForEach(Array(related.enumerated()), id: \.element.id) { index, entry in
          HairlineDivider().padding(.leading, IntradaSpacing.card)
          nestedRow(
            entry, isFirst: index == 0, isLast: index == related.count - 1,
            onMoveUp: { moveExerciseWithinBlock(entry, by: -1, in: block) },
            onMoveDown: { moveExerciseWithinBlock(entry, by: 1, in: block) },
            onRemove: { removeEntry(entry) })
        }
        HairlineDivider().padding(.leading, IntradaSpacing.card)
        Button {
          addingExerciseGroupId = groupId
        } label: {
          Label("Add a related exercise", systemImage: "plus")
            .font(IntradaFont.meta).foregroundStyle(IntradaColor.accent)
        }
        .buttonStyle(.plain)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.controlGap)
      }
    }
    .cardSurface(cornerRadius: IntradaRadius.card)
  }

  private func groupHeader(_ block: SetlistBlockView, groupId: String, collapsed: Bool)
    -> some View
  {
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
        Text(collapsed ? block.durationDisplay : expandedSubtitle(block))
          .font(IntradaFont.micro).foregroundStyle(IntradaColor.inkSecondary)
      }
      Spacer(minLength: IntradaSpacing.controlGap)
      blockMenu(groupId: groupId)
      Button {
        withAnimation(IntradaMotion.snappy) {
          if collapsed { collapsedGroups.remove(groupId) } else { collapsedGroups.insert(groupId) }
        }
      } label: {
        Image(systemName: collapsed ? "chevron.down" : "chevron.up")
          .font(IntradaFont.meta).foregroundStyle(IntradaColor.inkFaint).frame(width: 20)
      }
      .buttonStyle(.plain)
      .accessibilityLabel(collapsed ? "Expand block" : "Collapse block")
    }
    .padding(.horizontal, IntradaSpacing.cardCompact)
    .padding(.vertical, IntradaSpacing.cardCompact)
    .background(collapsed ? Color.clear : IntradaColor.surfaceSunken)
  }

  private var groupPill: some View {
    Text("Group")
      .font(IntradaFont.micro).textCase(.uppercase).kerning(0.4)
      .foregroundStyle(IntradaColor.pieceBadgeFg)
      .padding(.horizontal, 6).padding(.vertical, 2)
      .background(
        IntradaColor.pieceBadgeBg, in: RoundedRectangle(cornerRadius: IntradaRadius.badge))
  }

  private func nestedRow(
    _ entry: SetlistEntryView, isFirst: Bool, isLast: Bool, onMoveUp: @escaping () -> Void,
    onMoveDown: @escaping () -> Void, onRemove: @escaping () -> Void
  ) -> some View {
    HStack(spacing: IntradaSpacing.cardCompact) {
      ItemKind.exercise.bar.frame(width: 3, height: 26).clipShape(Capsule())
      VStack(alignment: .leading, spacing: 1) {
        Text(entry.itemTitle).font(IntradaFont.bodyMedium).foregroundStyle(IntradaColor.ink)
          .lineLimit(1)
        Text(nestedMeta(entry)).font(IntradaFont.micro).foregroundStyle(IntradaColor.inkSecondary)
      }
      Spacer(minLength: 0)
      if editMode == .active {
        nestedRowEditControls(
          entry, isFirst: isFirst, isLast: isLast, onMoveUp: onMoveUp, onMoveDown: onMoveDown,
          onRemove: onRemove)
      }
    }
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.vertical, IntradaSpacing.controlGap)
  }

  private func nestedRowEditControls(
    _ entry: SetlistEntryView, isFirst: Bool, isLast: Bool, onMoveUp: @escaping () -> Void,
    onMoveDown: @escaping () -> Void, onRemove: @escaping () -> Void
  ) -> some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      VStack(spacing: 0) {
        Button(action: onMoveUp) {
          Image(systemName: "chevron.up").imageScale(.small).font(IntradaFont.meta)
            .foregroundStyle(isFirst ? IntradaColor.inkFaint : IntradaColor.inkSecondary)
        }
        .buttonStyle(.plain)
        .disabled(isFirst)
        .accessibilityLabel("Move \(entry.itemTitle) up")
        Button(action: onMoveDown) {
          Image(systemName: "chevron.down").imageScale(.small).font(IntradaFont.meta)
            .foregroundStyle(isLast ? IntradaColor.inkFaint : IntradaColor.inkSecondary)
        }
        .buttonStyle(.plain)
        .disabled(isLast)
        .accessibilityLabel("Move \(entry.itemTitle) down")
      }
      Button(action: onRemove) {
        Image(systemName: "minus.circle").font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.danger)
      }
      .buttonStyle(.plain)
      .accessibilityLabel("Remove \(entry.itemTitle) from today's session")
      Button {
        configuringEntry = EntrySettingsTarget(id: entry.id, entry: entry)
      } label: {
        Image(systemName: "slider.horizontal.3").font(IntradaFont.bodyMedium)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .buttonStyle(.plain)
      .accessibilityLabel("\(entry.itemTitle) settings")
    }
  }

  private func blockMenu(groupId: String) -> some View {
    Menu {
      Button("Just the piece") { store.send(.session(.keepOnlyPiece(groupId: groupId))) }
      Button("Ungroup") { store.send(.session(.ungroupBlock(groupId: groupId))) }
      Button("Remove block", role: .destructive) {
        store.send(.session(.removeBlock(groupId: groupId)))
      }
    } label: {
      Image(systemName: "ellipsis")
        .font(IntradaFont.bodyMedium).foregroundStyle(IntradaColor.inkSecondary)
        .frame(width: 24, height: 24)
    }
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

  private func removeLabel(_ block: SetlistBlockView) -> String {
    if block.groupId != nil { return "Remove \(block.pieceTitle ?? "block")" }
    return "Remove \(block.entries.first?.itemTitle ?? "item")"
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
    "+\(block.relatedCount) related"
  }

  private func expandedSubtitle(_ block: SetlistBlockView) -> String {
    let related = block.relatedCount == 1 ? "1 related" : "\(block.relatedCount) related"
    return "\(related), then piece · \(block.durationDisplay)"
  }

  private func nestedMeta(_ entry: SetlistEntryView) -> String {
    entry.durationDisplay.isEmpty ? "Related" : "Related · \(entry.durationDisplay)"
  }

  // ── Actions ──────────────────────────────────────────────────────────

  private func removeUnit(_ block: SetlistBlockView) {
    let before = store.viewModel?.errorSeq
    if let groupId = block.groupId {
      store.send(.session(.removeBlock(groupId: groupId)))
    } else if let entryId = block.entries.first?.id {
      store.send(.session(.removeFromSetlist(entryId: entryId)))
    }
    if store.viewModel?.errorSeq == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  /// Swaps `entry` with its neighbor within `block`'s related-exercise run
  /// (never past the anchor piece, which is always last in `block.entries`).
  private func moveExerciseWithinBlock(
    _ entry: SetlistEntryView, by delta: Int, in block: SetlistBlockView
  ) {
    let related = block.entries.filter { $0.itemType == .exercise }
    guard let localIndex = related.firstIndex(where: { $0.id == entry.id }) else { return }
    let destLocal = localIndex + delta
    guard related.indices.contains(destLocal) else { return }
    guard let blockStart = entries.firstIndex(where: { $0.id == related[0].id }) else { return }
    let before = store.viewModel?.errorSeq
    store.send(
      .session(.reorderSetlist(entryId: entry.id, newPosition: UInt64(blockStart + destLocal))))
    if store.viewModel?.errorSeq == before {
      UISelectionFeedbackGenerator().selectionChanged()
    }
  }

  private func removeEntry(_ entry: SetlistEntryView) {
    let before = store.viewModel?.errorSeq
    store.send(.session(.removeFromSetlist(entryId: entry.id)))
    if store.viewModel?.errorSeq == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  /// A block drags as a whole via `reorderBlock`; a standalone via `reorderSetlist`.
  private func moveUnits(from source: IndexSet, to destination: Int) {
    guard let from = source.first, units.indices.contains(from) else { return }
    let target = from < destination ? destination - 1 : destination
    guard target != from else { return }
    let unit = units[from]
    let before = store.viewModel?.errorSeq
    if let groupId = unit.block.groupId {
      store.send(.session(.reorderBlock(groupId: groupId, newPosition: UInt64(target))))
    } else if let entryId = unit.block.entries.first?.id {
      var reordered = units
      let moved = reordered.remove(at: from)
      reordered.insert(moved, at: min(target, reordered.count))
      let flatIds = reordered.flatMap { $0.block.entries.map(\.id) }
      if let newIndex = flatIds.firstIndex(of: entryId) {
        store.send(.session(.reorderSetlist(entryId: entryId, newPosition: UInt64(newIndex))))
      }
    }
    if store.viewModel?.errorSeq == before {
      UISelectionFeedbackGenerator().selectionChanged()
    }
  }

  private func cancel() {
    if entries.isEmpty { dismiss() } else { confirmingCancel = true }
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
