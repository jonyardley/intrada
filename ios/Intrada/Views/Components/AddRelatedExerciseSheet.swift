import SharedTypes
import SwiftUI

/// Single-select picker for adding one more exercise into an already-present
/// session-builder block, opened from the block's own "Add a related
/// exercise" footer row. Distinct from `AddToSessionSheet`: hard-filtered to
/// exercises not already in the setlist, and tapping a row adds-and-dismisses
/// rather than toggling a persistent multi-select list.
struct AddRelatedExerciseSheet: View {
  let groupId: String
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  private var candidates: [LibraryItemView] {
    let inSetlist = Swift.Set((store.viewModel?.buildingSetlist?.entries ?? []).map(\.itemId))
    return (store.viewModel?.items ?? [])
      .filter { $0.itemType == .exercise && !inSetlist.contains($0.id) }
  }

  var body: some View {
    BottomSheet(title: "Add a related exercise", detents: [.large]) {
      if candidates.isEmpty {
        PlaceholderContent(
          systemImage: "dumbbell", message: "No other exercises left to add.")
      } else {
        ScrollView {
          LazyVStack(spacing: IntradaSpacing.cardCompact) {
            ForEach(candidates, id: \.id) { item in
              Button {
                add(item)
              } label: {
                LibraryItemCard(item: item)
              }
              .buttonStyle(.plain)
            }
          }
          .padding(IntradaSpacing.card)
        }
      }
    }
  }

  private func add(_ item: LibraryItemView) {
    let before = store.viewModel?.errorSeq
    store.send(.session(.addExerciseToBlock(groupId: groupId, itemId: item.id)))
    if store.viewModel?.errorSeq == before {
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
      dismiss()
    }
  }
}
