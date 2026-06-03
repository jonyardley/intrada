import SharedTypes
import SwiftUI

// Cancel/back isn't handled here: PracticeScreen's navigation binding sends
// `cancelBuilding` when this screen pops.
struct SessionBuilderScreen: View {
  @Environment(Store.self) private var store
  @State private var picking = false

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
    .sheet(isPresented: $picking) {
      SessionItemPickerSheet().environment(store)
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
              top: IntradaSpacing.controlGap, leading: IntradaSpacing.card,
              bottom: IntradaSpacing.controlGap, trailing: IntradaSpacing.card)
          )
          .swipeActions(edge: .trailing) {
            Button(role: .destructive) {
              store.send(.session(.removeFromSetlist(entryId: entry.id)))
            } label: {
              Label("Remove", systemImage: "trash")
            }
          }
      }
    }
    .listStyle(.plain)
    .scrollContentBackground(.hidden)
  }

  // Present-but-disabled until the player exists — the one-primary-action
  // frontier, moved down a level from the Practice tab.
  private var startBar: some View {
    VStack(spacing: 6) {
      Label("Start session", systemImage: "play.fill")
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.onAccent)
        .frame(maxWidth: .infinity)
        .padding(.vertical, IntradaSpacing.row)
        .background(LinearGradient.brandBar)
        .clipShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
        .opacity(0.5)
      Text("Coming soon")
        .font(IntradaFont.micro)
        .foregroundStyle(IntradaColor.inkFaint)
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel("Start session, coming soon")
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
