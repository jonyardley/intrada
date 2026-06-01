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

/// Segmented pill control: the indigo selected pill slides between options via
/// a shared `matchedGeometryEffect`. Spring approximates the iOS default.
struct LibraryFilterTabs: View {
  @Binding var selection: LibraryFilter
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Namespace private var pill

  var body: some View {
    HStack(spacing: 8) {
      ForEach(LibraryFilter.allCases) { filter in
        let isSelected = filter == selection
        Button {
          withAnimation(reduceMotion ? nil : .spring(response: 0.35, dampingFraction: 0.8)) {
            selection = filter
          }
        } label: {
          Text(filter.label)
            .font(.system(size: 13, weight: .medium))
            .foregroundStyle(isSelected ? IntradaColor.onAccent : IntradaColor.inkFaint)
            .padding(.vertical, 6)
            .padding(.horizontal, 14)
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
