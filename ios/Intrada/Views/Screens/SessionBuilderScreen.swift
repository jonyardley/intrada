import SharedTypes
import SwiftUI

// `cancelBuilding` fires from PracticeScreen's navigation binding on dismiss.
struct SessionBuilderScreen: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var picking = false
  @State private var confirmingCancel = false

  private var entries: [SetlistEntryView] { store.viewModel?.buildingSetlist?.entries ?? [] }

  var body: some View {
    ScreenScaffold(
      title: "New session",
      subtitle: subtitle,
      trailing: .init(label: "Add items", action: { picking = true })
    ) {
      VStack(spacing: 0) {
        if entries.isEmpty {
          PlaceholderContent(
            systemImage: "music.note.list",
            message: "No items yet — tap + to add pieces and exercises to practise.")
        } else {
          setlist
        }
        startBar
      }
    }
    .navigationBarTitleDisplayMode(.inline)
    // Hiding the back chevron also disables the swipe-back that bypasses cancel().
    .navigationBarBackButtonHidden(true)
    .toolbar {
      ToolbarItem(placement: .topBarLeading) {
        Button("Cancel") { cancel() }
      }
      ToolbarItem(placement: .topBarTrailing) {
        if entries.count > 1 { EditButton() }
      }
    }
    .sheet(isPresented: $picking) {
      SessionItemPickerSheet().environment(store)
    }
    // Alert (not confirmationDialog) so the buttons show on iPad/regular width.
    .alert("Discard session plan?", isPresented: $confirmingCancel) {
      Button("Discard", role: .destructive) { dismiss() }
      Button("Keep editing", role: .cancel) {}
    } message: {
      Text("The items you've added will be cleared.")
    }
  }

  private func cancel() {
    if entries.isEmpty {
      dismiss()
    } else {
      confirmingCancel = true
    }
  }

  private var setlist: some View {
    List {
      ForEach(entries, id: \.id) { entry in
        SetlistEntryRow(entry: entry)
          .listRowBackground(Color.clear)
          .listRowSeparator(.hidden)
          .listRowInsets(
            EdgeInsets(
              top: 0, leading: IntradaSpacing.card, bottom: 0, trailing: IntradaSpacing.card)
          )
          .swipeActions(edge: .trailing) {
            Button(role: .destructive) {
              store.send(.session(.removeFromSetlist(entryId: entry.id)))
            } label: {
              Label("Remove", systemImage: "trash")
            }
          }
      }
      .onMove(perform: move)
    }
    .listStyle(.plain)
    .scrollContentBackground(.hidden)
  }

  // SwiftUI's `to` is an insert-before index; reorderSetlist wants a final index.
  private func move(_ source: IndexSet, to destination: Int) {
    guard let from = source.first else { return }
    let target = from < destination ? destination - 1 : destination
    store.send(
      .session(.reorderSetlist(entryId: entries[from].id, newPosition: UInt64(target))))
  }

  // The one-primary-action frontier. `startSession` flips the core Building →
  // Active; `buildingSetlist` goes nil (this screen auto-pops) and `activeSession`
  // goes non-nil (RootView presents the player). State-driven — no local nav flag.
  private var startBar: some View {
    Button {
      store.send(.session(.startSession(now: SessionClock.nowRFC3339())))
    } label: {
      Label("Start session", systemImage: "play.fill")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.row)
        .background(LinearGradient.brandBar)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
        .opacity(entries.isEmpty ? 0.5 : 1)
    }
    .buttonStyle(.plain)
    .disabled(entries.isEmpty)
    .accessibilityLabel("Start session")
    .padding(IntradaSpacing.card)
  }

  private var subtitle: String {
    let count = entries.count
    return count == 0 ? "No items yet" : "\(count) item\(count == 1 ? "" : "s")"
  }
}

#if DEBUG
  #Preview("Populated") {
    NavigationStack {
      SessionBuilderScreen()
    }
    .environment(Store.previewBuilding)
  }

  #Preview("Empty") {
    NavigationStack {
      SessionBuilderScreen()
    }
    .environment(Store.preview)
  }
#endif
