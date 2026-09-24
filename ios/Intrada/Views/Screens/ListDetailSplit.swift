import SwiftUI

/// The iPad list beside its detail: each column keeps its own stack, so a push
/// in the detail stays in the detail.
struct ListDetailSplit<ListContent: View, Detail: View>: View {
  @ViewBuilder let list: () -> ListContent
  @ViewBuilder let detail: () -> Detail

  var body: some View {
    HStack(spacing: 0) {
      NavigationStack { list() }
        .frame(maxWidth: 380)
      // PaperBackground ignores the safe area, so without this the line
      // runs up behind the tabs instead of starting below them (#1682).
      Divider().safeAreaPadding(.top)
      NavigationStack { detail() }
        .frame(maxWidth: .infinity)
    }
  }
}

struct SplitDetailPlaceholder: View {
  let message: String

  var body: some View {
    ZStack {
      PaperBackground()
      PlaceholderContent(
        systemImage: "sidebar.left", message: message, glyphTint: IntradaColor.inkFainter)
    }
    // ScreenScaffold forces the same blank inline bar (#1724, #1822); match
    // that here so nothing jumps when a selection lands.
    .navigationBarTitleDisplayMode(.inline)
    .navigationTitle("")
    .toolbar(.visible, for: .navigationBar)
  }
}

extension View {
  func splitSelected(_ isSelected: Bool) -> some View {
    overlay(
      RoundedRectangle(cornerRadius: IntradaRadius.card)
        .stroke(IntradaColor.accent, lineWidth: 2)
        .opacity(isSelected ? 1 : 0)
    )
    .accessibilityAddTraits(isSelected ? .isSelected : [])
  }
}
