import SharedTypes
import SwiftUI

// Library-first builder: browse/search the library up top, tap to add, and the
// setlist forms in the queue tray below. A piece's related exercises arrive as
// one block (related-first, piece-last); standalone items render as plain rows.
// `cancelBuilding` fires from PracticeScreen's navigation binding on dismiss.
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
    let index: Int
    let block: SetlistBlockView
  }

  private var units: [BuilderUnit] {
    blocks.enumerated().map { offset, block in
      BuilderUnit(id: block.entries.first?.id ?? "unit-\(offset)", index: offset, block: block)
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

  // ── Queue tray (units: blocks + standalone items) ────────────────────

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
        ScrollView {
          VStack(spacing: IntradaSpacing.controlGap) {
            ForEach(units) { unit in
              unitView(unit.block, at: unit.index)
            }
          }
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.vertical, IntradaSpacing.cardCompact)
        }
        .frame(maxHeight: 240)
      }
    }
    .frame(maxWidth: .infinity)
    .background(IntradaColor.paperTop)
    .overlay(alignment: .top) { Rectangle().fill(IntradaColor.hairline).frame(height: 1) }
    .cardShadow(above: true)
  }

  @ViewBuilder private func unitView(_ block: SetlistBlockView, at index: Int) -> some View {
    if block.groupId == nil, let entry = block.entries.first {
      HStack(spacing: IntradaSpacing.controlGap) {
        SetlistQueueRow(entry: entry) {
          store.send(.session(.removeFromSetlist(entryId: entry.id)))
        }
        reorderControls(at: index)
          .padding(.trailing, IntradaSpacing.cardCompact)
      }
      .background(IntradaColor.cardFill)
      .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
    } else {
      blockCard(block, at: index)
    }
  }

  private func blockCard(_ block: SetlistBlockView, at index: Int) -> some View {
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
        reorderControls(at: index)
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

  private func reorderControls(at index: Int) -> some View {
    HStack(spacing: 2) {
      Button { moveUnit(at: index, by: -1) } label: {
        Image(systemName: "chevron.up").font(IntradaFont.micro)
      }
      .disabled(index == 0)
      .accessibilityLabel("Move up")
      Button { moveUnit(at: index, by: 1) } label: {
        Image(systemName: "chevron.down").font(IntradaFont.micro)
      }
      .disabled(index == blocks.count - 1)
      .accessibilityLabel("Move down")
    }
    .buttonStyle(.plain)
    .foregroundStyle(IntradaColor.inkFaint)
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

  /// A block moves as a unit via `reorderBlock`; a standalone item is a single
  /// entry, so it slides to the neighbour unit's edge position via
  /// `reorderSetlist`.
  private func moveUnit(at index: Int, by delta: Int) {
    guard index < blocks.count else { return }
    let dest = index + delta
    guard dest >= 0, dest < blocks.count else { return }
    let unit = blocks[index]
    let before = store.viewModel?.error
    if let groupId = unit.groupId {
      store.send(.session(.reorderBlock(groupId: groupId, newPosition: UInt64(dest))))
    } else if let entryId = unit.entries.first?.id {
      let neighbour = blocks[dest]
      let target = delta < 0 ? neighbour.entries.first : neighbour.entries.last
      guard let position = target?.position else { return }
      store.send(.session(.reorderSetlist(entryId: entryId, newPosition: position)))
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
