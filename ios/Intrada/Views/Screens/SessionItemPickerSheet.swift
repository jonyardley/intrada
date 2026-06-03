import SharedTypes
import SwiftUI

struct SessionItemPickerSheet: View {
  @Environment(Store.self) private var store
  @Environment(\.dismiss) private var dismiss
  @State private var addError: String?

  private var items: [LibraryItemView] { store.viewModel?.items ?? [] }

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
          Button("Done") { dismiss() }
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
      .scrollEdgeShadow()
    }
  }

  // The global error banner lives in RootView, behind this sheet — surface the
  // add failure here, and only ack the tap once the core confirms the add.
  private func add(_ item: LibraryItemView) {
    let before = store.viewModel?.error
    store.send(.session(.addToSetlist(itemId: item.id)))
    let after = store.viewModel?.error
    if let after, after != before {
      addError = after
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
