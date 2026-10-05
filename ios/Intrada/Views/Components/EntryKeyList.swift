import SharedTypes
import SwiftUI

/// The key one entry is planned in (#2249). Pushed from the item sheet; a tap
/// sends and steps back.
struct EntryKeyList: View {
  let entryId: String
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  private var keysView: EntryKeysView? {
    store.viewModel?.buildingSetlist?.entryKeys.first { $0.entryId == entryId }
  }

  private var choices: [KeyChoiceView] { keysView?.keys ?? [] }

  var body: some View {
    ZStack {
      PaperBackground()
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
          VStack(spacing: 0) {
            ForEach(Array(choices.enumerated()), id: \.element.label) { index, choice in
              row(choice)
              if index < choices.count - 1 {
                HairlineDivider().padding(.leading, IntradaSpacing.card)
              }
            }
          }
          .cardSurface()
          Text("The keys you keep for this piece. Add more on the piece's page.")
            .font(IntradaFont.small)
            .foregroundStyle(IntradaColor.inkSecondary)
            .padding(.horizontal, IntradaSpacing.controlGap)
        }
        .padding(IntradaSpacing.card)
      }
    }
    .navigationTitle("Key")
    .navigationBarTitleDisplayMode(.inline)
  }

  private func row(_ choice: KeyChoiceView) -> some View {
    PickerRow(
      label: choice.label, caption: choice.caption, isCurrent: choice.key == keysView?.current,
      hint: "Plans this key", identifier: "entryKey.row"
    ) {
      choose(choice.key)
    }
  }

  private func choose(_ key: Key?) {
    guard key != keysView?.current else {
      dismiss()
      return
    }
    if store.send(.session(.setEntryKey(entryId: entryId, key: key)), onSuccess: .selection) {
      dismiss()
    }
  }
}
