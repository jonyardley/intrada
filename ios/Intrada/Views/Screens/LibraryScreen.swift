import SharedTypes
import SwiftUI

/// The live item count proves the core bridge still feeds the new tab shell.
struct LibraryScreen: View {
  @Environment(Store.self) private var store

  var body: some View {
    ScreenScaffold(title: "Library", subtitle: subtitle) {
      PlaceholderContent(
        systemImage: "books.vertical",
        message: "Your pieces and exercises will live here.")
    }
  }

  private var subtitle: String? {
    guard let viewModel = store.viewModel else { return nil }
    let count = viewModel.items.count
    return "\(count) \(count == 1 ? "item" : "items")"
  }
}

#if DEBUG
  #Preview {
    LibraryScreen()
      .environment(Store.preview)
  }
#endif
