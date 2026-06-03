import SharedTypes
import SwiftUI

/// The library's type filter. `kind` is `nil` for All; the screen maps it to a
/// core `ListQuery` so the core does the filtering, not the shell.
enum LibraryFilter: CaseIterable, Identifiable {
  case all, pieces, exercises

  var id: Self { self }

  init(kind: ItemKind?) {
    switch kind {
    case .none: self = .all
    case .piece: self = .pieces
    case .exercise: self = .exercises
    }
  }

  var label: String {
    switch self {
    case .all: "All"
    case .pieces: "Pieces"
    case .exercises: "Exercises"
    }
  }

  var kind: ItemKind? {
    switch self {
    case .all: nil
    case .pieces: .piece
    case .exercises: .exercise
    }
  }
}

struct LibraryFilterTabs: View {
  @Binding var selection: LibraryFilter
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Namespace private var pill

  var body: some View {
    // Horizontal scroll so the pills slide rather than wrap mid-word once the
    // labels grow past the width at large Dynamic Type (#810).
    ScrollView(.horizontal, showsIndicators: false) {
      HStack(spacing: IntradaSpacing.controlGap) {
        ForEach(LibraryFilter.allCases) { filter in
          let isSelected = filter == selection
          Button {
            withAnimation(reduceMotion ? nil : .spring(response: 0.35, dampingFraction: 0.8)) {
              selection = filter
            }
          } label: {
            Text(filter.label)
              .font(IntradaFont.tab)
              .lineLimit(1)
              .foregroundStyle(isSelected ? IntradaColor.onAccent : IntradaColor.inkFaint)
              .padding(.vertical, 6)
              .padding(.horizontal, IntradaSpacing.row)
              .background {
                if isSelected {
                  Capsule()
                    .fill(IntradaColor.accent)
                    .matchedGeometryEffect(id: "selectedPill", in: pill)
                }
              }
          }
          .buttonStyle(.plain)
          .accessibilityLabel(filter.label)
          .accessibilityAddTraits(isSelected ? [.isSelected] : [])
        }
      }
    }
  }
}

#if DEBUG
  private struct LibraryFilterTabsPreview: View {
    @State private var selection: LibraryFilter = .pieces
    var body: some View {
      ZStack {
        PaperBackground()
        LibraryFilterTabs(selection: $selection)
      }
    }
  }

  #Preview {
    LibraryFilterTabsPreview()
  }
#endif
