import SwiftUI

/// The library's search field, revealed by the magnifier button in
/// `LibraryScreen`'s header. Cancel clears the text and resigns focus, which the
/// screen reads to tuck the bar away again.
struct LibrarySearchBar: View {
  @Binding var text: String
  var focused: FocusState<Bool>.Binding
  var onCancel: () -> Void

  var body: some View {
    HStack(spacing: 10) {
      HStack(spacing: IntradaSpacing.controlGap) {
        Image(systemName: "magnifyingglass")
          .font(.system(size: 15, weight: .medium))
          .foregroundStyle(IntradaColor.inkFaint)
        TextField("Search library", text: $text)
          .font(IntradaFont.field)
          .foregroundStyle(IntradaColor.ink)
          .focused(focused)
          .submitLabel(.search)
          .textInputAutocapitalization(.never)
          .autocorrectionDisabled()
          .accessibilityLabel("Search library")
        if !text.isEmpty {
          Button {
            text = ""
          } label: {
            Image(systemName: "xmark.circle.fill")
              .font(.system(size: 15))
              .foregroundStyle(IntradaColor.inkFaint)
          }
          .buttonStyle(.plain)
          .accessibilityLabel("Clear search")
        }
      }
      .padding(.vertical, 9)
      .padding(.horizontal, IntradaSpacing.cardCompact)
      .background(Capsule(style: .continuous).fill(IntradaColor.cardFill))
      .overlay(Capsule(style: .continuous).strokeBorder(IntradaColor.hairline, lineWidth: 1))

      Button("Cancel", action: onCancel)
        .font(IntradaFont.bodyMedium)
        .foregroundStyle(IntradaColor.accent)
        .buttonStyle(.plain)
    }
  }
}

#if DEBUG
  private struct LibrarySearchBarPreview: View {
    @State private var text = "clair"
    @FocusState private var focused: Bool
    var body: some View {
      ZStack {
        PaperBackground()
        LibrarySearchBar(text: $text, focused: $focused, onCancel: { text = "" })
          .padding(IntradaSpacing.card)
      }
    }
  }

  #Preview {
    LibrarySearchBarPreview()
  }
#endif
