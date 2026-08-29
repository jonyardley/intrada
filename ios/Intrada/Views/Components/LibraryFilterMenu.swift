import SwiftUI

/// Pull-down type filter — the dropdown sibling of `LibrarySortMenu`.
struct LibraryFilterMenu: View {
  @Environment(\.dynamicTypeSize) private var dynamicTypeSize
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
      menuLabel
        // The reservation copies are visual-only; the Menu's explicit label/value
        // below own VoiceOver (`.hidden()` alone leaves them in the a11y tree).
        .accessibilityHidden(true)
        .padding(.vertical, IntradaSpacing.controlGap)
    }
    .accessibilityLabel("Filter by type")
    .accessibilityValue(current.label)
  }

  /// Reserving the widest option stops the label's width animating when the
  /// selection changes, but `fixedSize` made the browse bar un-shrinkable at
  /// accessibility sizes and pushed the screen off its leading edge (#1470).
  @ViewBuilder private var menuLabel: some View {
    if dynamicTypeSize.isAccessibilitySize {
      labelContent(current.label)
    } else {
      ZStack(alignment: .leading) {
        ForEach(LibraryFilter.allCases) { option in
          labelContent(option.label).hidden()
        }
        labelContent(current.label)
      }
      .fixedSize()
    }
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
