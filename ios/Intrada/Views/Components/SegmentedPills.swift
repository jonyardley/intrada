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
    /// Equal segments while the widest label fits, then label-sized pills on one
    /// row, then whole pills over balanced rows rather than a broken label (#2490).
    case wrappingTrack
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
  @State private var pillHeight: CGFloat = 0

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
    case .wrappingTrack:
      if typeSize.isAccessibilitySize {
        fullWidthTrack(.stacked)
      } else {
        ViewThatFits(in: .horizontal) {
          fullWidthTrack(.equalRow)
          fullWidthTrack(.labelSized)
          ForEach(wrappedRowLengths, id: \.self) { wrappedTrack(perRow: $0) }
          fullWidthTrack(.stacked)
        }
      }
    }
  }

  private enum TrackStyle {
    case row
    case equalRow
    case labelSized
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
    // Circular: a continuous capsule or a 999pt corner strokes a flat sliver at each end (#2490).
    let shape =
      stacked
      ? AnyShape(RoundedRectangle(cornerRadius: IntradaRadius.card))
      : AnyShape(Capsule(style: .circular))
    let trackLayout =
      switch style {
      case .stacked: AnyLayout(VStackLayout(spacing: 4))
      case .labelSized: AnyLayout(SpareSharedRow(spacing: 4))
      default: AnyLayout(HStackLayout(spacing: 4))
      }
    return trackLayout {
      ForEach(options, id: \.self) { option in
        pillButton(option, style: style)
      }
    }
    .padding(4)
    .background(IntradaColor.cardFill, in: shape)
    .overlay(shape.stroke(IntradaColor.hairline, lineWidth: 1))
  }

  private var wrappedRowLengths: [Int] {
    let lengths = (2..<max(options.count, 2)).map { (options.count + $0 - 1) / $0 }
    return lengths.enumerated().filter { $0.offset == 0 || lengths[$0.offset - 1] != $0.element }
      .map(\.element)
  }

  private func wrappedTrack(perRow: Int) -> some View {
    let chunks = stride(from: 0, to: options.count, by: perRow).map {
      Array(options[$0..<min($0 + perRow, options.count)])
    }
    let shape = RoundedRectangle(cornerRadius: pillHeight / 2 + 4, style: .circular)
    return VStack(spacing: 4) {
      ForEach(chunks, id: \.self) { row in
        HStack(spacing: 4) {
          ForEach(row, id: \.self) { option in
            pillButton(option, style: .equalRow, sizedTo: row)
          }
        }
      }
    }
    .onGeometryChange(for: CGFloat.self) {
      ($0.size.height - 4 * CGFloat(chunks.count - 1)) / CGFloat(chunks.count)
    } action: {
      pillHeight = $0
    }
    .padding(4)
    .background(IntradaColor.cardFill, in: shape)
    .overlay(shape.stroke(IntradaColor.hairline, lineWidth: 1))
  }

  private func pillButton(_ option: Option, style: TrackStyle, sizedTo row: [Option]? = nil)
    -> some View
  {
    let isSelected = option == selection
    return Button {
      withAnimation(reduceMotion ? nil : IntradaMotion.snappy) {
        selection = option
      }
    } label: {
      pillLabel(option, isSelected: isSelected, style: style, sizedTo: row ?? options)
        .contentShape(Capsule(style: .circular))
        .background {
          if isSelected {
            Capsule(style: .circular)
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

  @ViewBuilder private func pillLabel(
    _ option: Option, isSelected: Bool, style: TrackStyle, sizedTo row: [Option]
  ) -> some View {
    let text = Text(label(option))
      .font(font)
      .foregroundStyle(isSelected ? IntradaColor.onAccent : unselectedColor)
    switch layout {
    case .inlineScrolling:
      text.lineLimit(1).padding(.vertical, 6).padding(.horizontal, IntradaSpacing.card)
    case .wrappingTrack where style == .stacked:
      text.frame(maxWidth: .infinity).padding(.vertical, IntradaSpacing.controlGap)
    case .wrappingTrack where style == .labelSized:
      text.lineLimit(1).padding(.horizontal, 4)
        .frame(maxWidth: .infinity).padding(.vertical, IntradaSpacing.controlGap)
    case .wrappingTrack:
      // The row's widest label plus a margin, so a row that only just fits wraps instead (#2490).
      ZStack {
        ForEach(row, id: \.self) { Text(label($0)).font(font).hidden() }
        text
      }
      .padding(.horizontal, 4)
      .frame(maxWidth: .infinity).padding(.vertical, IntradaSpacing.controlGap)
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

/// One row where each pill takes its label's width plus an even share of what is left.
private struct SpareSharedRow: Layout {
  var spacing: CGFloat

  func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout Void) -> CGSize {
    let sizes = subviews.map { $0.sizeThatFits(.unspecified) }
    let natural = sizes.map(\.width).reduce(0, +) + spacing * CGFloat(max(subviews.count - 1, 0))
    return CGSize(
      width: max(proposal.width ?? natural, natural), height: sizes.map(\.height).max() ?? 0)
  }

  func placeSubviews(
    in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout Void
  ) {
    let sizes = subviews.map { $0.sizeThatFits(.unspecified) }
    let natural = sizes.map(\.width).reduce(0, +) + spacing * CGFloat(max(subviews.count - 1, 0))
    let share = subviews.isEmpty ? 0 : max(bounds.width - natural, 0) / CGFloat(subviews.count)
    var x = bounds.minX
    for (subview, size) in zip(subviews, sizes) {
      let width = size.width + share
      subview.place(
        at: CGPoint(x: x, y: bounds.minY), anchor: .topLeading,
        proposal: ProposedViewSize(width: width, height: bounds.height))
      x += width + spacing
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
