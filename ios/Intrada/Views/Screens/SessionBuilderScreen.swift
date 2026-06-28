import SharedTypes
import SwiftUI

// Library-first builder: browse/search the library up top, tap to add, and the
// setlist forms in the reorderable queue tray at the bottom. `cancelBuilding`
// fires from PracticeScreen's navigation binding on dismiss.
struct SessionBuilderScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var confirmingCancel = false

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }
  private var entries: [SetlistEntryView] { store.viewModel?.buildingSetlist?.entries ?? [] }

  private var entryByItem: [String: String] {
    let entries = store.viewModel?.buildingSetlist?.entries ?? []
    return Dictionary(entries.map { ($0.itemId, $0.id) }, uniquingKeysWith: { first, _ in first })
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

  // ── Queue tray (the setlist, reorderable) ────────────────────────────

  private var queueTray: some View {
    VStack(alignment: .leading, spacing: 0) {
      HStack {
        Text("In this session").font(IntradaFont.cardTitle(16)).foregroundStyle(IntradaColor.ink)
        Spacer()
        startButton
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.top, IntradaSpacing.cardCompact)

      if entries.isEmpty {
        Text("Tap items above to build your session.")
          .font(IntradaFont.meta).foregroundStyle(IntradaColor.inkFaint)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(IntradaSpacing.card)
      } else {
        List {
          ForEach(entries, id: \.id) { entry in
            SetlistQueueRow(entry: entry) {
              store.send(.session(.removeFromSetlist(entryId: entry.id)))
            }
            .listRowBackground(Color.clear)
            .listRowSeparator(.hidden)
            .listRowInsets(
              EdgeInsets(
                top: 0, leading: IntradaSpacing.card, bottom: 0, trailing: IntradaSpacing.card))
          }
          .onMove(perform: move)
        }
        .listStyle(.plain)
        .scrollContentBackground(.hidden)
        // Always-on reorder grips. No `.onDelete` — remove is the inline button,
        // so edit mode adds no second delete control.
        .environment(\.editMode, .constant(.active))
        .frame(height: min(CGFloat(entries.count) * 44 + 8, 176))
      }
    }
    .frame(maxWidth: .infinity)
    .background(IntradaColor.paperTop)
    .overlay(alignment: .top) { Rectangle().fill(IntradaColor.hairline).frame(height: 1) }
    .cardShadow(above: true)
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

  // SwiftUI's `to` is an insert-before index; reorderSetlist wants a final index.
  private func move(_ source: IndexSet, to destination: Int) {
    guard let from = source.first else { return }
    let target = from < destination ? destination - 1 : destination
    store.send(.session(.reorderSetlist(entryId: entries[from].id, newPosition: UInt64(target))))
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
