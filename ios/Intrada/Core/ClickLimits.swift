import SharedTypes

/// The click's band and bars as the core sends them (#2225): lookups only, the
/// rules behind them live in `domain/metre.rs`.
extension LimitsView {
  var clickStep: Int { Int(clickTempoStep) }

  /// Counted in the bar's own unit: a 6/8 piece at quaver = 240 is inside the
  /// quaver band, though above the crotchet ceiling (#1499).
  func clickBand(unit: UInt8) -> ClosedRange<Int> {
    guard let band = clickTempoBands.first(where: { $0.unit == unit }) else {
      return Int(clickTempoDefault)...Int(clickTempoDefault)
    }
    return Int(band.min)...Int(band.max)
  }

  func clampClickTempo(_ value: Int, unit: UInt8) -> Int {
    let band = clickBand(unit: unit)
    return min(band.upperBound, max(band.lowerBound, value))
  }

  func clickPresets(for metre: Metre) -> [ClickPresetOption] {
    clickBars.first { $0.beats == metre.beats && $0.groups == metre.groups }?.presets ?? []
  }

  func clickGroupings(beats: Int) -> [[UInt8]] {
    clickBars.compactMap { $0.beats == beats ? $0.groups : nil }
  }
}

extension TempoBand {
  var range: ClosedRange<Int> { Int(min)...Int(max) }
}

extension ClickPreset {
  var title: String {
    switch self {
    case .everyBeat: "Every beat"
    case .groupStarts: "Group starts"
    case .downbeat: "Downbeat"
    case .backbeat: "2 and 4"
    }
  }
}
