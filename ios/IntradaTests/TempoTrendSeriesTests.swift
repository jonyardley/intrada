import Foundation
import SharedTypes
import Testing

@testable import Intrada

/// `series` holds the whole correctness surface of the tempo trend plot, and a
/// pixel snapshot cannot say which invariant broke.
struct TempoTrendSeriesTests {
  private static let base = Date(timeIntervalSince1970: 1_782_291_600)

  private static func series(_ tempos: [Int?], daysApart: Double = 3) -> TempoTrendSeries {
    TempoTrendSeries(
      marks: tempos.enumerated().map { index, tempo in
        TempoTrendMark(
          date: base.addingTimeInterval(Double(index) * daysApart * 86_400), tempo: tempo)
      })
  }

  private static let shapes: [[Int?]] = [
    [], [nil], [88], [88, nil], [nil, 88],
    [88, nil, 96], [nil, 88, 96, nil], [88, 96, nil, nil, 104],
    [88, 92, 96, nil, 100, 104, nil, 108, 116],
  ]

  @Test("no run of line ever spans a session that measured nothing", arguments: shapes)
  func runsNeverCrossAGap(tempos: [Int?]) {
    let series = Self.series(tempos)
    for run in series.runs {
      #expect(run.count >= 2, "a lone measurement has nothing to join to")
      for index in run {
        #expect(tempos[index] != nil, "a run may only hold measured sessions")
      }
      // Consecutive by index is what makes the gap a gap: two measured points
      // either side of an unmeasured one must never end up in the same run.
      #expect(Array(run.first!...run.last!) == run)
    }
  }

  @Test("every measured session gets a dot, and no unmeasured one does", arguments: shapes)
  func dotsMatchTheMeasurements(tempos: [Int?]) {
    let series = Self.series(tempos)
    #expect(series.measuredIndices.map { tempos[$0] } == tempos.filter { $0 != nil })
    #expect(series.unmeasuredIndices.allSatisfy { tempos[$0] == nil })
    #expect(series.measuredIndices.count + series.unmeasuredIndices.count == tempos.count)
  }

  @Test("every point lands inside the plot", arguments: shapes)
  func geometryStaysInsideThePlot(tempos: [Int?]) {
    let series = Self.series(tempos)
    let (width, height, inset): (CGFloat, CGFloat, CGFloat) = (300, 84, 5)
    for index in series.measuredIndices {
      let x = series.x(at: index, width: width, inset: inset)
      let y = series.y(at: index, height: height, inset: inset)
      #expect(x >= inset && x <= width - inset)
      #expect(y >= inset && y <= height - inset)
    }
  }

  @Test("x runs left to right, oldest first")
  func xIsMonotonic() {
    let series = Self.series([88, 92, 96, nil, 104])
    let xs = (0..<5).map { series.x(at: $0, width: 300, inset: 5) }
    #expect(xs == xs.sorted())
    #expect(xs.first == 5)
    #expect(xs.last == 295)
  }

  @Test("sessions on one date fall back to even spacing rather than stacking")
  func sameDateSpacesEvenly() {
    let series = Self.series([88, 96, 104], daysApart: 0)
    let xs = (0..<3).map { series.x(at: $0, width: 300, inset: 5) }
    #expect(xs == [5, 150, 295])
  }

  @Test("a single session sits in the middle rather than on the edge")
  func singleMarkIsCentred() {
    #expect(Self.series([88]).x(at: 0, width: 300, inset: 5) == 150)
  }

  @Test("one tempo repeated draws flat, not at the floor or the ceiling")
  func equalTemposDrawFlat() {
    let series = Self.series([96, 96, 96])
    let ys = (0..<3).map { series.y(at: $0, height: 84, inset: 5) }
    #expect(ys.allSatisfy { $0 == ys[0] })
    #expect(ys[0] == 42)
  }

  /// A two-beat drift and a thirty-beat climb must not draw the same shape.
  @Test("a change smaller than the minimum span does not fill the plot")
  func smallChangesStaySmall() {
    let narrow = Self.series([96, 98])
    let wide = Self.series([80, 120])
    let narrowRise =
      narrow.y(at: 0, height: 84, inset: 5) - narrow.y(at: 1, height: 84, inset: 5)
    let wideRise = wide.y(at: 0, height: 84, inset: 5) - wide.y(at: 1, height: 84, inset: 5)
    #expect(narrowRise < wideRise / 3)
    #expect(narrowRise > 0)
  }

  // ── The mapping from the core's view ──

  private func summary(_ points: [(date: String, tempo: UInt16?)], hasTrend: Bool)
    -> ItemPracticeSummary
  {
    ItemPracticeSummary.fixture(
      tempoTrend: TempoTrendView(
        points: points.enumerated().map { index, point in
          TempoTrendPoint(sessionDate: point.date, sessionId: "s\(index)", tempo: point.tempo)
        },
        hasTrend: hasTrend))
  }

  @Test("nothing measured draws no card at all")
  func nothingMeasuredYieldsNoDisplay() {
    let summary = summary(
      [("2026-06-21T09:00:00Z", nil), ("2026-06-24T09:00:00Z", nil)], hasTrend: false)
    #expect(
      summary.tempoTrendDisplay(locale: .init(identifier: "en_US"), calendar: .current) == nil)
  }

  @Test("a date the shell cannot place takes the whole card, not just its point")
  func unplaceableDateYieldsNoDisplay() {
    let summary = summary(
      [("not a date", 96), ("2026-06-24T09:00:00Z", 104)], hasTrend: true)
    #expect(
      summary.tempoTrendDisplay(locale: .init(identifier: "en_US"), calendar: .current) == nil)
  }

  @Test("the display carries the core's ruling rather than recomputing it")
  func displayTakesHasTrendFromTheCore() {
    let summary = summary(
      [("2026-06-21T09:00:00Z", 96), ("2026-06-24T09:00:00Z", 104)], hasTrend: false)
    let display = summary.tempoTrendDisplay(
      locale: .init(identifier: "en_US"), calendar: PreviewCalendar.utc)
    #expect(display?.hasTrend == false)
    #expect(display?.series.measured == [96, 104])
  }
}
