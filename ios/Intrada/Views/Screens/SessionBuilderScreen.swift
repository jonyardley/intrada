import SharedTypes
import SwiftUI

// Library-first builder: browse + tap to add; the setlist forms below, where a
// piece's related exercises arrive as one draggable block.
struct SessionBuilderScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var confirmingCancel = false
  // `SharedTypes`' domain `Set` (setlists) shadows `Swift.Set` here.
  @State private var collapsedGroups: Swift.Set<String> = []

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }
  private var entries: [SetlistEntryView] { store.viewModel?.buildingSetlist?.entries ?? [] }
  private var blocks: [SetlistBlockView] { store.viewModel?.buildingSetlist?.blocks ?? [] }
  private var hasBlocks: Bool { blocks.contains { $0.groupId != nil } }

  private var entryByItem: [String: String] {
    Dictionary(entries.map { ($0.itemId, $0.id) }, uniquingKeysWith: { first, _ in first })
  }

  /// Stable-identity units for the queue `ForEach` — keying on the first entry's
  /// ulid (not the array index) avoids the SwiftUI out-of-range crash when an
  /// item is removed and `blocks` shrinks mid-render.
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
    ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        BrowseControlsBar(elevated: true)
        library
        queueTray
      }
    }
    .navigationTitle("New session")
    .navigationBarTitleDisplayMode(.inline)
    .navigationBarBackButtonHidden(true)
    .toolbar {
      ToolbarItem(placement: .topBarLeading) {
        Button("Cancel") { cancel() }
      }
      if hasBlocks {
        ToolbarItem(placement: .topBarTrailing) {
          Button("Ungroup all") { store.send(.session(.ungroupAllBlocks)) }
        }
      }
    }
    .alert("Discard session plan?", isPresented: $confirmingCancel) {
      Button("Discard", role: .destructive) { dismiss() }
      Button("Keep editing", role: .cancel) {}
    } message: {
      Text("The items you've added will be cleared.")
    }
  }

  // ── Library (browse + add) ───────────────────────────────────────────

  @ViewBuilder private var library: some View {
    if items.isEmpty {
      PlaceholderContent(
        systemImage: isSearching ? "magnifyingglass" : "books.vertical", message: emptyMessage
      )
      .frame(maxWidth: .infinity, maxHeight: .infinity)
    } else {
      ScrollView {
        LazyVStack(spacing: IntradaSpacing.cardCompact) {
          ForEach(items, id: \.id) { item in
            libraryRow(item)
          }
        }
        .padding(.horizontal, IntradaSpacing.card)
        .padding(.vertical, IntradaSpacing.card)
      }
      .scrollDismissesKeyboard(.interactively)
      .frame(maxHeight: .infinity)
    }
  }

  private func libraryRow(_ item: LibraryItemView) -> some View {
    let added = entryByItem[item.id] != nil
    return Button {
      toggle(item)
    } label: {
      LibraryItemCard(item: item)
        .overlay(alignment: .trailing) {
          Image(systemName: added ? "checkmark.circle.fill" : "plus.circle")
            .font(.title2)
            .foregroundStyle(added ? IntradaColor.accent : IntradaColor.inkFaint)
            .padding(.trailing, IntradaSpacing.card)
            .accessibilityHidden(true)
        }
        .overlay(
          RoundedRectangle(cornerRadius: IntradaRadius.card)
            .stroke(IntradaColor.accent, lineWidth: 2).opacity(added ? 1 : 0)
        )
        .cardShadow()
    }
    .buttonStyle(.plain)
    .accessibilityValue(added ? "Added" : "Not added")
    .accessibilityHint(added ? "Removes it from the session" : "Adds it to the session")
  }

  // ── Queue tray (units: blocks + standalone items, drag-reorderable) ──

  private var queueTray: some View {
    VStack(alignment: .leading, spacing: 0) {
      HStack {
        Text("In this session").font(IntradaFont.cardTitle(16)).foregroundStyle(IntradaColor.ink)
        Spacer()
        startButton
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.cardCompact)

      if blocks.isEmpty {
        Text("Tap items above to build your session.")
          .font(IntradaFont.meta).foregroundStyle(IntradaColor.inkFaint)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(IntradaSpacing.card)
      } else {
        List {
          ForEach(units) { unit in
            unitView(unit.block)
              .listRowBackground(Color.clear)
              .listRowSeparator(.hidden)
              .listRowInsets(
                EdgeInsets(
                  top: IntradaSpacing.controlGap / 2, leading: IntradaSpacing.card,
                  bottom: IntradaSpacing.controlGap / 2, trailing: IntradaSpacing.card))
          }
          .onMove(perform: moveUnits)
        }
        .listStyle(.plain)
        .scrollContentBackground(.hidden)
        // Edit-mode grips drag whole units (a block card as one); no `.onDelete`.
        .environment(\.editMode, .constant(.active))
        .frame(maxHeight: 280)
      }
    }
    .frame(maxWidth: .infinity)
    .background(IntradaColor.paperTop)
    .overlay(alignment: .top) { Rectangle().fill(IntradaColor.hairline).frame(height: 1) }
    .cardShadow(above: true)
  }

  @ViewBuilder private func unitView(_ block: SetlistBlockView) -> some View {
    if block.groupId == nil, let entry = block.entries.first {
      SetlistQueueRow(entry: entry) {
        store.send(.session(.removeFromSetlist(entryId: entry.id)))
      }
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .background(IntradaColor.cardFill)
      .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    } else {
      blockCard(block)
    }
  }

  private func blockCard(_ block: SetlistBlockView) -> some View {
    let groupId = block.groupId ?? ""
    let collapsed = collapsedGroups.contains(groupId)
    return VStack(spacing: 0) {
      HStack(spacing: IntradaSpacing.controlGap) {
        Button {
          if collapsed { collapsedGroups.remove(groupId) } else { collapsedGroups.insert(groupId) }
        } label: {
          Image(systemName: collapsed ? "chevron.right" : "chevron.down")
            .font(IntradaFont.meta)
            .foregroundStyle(IntradaColor.inkFaint)
            .frame(width: 16)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(collapsed ? "Expand block" : "Collapse block")

        VStack(alignment: .leading, spacing: 1) {
          Text(block.pieceTitle ?? "Related exercises")
            .font(IntradaFont.bodyMedium)
            .foregroundStyle(IntradaColor.ink)
            .lineLimit(1)
          Text(blockSubtitle(block))
            .font(IntradaFont.micro)
            .foregroundStyle(IntradaColor.inkFaint)
        }
        Spacer(minLength: IntradaSpacing.controlGap)
        blockMenu(groupId: groupId)
      }
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .padding(.vertical, IntradaSpacing.controlGap)

      if !collapsed {
        ForEach(block.entries, id: \.id) { entry in
          HairlineDivider().padding(.leading, IntradaSpacing.card)
          SetlistQueueRow(entry: entry) {
            store.send(.session(.removeFromSetlist(entryId: entry.id)))
          }
          .padding(.horizontal, IntradaSpacing.cardCompact)
        }
      }
    }
    .background(IntradaColor.cardFill)
    .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    .overlay(alignment: .leading) {
      Rectangle().fill(IntradaColor.accent).frame(width: 3)
        .clipShape(RoundedRectangle(cornerRadius: 1.5))
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
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.inkSecondary)
        .frame(width: 24, height: 24)
    }
    .accessibilityLabel("Block actions")
  }

  private func blockSubtitle(_ block: SetlistBlockView) -> String {
    let related = block.relatedCount == 1 ? "1 related" : "\(block.relatedCount) related"
    return "\(related), then piece · \(block.durationDisplay)"
  }

  // The one-primary-action frontier. `startSession` flips the core Building →
  // Active; `buildingSetlist` goes nil (this screen auto-pops) and `activeSession`
  // goes non-nil (RootView presents the player). State-driven — no local nav flag.
  private var startButton: some View {
    Button {
      store.send(.session(.startSession(now: SessionClock.nowRFC3339())))
    } label: {
      HStack(spacing: 5) {
        Image(systemName: "play.fill").font(IntradaFont.micro)
        Text(entries.isEmpty ? "Start" : "Start · \(entries.count)")
          .font(IntradaFont.metaMedium)
      }
      .foregroundStyle(IntradaColor.onAccent)
      .padding(.vertical, 6)
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .background(LinearGradient.brandBar, in: Capsule())
      .opacity(entries.isEmpty ? 0.5 : 1)
      .cardShadow()
    }
    .buttonStyle(.plain)
    .disabled(entries.isEmpty)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Start session")
    .accessibilityValue(entries.isEmpty ? "No items yet" : "\(entries.count) items")
  }

  // ── Actions ──────────────────────────────────────────────────────────

  private func toggle(_ item: LibraryItemView) {
    let before = store.viewModel?.error
    if let entryId = entryByItem[item.id] {
      store.send(.session(.removeFromSetlist(entryId: entryId)))
    } else {
      store.send(.session(.addToSetlist(itemId: item.id)))
    }
    // Only ack the tap once the core confirms (errors surface in RootView's banner).
    if store.viewModel?.error == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }

  /// A block drags as a whole via `reorderBlock`; a standalone via `reorderSetlist`.
  private func moveUnits(from source: IndexSet, to destination: Int) {
    guard let from = source.first, units.indices.contains(from) else { return }
    let target = from < destination ? destination - 1 : destination
    guard target != from else { return }
    let unit = units[from]
    let before = store.viewModel?.error
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
    if store.viewModel?.error == before {
      UISelectionFeedbackGenerator().selectionChanged()
    }
  }

  private func cancel() {
    if entries.isEmpty { dismiss() } else { confirmingCancel = true }
  }

  private var isSearching: Bool { !(store.viewModel?.activeQuery?.text ?? "").isEmpty }

  private var emptyMessage: String {
    if let text = store.viewModel?.activeQuery?.text, !text.isEmpty {
      return "No items match “\(text)”."
    }
    return "Your library is empty — add pieces and exercises first."
  }
}

#if DEBUG
  #Preview("Populated") {
    NavigationStack { SessionBuilderScreen() }.environment(Store.previewBuilding)
  }
#endif
