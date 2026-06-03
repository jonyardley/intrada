import SharedTypes
import SwiftUI

struct SessionItemPickerSheet: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var addError: String?

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }

  // Binary membership: the core doesn't dedupe by item id (#939); keep first.
  private var entryByItem: [String: String] {
    let entries = store.viewModel?.buildingSetlist?.entries ?? []
    return Dictionary(entries.map { ($0.itemId, $0.id) }, uniquingKeysWith: { first, _ in first })
  }

  private var addedCount: Int { store.viewModel?.buildingSetlist?.entries.count ?? 0 }

  var body: some View {
    NavigationStack {
      ZStack {
        PaperBackground()
        VStack(spacing: 0) {
          if let addError {
            FormErrorBanner(message: addError)
              .padding(.horizontal, IntradaSpacing.card)
              .padding(.top, IntradaSpacing.card)
          }
          content
        }
      }
      .navigationTitle("Add to session")
      .navigationBarTitleDisplayMode(.inline)
      .toolbar {
        ToolbarItem(placement: .topBarTrailing) {
          Button(addedCount > 0 ? "Done · \(addedCount)" : "Done") { dismiss() }
            .accessibilityIdentifier("sessionPickerDone")
        }
      }
    }
  }

  @ViewBuilder private var content: some View {
    if items.isEmpty {
      PlaceholderContent(
        systemImage: "books.vertical",
        message: "Your library is empty — add pieces and exercises first.")
    } else {
      ScrollView {
        LazyVStack(spacing: IntradaSpacing.row) {
          ForEach(items, id: \.id) { item in
            row(item)
          }
        }
        .padding(IntradaSpacing.card)
      }
      .scrollEdgeShadow()
    }
  }

  private func row(_ item: LibraryItemView) -> some View {
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
            .stroke(IntradaColor.accent, lineWidth: 2)
            .opacity(added ? 1 : 0)
        )
    }
    .buttonStyle(.plain)
    .accessibilityValue(added ? "Added" : "Not added")
    .accessibilityHint(added ? "Removes it from the session" : "Adds it to the session")
  }

  // RootView's error banner sits behind this sheet — surface failures here.
  private func toggle(_ item: LibraryItemView) {
    let before = store.viewModel?.error
    if let entryId = entryByItem[item.id] {
      store.send(.session(.removeFromSetlist(entryId: entryId)))
    } else {
      store.send(.session(.addToSetlist(itemId: item.id)))
    }
    if let error = store.viewModel?.error, error != before {
      addError = error
    } else {
      addError = nil
      UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
  }
}

#if DEBUG
  #Preview {
    SessionItemPickerSheet()
      .environment(Store.previewLibrary)
  }
#endif
