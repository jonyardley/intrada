import SwiftUI

/// Pull-down type filter (All / Pieces / Exercises) — the dropdown sibling of
/// `LibrarySortMenu`. The label shows the active filter so the current scope is
/// always visible.
struct LibraryFilterMenu: View {
  let current: LibraryFilter
  let onChange: (LibraryFilter) -> Void

  var body: some View {
    Menu {
      ForEach(LibraryFilter.allCases) { filter in
        Button {
          onChange(filter)
        } label: {
          if filter == current {
            Label(filter.label, systemImage: "checkmark")
          } else {
            Text(filter.label)
          }
        }
      }
    } label: {
      // Reserve the slot at the widest option so the label's width never changes
      // when the selection does — otherwise the frame animates the width down and
      // the text clips/snaps mid-transition.
      ZStack(alignment: .leading) {
        ForEach(LibraryFilter.allCases) { option in
          labelContent(option.label).hidden()
        }
        labelContent(current.label)
      }
      .padding(.vertical, IntradaSpacing.controlGap)
      .fixedSize()
    }
    .accessibilityLabel("Filter by type")
    .accessibilityValue(current.label)
  }

  private func labelContent(_ text: String) -> some View {
    HStack(spacing: 4) {
      Text(text)
        .font(IntradaFont.segment)
      Image(systemName: "chevron.down")
        .font(.system(size: 11, weight: .semibold))
        .foregroundStyle(IntradaColor.inkFaint)
    }
    .foregroundStyle(IntradaColor.ink)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      LibraryFilterMenu(current: .all, onChange: { _ in })
    }
  }
#endif
