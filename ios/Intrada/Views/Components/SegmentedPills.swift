import SharedTypes
import SwiftUI

/// Shared accent-pill segmented control: a sliding `matchedGeometryEffect`
/// capsule across `options`, so the pill language lives in one place (#807).
struct SegmentedPills<Option: Hashable>: View {
  enum Layout: Equatable {
    /// Leading, horizontally-scrollable tabs that hug their labels.
    case inlineScrolling(edgeInset: CGFloat)
    /// Equal-width segments on a bordered card track.
    case fullWidthTrack
  }

  let options: [Option]
  @Binding var selection: Option
  let label: (Option) -> String
  var hint: ((Option) -> String)?
  var identifier: ((Option) -> String)?
  var font: Font = IntradaFont.smallMedium
  var unselectedColor: Color = IntradaColor.inkSecondary
  var layout: Layout = .inlineScrolling(edgeInset: 0)

  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Environment(\.dynamicTypeSize) private var typeSize
  @Namespace private var pill

  var body: some View {
    switch layout {
    case .inlineScrolling(let edgeInset):
      // Horizontal scroll so the pills slide rather than wrap mid-word once the
      // labels grow past the width at large Dynamic Type (#810).
      ScrollView(.horizontal, showsIndicators: false) { track }
        .contentMargins(.leading, edgeInset, for: .scrollContent)
        .padding(.leading, -edgeInset)
    case .fullWidthTrack:
      if typeSize.isAccessibilitySize {
        ViewThatFits(in: .horizontal) {
          fullWidthTrack(.equalRow)
          fullWidthTrack(.stacked)
        }
      } else {
        fullWidthTrack(.row)
      }
    }
  }

  private enum TrackStyle {
    case row
    case equalRow
    case stacked
  }

  private var track: some View {
    HStack(spacing: IntradaSpacing.controlGap) {
      ForEach(options, id: \.self) { option in
        pillButton(option, style: .row)
      }
    }
  }

  private func fullWidthTrack(_ style: TrackStyle) -> some View {
    let stacked = style == .stacked
    let shape = RoundedRectangle(cornerRadius: stacked ? IntradaRadius.card : IntradaRadius.pill)
    let trackLayout =
      stacked ? AnyLayout(VStackLayout(spacing: 4)) : AnyLayout(HStackLayout(spacing: 4))
    return trackLayout {
      ForEach(options, id: \.self) { option in
        pillButton(option, style: style)
      }
    }
    .padding(4)
    .background(IntradaColor.cardFill, in: shape)
    .overlay(shape.stroke(IntradaColor.hairline, lineWidth: 1))
  }

  private func pillButton(_ option: Option, style: TrackStyle) -> some View {
    let isSelected = option == selection
    return Button {
      withAnimation(reduceMotion ? nil : IntradaMotion.snappy) {
        selection = option
      }
    } label: {
      pillLabel(option, isSelected: isSelected, style: style)
        .background {
          if isSelected {
            Capsule()
              .fill(IntradaColor.accent)
              .matchedGeometryEffect(id: "selectedPill", in: pill)
          }
        }
    }
    .buttonStyle(.plain)
    .accessibilityLabel(label(option))
    .accessibilityHint(hint?(option) ?? "")
    .accessibilityIdentifier(identifier?(option) ?? "")
    .accessibilityAddTraits(isSelected ? [.isSelected] : [])
  }

  @ViewBuilder private func pillLabel(_ option: Option, isSelected: Bool, style: TrackStyle)
    -> some View
  {
    let text = Text(label(option))
      .font(font)
      .foregroundStyle(isSelected ? IntradaColor.onAccent : unselectedColor)
    switch layout {
    case .inlineScrolling:
      text.lineLimit(1).padding(.vertical, 6).padding(.horizontal, IntradaSpacing.card)
    case .fullWidthTrack where style == .equalRow:
      // Sized to the longest label so the row wins only when every segment fits (#2133).
      ZStack {
        ForEach(options, id: \.self) { Text(label($0)).font(font).hidden() }
        text
      }
      .frame(maxWidth: .infinity).padding(.vertical, IntradaSpacing.controlGap)
    case .fullWidthTrack:
      text.frame(maxWidth: .infinity).padding(.vertical, IntradaSpacing.controlGap)
    }
  }
}

#if DEBUG
  private struct SegmentedPillsPreview: View {
    @State private var tab: LibraryFilter = .pieces
    @State private var kind: ItemKind = .piece
    var body: some View {
      ZStack {
        PaperBackground()
        VStack(spacing: IntradaSpacing.card) {
          SegmentedPills(
            options: LibraryFilter.allCases, selection: $tab, label: \.label,
            layout: .inlineScrolling(edgeInset: 0))
          SegmentedPills(
            options: [.piece, .exercise], selection: $kind, label: \.label,
            font: IntradaFont.segment,
            layout: .fullWidthTrack)
        }
        .padding(IntradaSpacing.card)
      }
    }
  }

  #Preview { SegmentedPillsPreview() }
#endif
