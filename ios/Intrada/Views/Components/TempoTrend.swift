import SwiftUI

/// One session in `TempoTrend`. `tempo` is nil where that session measured
/// none, which the plot draws as a break rather than a zero.
struct TempoTrendMark: Identifiable {
  let id: String
  let date: Date
  let tempo: Int?
}

/// Everything `TempoTrend` draws, with the end dates already formatted against
/// the environment's locale and calendar so snapshot hosts stay deterministic
/// (the same reason `recentSessionRows` takes them).
struct TempoTrendDisplay {
  /// Oldest first, matching the core's `tempoTrend.points` ordering.
  let marks: [TempoTrendMark]
  let hasTrend: Bool
  let startDateText: String
  let endDateText: String
}

/// An item's measured tempo over time: a line across the sessions that measured
/// one, breaking wherever a session measured none. Presentation only — which
/// sessions carry a number, and whether there is a trend at all, are the core's
/// rulings (`TempoTrendView`, design-principles T16).
struct TempoTrend: View {
  let display: TempoTrendDisplay

  @Environment(\.dynamicTypeSize) private var dynamicTypeSize

  private var marks: [TempoTrendMark] { display.marks }
  private var hasTrend: Bool { display.hasTrend }

  private let plotHeight: CGFloat = 84
  private let dotRadius: CGFloat = 3.5
  private let latestDotRadius: CGFloat = 5

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      header
      if hasTrend {
        plot
          .frame(height: plotHeight + 6)
          .padding(.top, 2)
        footer
      } else {
        Text(singleMeasurementText)
          .font(IntradaFont.meta)
          .foregroundStyle(IntradaColor.inkSecondary)
      }
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.horizontal, IntradaSpacing.card)
    .padding(.vertical, IntradaSpacing.cardCompact)
    .cardSurface(cornerRadius: IntradaRadius.panel)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityLabel)
  }

  // ── Header ──

  private var header: some View {
    Group {
      // Side by side normally; stacked at accessibility sizes, where sharing
      // the row leaves the eyebrow too narrow for its own first word.
      if dynamicTypeSize.isAccessibilitySize {
        VStack(alignment: .leading, spacing: 2) {
          eyebrow
          chip
        }
      } else {
        HStack(alignment: .firstTextBaseline) {
          eyebrow
          Spacer(minLength: IntradaSpacing.controlGap)
          chip
        }
      }
    }
    .padding(.bottom, IntradaSpacing.controlGap)
  }

  private var eyebrow: some View {
    Text("Measured tempo")
      .font(IntradaFont.eyebrow)
      .textCase(.uppercase)
      .kerning(1.2)
      .foregroundStyle(IntradaColor.inkFaint)
  }

  @ViewBuilder private var chip: some View {
    if hasTrend, let first = measured.first, let latest = measured.last {
      // Neutral, never a success colour: a faster tempo is not a better one,
      // and slowing down deliberately is practice, not a decline.
      Text("♩ = \(first) → \(latest)")
        .font(IntradaFont.badge)
        .foregroundStyle(IntradaColor.inkSecondary)
    }
  }

  // ── Plot ──

  private var plot: some View {
    GeometryReader { geometry in
      let width = geometry.size.width
      ZStack(alignment: .topLeading) {
        Rectangle()
          .fill(IntradaColor.hairline)
          .frame(height: 1)
          .offset(y: plotHeight)

        ForEach(unmeasuredIndices, id: \.self) { index in
          // Below the axis, never on it: a mark inside the plot would read as
          // ♩ = 0, which is the reading this whole feature exists to avoid.
          Rectangle()
            .fill(IntradaColor.inkFainter)
            .frame(width: 1.5, height: 6)
            .offset(x: x(at: index, width: width) - 0.75, y: plotHeight + 1)
        }

        ForEach(Array(runs.enumerated()), id: \.offset) { _, run in
          Path { path in
            for (step, index) in run.enumerated() {
              let point = CGPoint(x: x(at: index, width: width), y: y(at: index))
              if step == 0 {
                path.move(to: point)
              } else {
                path.addLine(to: point)
              }
            }
          }
          .stroke(
            IntradaColor.accent,
            style: StrokeStyle(lineWidth: 2, lineCap: .round, lineJoin: .round))
        }

        ForEach(measuredIndices, id: \.self) { index in
          let isLatest = index == measuredIndices.last
          let radius = isLatest ? latestDotRadius : dotRadius
          Circle()
            .fill(IntradaColor.accent)
            .frame(width: radius * 2, height: radius * 2)
            .overlay(
              Circle()
                .stroke(IntradaColor.cardFill, lineWidth: isLatest ? 2 : 0)
            )
            .offset(
              x: x(at: index, width: width) - radius,
              y: y(at: index) - radius)
        }
      }
    }
  }

  private var footer: some View {
    HStack(alignment: .firstTextBaseline) {
      // At accessibility sizes three labels cannot share the row, and the count
      // is the one that earns its place: it is what stops the breaks in the
      // line reading as a fault.
      if !dynamicTypeSize.isAccessibilitySize {
        Text(display.startDateText)
        Spacer()
      }
      Text(measuredCountText)
      if !dynamicTypeSize.isAccessibilitySize {
        Spacer()
        Text(display.endDateText)
      }
    }
    .font(IntradaFont.micro)
    .foregroundStyle(IntradaColor.inkFaint)
    .padding(.top, 6)
  }

  // ── Geometry ──

  /// Spaced by date, so a fortnight off reads as a fortnight and not as one
  /// step. Sessions sharing a date (or a single one) fall back to even spacing.
  private func x(at index: Int, width: CGFloat) -> CGFloat {
    let inset = latestDotRadius
    let usable = max(1, width - inset * 2)
    guard marks.count > 1 else { return inset + usable / 2 }
    guard let first = marks.first?.date, let last = marks.last?.date else { return inset }
    let span = last.timeIntervalSince(first)
    guard span > 0 else {
      return inset + usable * CGFloat(index) / CGFloat(marks.count - 1)
    }
    let offset = marks[index].date.timeIntervalSince(first)
    return inset + usable * CGFloat(offset / span)
  }

  private func y(at index: Int) -> CGFloat {
    guard let tempo = marks[index].tempo, let low = measured.min(), let high = measured.max()
    else { return plotHeight }
    let span = CGFloat(high - low)
    let top = latestDotRadius
    let bottom = plotHeight - latestDotRadius
    guard span > 0 else { return (top + bottom) / 2 }
    return bottom - (bottom - top) * CGFloat(tempo - low) / span
  }

  // ── Series ──

  private var measured: [Int] { marks.compactMap(\.tempo) }
  private var measuredIndices: [Int] {
    marks.enumerated().filter { $0.element.tempo != nil }.map(\.offset)
  }
  private var unmeasuredIndices: [Int] {
    marks.enumerated().filter { $0.element.tempo == nil }.map(\.offset)
  }

  /// Runs of consecutive measured sessions. A run of one draws its dot and no
  /// line, so nothing is ever joined across a session that measured nothing.
  private var runs: [[Int]] {
    var runs: [[Int]] = []
    var current: [Int] = []
    for (index, mark) in marks.enumerated() {
      if mark.tempo == nil {
        if current.count > 1 { runs.append(current) }
        current = []
      } else {
        current.append(index)
      }
    }
    if current.count > 1 { runs.append(current) }
    return runs
  }

  // ── Copy ──

  private var measuredCountText: String {
    "\(measured.count) of \(marks.count) sessions measured"
  }

  private var singleMeasurementText: String {
    guard let only = measured.first else { return "No tempo measured yet" }
    return "Measured once, at ♩ = \(only) · no trend yet"
  }

  private var accessibilityLabel: String {
    guard hasTrend, let first = measured.first, let latest = measured.last else {
      return "Measured tempo. \(singleMeasurementText)"
    }
    return
      "Measured tempo, \(first) to \(latest) beats per minute across \(measured.count) of \(marks.count) sessions"
  }
}

#if DEBUG
  #Preview("Tempo trend") {
    ZStack {
      PaperBackground()
      VStack(spacing: IntradaSpacing.cardCompact) {
        TempoTrend(display: .previewWithGaps)
        TempoTrend(display: .previewSingleMeasurement)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
