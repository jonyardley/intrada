import SwiftUI

/// One practised session's slot in a `TempoTrendSeries`. `tempo` is nil where
/// that session measured none.
struct TempoTrendMark {
  let date: Date
  let tempo: Int?
}

/// The plot's arithmetic, apart from the drawing so it can be tested directly:
/// which sessions carry a dot, which stretches of line may be joined, and where
/// each point lands. Oldest first, matching the core's `tempoTrend.points`.
struct TempoTrendSeries {
  let marks: [TempoTrendMark]

  /// A two-beat drift would otherwise fill the same height as a thirty-beat
  /// climb, which is a shape claiming more than the numbers hold.
  static let minimumSpan = 8

  var measured: [Int] { marks.compactMap(\.tempo) }

  var measuredIndices: [Int] {
    marks.enumerated().filter { $0.element.tempo != nil }.map(\.offset)
  }

  var unmeasuredIndices: [Int] {
    marks.enumerated().filter { $0.element.tempo == nil }.map(\.offset)
  }

  /// Stretches of consecutive measured sessions, each two or more long: a lone
  /// measurement has nothing to join to, and no run ever spans a session that
  /// measured nothing.
  var runs: [[Int]] {
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

  /// Spaced by date, so a fortnight off reads as a fortnight and not as one
  /// step. Marks sharing a date (or a single mark) fall back to even spacing.
  func x(at index: Int, width: CGFloat, inset: CGFloat) -> CGFloat {
    let usable = max(1, width - inset * 2)
    guard marks.count > 1 else { return inset + usable / 2 }
    let dates = marks.map(\.date)
    guard let first = dates.min(), let last = dates.max() else { return inset }
    let span = last.timeIntervalSince(first)
    guard span > 0 else {
      return inset + usable * CGFloat(index) / CGFloat(marks.count - 1)
    }
    return inset + usable * CGFloat(marks[index].date.timeIntervalSince(first) / span)
  }

  func y(at index: Int, height: CGFloat, inset: CGFloat) -> CGFloat {
    guard let tempo = marks[index].tempo, let low = measured.min(), let high = measured.max()
    else { return height - inset }
    let top = inset
    let bottom = height - inset
    let span = Double(max(Self.minimumSpan, high - low))
    let floor = Double(low + high) / 2 - span / 2
    return bottom - (bottom - top) * CGFloat((Double(tempo) - floor) / span)
  }
}

/// Everything `TempoTrend` draws, with the end dates already formatted against
/// the environment's locale and calendar so snapshot hosts stay deterministic
/// (the same reason `recentSessionRows` takes them).
struct TempoTrendDisplay {
  let series: TempoTrendSeries
  let hasTrend: Bool
  let startDateText: String
  let endDateText: String
}

/// An item's measured tempo over time. Presentation only: which sessions carry
/// a number, and whether there is a trend at all, are the core's rulings
/// (`TempoTrendView`, design-principles T17).
struct TempoTrend: View {
  let display: TempoTrendDisplay

  @Environment(\.dynamicTypeSize) private var dynamicTypeSize

  private let plotHeight: CGFloat = 84
  private let dotRadius: CGFloat = 3.5
  private let latestDotRadius: CGFloat = 5
  private let tickHeight: CGFloat = 6

  private var series: TempoTrendSeries { display.series }
  private var measured: [Int] { series.measured }

  var body: some View {
    VStack(alignment: .leading, spacing: 0) {
      header
      if display.hasTrend {
        plot
          .frame(height: plotHeight + tickHeight)
        footer
      } else if let only = measured.first {
        Text("Measured once, at ♩ = \(only) · no trend yet")
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
          Eyebrow("Measured tempo")
          chip
        }
      } else {
        HStack(alignment: .firstTextBaseline) {
          Eyebrow("Measured tempo")
          Spacer(minLength: IntradaSpacing.controlGap)
          chip
        }
      }
    }
    .padding(.bottom, IntradaSpacing.controlGap)
  }

  @ViewBuilder private var chip: some View {
    if display.hasTrend, let first = measured.first, let latest = measured.last {
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

        ForEach(series.unmeasuredIndices, id: \.self) { index in
          // Below the axis, never on it: a mark inside the plot would read as
          // ♩ = 0, which is the reading this whole card exists to avoid.
          Rectangle()
            .fill(IntradaColor.inkFainter)
            .frame(width: 1.5, height: tickHeight)
            .offset(x: x(at: index, width: width) - 0.75, y: plotHeight + 1)
        }

        ForEach(Array(series.runs.enumerated()), id: \.offset) { _, run in
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

        ForEach(series.measuredIndices, id: \.self) { index in
          let isLatest = index == series.measuredIndices.last
          let radius = isLatest ? latestDotRadius : dotRadius
          Circle()
            .fill(IntradaColor.accent)
            .frame(width: radius * 2, height: radius * 2)
            .overlay(
              Circle()
                .stroke(IntradaColor.cardFill, lineWidth: isLatest ? 2 : 0)
            )
            .offset(x: x(at: index, width: width) - radius, y: y(at: index) - radius)
        }
      }
    }
  }

  private var footer: some View {
    HStack(alignment: .firstTextBaseline) {
      // Three labels cannot share this row at accessibility sizes, and the
      // count is the one that earns its place: it is what stops the breaks in
      // the line reading as a fault.
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
    .padding(.top, IntradaSpacing.controlGap)
  }

  private func x(at index: Int, width: CGFloat) -> CGFloat {
    series.x(at: index, width: width, inset: latestDotRadius)
  }

  private func y(at index: Int) -> CGFloat {
    series.y(at: index, height: plotHeight, inset: latestDotRadius)
  }

  // ── Copy ──

  private var measuredCountText: String {
    "\(measured.count) of \(series.marks.count) sessions measured"
  }

  /// Spelled out, because VoiceOver reads neither the ♩ glyph nor the middle
  /// dot (the same reason `TempoFormatting.spoken` exists).
  private var accessibilityLabel: String {
    guard display.hasTrend, let first = measured.first, let latest = measured.last else {
      guard let only = measured.first else { return "Measured tempo" }
      return "Measured tempo, measured once at \(only) beats per minute, no trend yet"
    }
    return
      "Measured tempo, \(first) to \(latest) beats per minute across \(measured.count) of \(series.marks.count) sessions"
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
