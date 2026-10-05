import SharedTypes
import SwiftUI

/// The key one entry is planned in (#2249): the written key, then the keys the
/// item keeps. Pushed from the item sheet; a tap sends and steps back.
struct EntryKeyList: View {
  let entryId: String
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss

  private var choices: [KeyChoiceView] {
    store.viewModel?.buildingSetlist?.entryKeys.first { $0.entryId == entryId }?.keys ?? []
  }

  private var plannedKey: Key? {
    store.viewModel?.buildingSetlist?.entries.first { $0.id == entryId }?.plannedKey
  }

  static func currentLabel(_ choices: [KeyChoiceView], planned: Key?) -> String {
    choices.first { $0.key == planned }?.label ?? "Written key"
  }

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
    let isCurrent = choice.key == plannedKey
    return Button {
      choose(choice.key)
    } label: {
      HStack(spacing: IntradaSpacing.cardCompact) {
        VStack(alignment: .leading, spacing: 2) {
          Text(choice.label)
            .font(isCurrent ? IntradaFont.bodyMedium : IntradaFont.body)
            .foregroundStyle(IntradaColor.ink)
          if let caption = choice.caption {
            Text(caption)
              .font(IntradaFont.small)
              .foregroundStyle(IntradaColor.inkSecondary)
          }
        }
        .multilineTextAlignment(.leading)
        .fixedSize(horizontal: false, vertical: true)
        .frame(maxWidth: .infinity, alignment: .leading)
        if isCurrent {
          Image(systemName: "checkmark")
            .font(IntradaFont.segment.weight(.semibold))
            .foregroundStyle(IntradaColor.ink)
        }
      }
      .padding(.horizontal, IntradaSpacing.card)
      .padding(.vertical, IntradaSpacing.controlGap)
      .frame(minHeight: 52)
      .contentShape(Rectangle())
    }
    .buttonStyle(.plain)
    .accessibilityLabel(choice.caption.map { "\(choice.label), \($0)" } ?? choice.label)
    .accessibilityAddTraits(isCurrent ? .isSelected : [])
    .accessibilityIdentifier("entryKey.row")
  }

  private func choose(_ key: Key?) {
    guard key != plannedKey else {
      dismiss()
      return
    }
    if store.send(.session(.setEntryKey(entryId: entryId, key: key)), onSuccess: .selection) {
      dismiss()
    }
  }
}
