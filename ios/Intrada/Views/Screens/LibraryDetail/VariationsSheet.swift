import SharedTypes
import SwiftUI

/// Chooses an item's variations from the library's one list, or types a new
/// one (#2247). Ticks save together on Done; renaming a row acts on the whole
/// library at once, so it saves straight away.
struct VariationsSheet: View {
  let item: LibraryItemView

  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var choice: VariationChoice
  @State private var query = ""
  @State private var formError: String?
  @State private var renaming: VariationOptionView?
  @State private var renameText = ""
  private let previewLibrary: [VariationOptionView]?

  init(
    item: LibraryItemView, query: String = "", previewLibrary: [VariationOptionView]? = nil
  ) {
    self.item = item
    self.previewLibrary = previewLibrary
    _choice = State(initialValue: VariationChoice(ids: item.variations.map(\.id)))
    _query = State(initialValue: query)
  }

  private var library: [VariationOptionView] {
    previewLibrary ?? store.viewModel?.variations ?? []
  }

  var body: some View {
    BottomSheet(
      title: "Variations", detents: [.large], dismissesOnDone: false, onDone: save,
      leadingAction: {
        Button("Cancel") { dismiss() }
          .accessibilityIdentifier("variationsSheet.cancel")
      },
      content: { content }
    )
    .alert(
      "Rename \(renaming?.label ?? "variation")", isPresented: renamingShown,
      presenting: renaming
    ) { variation in
      TextField("Name", text: $renameText)
      Button("Rename") { rename(variation) }
      Button("Cancel", role: .cancel) {}
    } message: { _ in
      Text("The new name shows on every piece and exercise that uses it.")
    }
  }

  private var content: some View {
    VStack(spacing: 0) {
      if let formError {
        FormErrorBanner(message: formError)
          .padding(.horizontal, IntradaSpacing.card)
          .padding(.top, IntradaSpacing.cardCompact)
      }
      ScrollView {
        VStack(alignment: .leading, spacing: IntradaSpacing.card) {
          FormField(
            label: "Find or add", text: $query, placeholder: "Left hand, slow practice",
            identifier: "variationsSheet.find"
          )
          .cardSurface()
          if !query.trimmingCharacters(in: .whitespaces).isEmpty {
            matchesCard
          }
          VStack(alignment: .leading, spacing: IntradaSpacing.cardCompact) {
            SectionTitle("All variations")
            allCard
          }
          Text("A new variation joins the list on every piece and exercise.")
            .font(IntradaFont.small)
            .foregroundStyle(IntradaColor.inkSecondary)
        }
        .padding(IntradaSpacing.card)
      }
    }
  }

  private var matchesCard: some View {
    let matches = VariationChoice.matches(query, in: library)
    let addable = choice.addable(query, in: library)
    return VStack(spacing: 0) {
      ForEach(Array(matches.enumerated()), id: \.element.id) { index, variation in
        if index > 0 { HairlineDivider() }
        row(variation.label, caption: variation.usage, chosen: choice.isChosen(variation.id)) {
          choice.toggle(variation.id)
        }
      }
      if let addable {
        if !matches.isEmpty { HairlineDivider() }
        AddRowButton(title: "Add \u{201C}\(addable)\u{201D}", style: .plain) {
          choice.toggleNew(addable)
          query = ""
        }
        .accessibilityLabel("Add \(addable) as a new variation")
        .accessibilityIdentifier("variationsSheet.add")
      } else if matches.isEmpty {
        Text("Already ticked below")
          .font(IntradaFont.secondary)
          .foregroundStyle(IntradaColor.inkSecondary)
          .frame(maxWidth: .infinity, alignment: .leading)
          .padding(IntradaSpacing.card)
      }
    }
    .cardSurface()
  }

  private var allCard: some View {
    VStack(spacing: 0) {
      ForEach(Array(choice.newLabels.enumerated()), id: \.element) { index, label in
        if index > 0 { HairlineDivider() }
        row(label, chosen: true) { choice.toggleNew(label) }
      }
      ForEach(Array(library.enumerated()), id: \.element.id) { index, variation in
        if index > 0 || !choice.newLabels.isEmpty { HairlineDivider() }
        row(variation.label, caption: variation.usage, chosen: choice.isChosen(variation.id)) {
          choice.toggle(variation.id)
        }
        .contextMenu {
          Button("Rename") { startRenaming(variation) }
        }
        .accessibilityAction(named: "Rename") { startRenaming(variation) }
      }
    }
    .cardSurface()
  }

  private func row(
    _ label: String, caption: String? = nil, chosen: Bool, toggle: @escaping () -> Void
  ) -> some View {
    TickRow(
      label: label, caption: caption, chosen: chosen, identifier: "variationsSheet.row",
      toggle: toggle)
  }

  private var renamingShown: Binding<Bool> {
    Binding(get: { renaming != nil }, set: { if !$0 { renaming = nil } })
  }

  private func startRenaming(_ variation: VariationOptionView) {
    renameText = variation.label
    renaming = variation
  }

  private func rename(_ variation: VariationOptionView) {
    _ = send(.variation(.rename(id: variation.id, label: renameText)))
  }

  private func save() {
    let unchanged =
      choice.ids == item.variations.map(\.id) && choice.newLabels.isEmpty
    if unchanged {
      dismiss()
      return
    }
    let saved = send(
      .item(
        .updateItemVariations(
          id: item.id, variationIds: choice.ids, newLabels: choice.newLabels)))
    if saved { dismiss() }
  }

  private func send(_ event: Event) -> Bool {
    formError = nil
    let error = store.sendFromSheet(event)
    withAnimation { formError = error }
    return error == nil
  }
}
