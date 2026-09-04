/// Not `Tempo`: the generated `SharedTypes.Tempo` already takes that name in the app module.
enum TempoScale {
  /// Narrower than the core's validation; a target outside it arrives clamped
  /// rather than showing a tempo the steppers cannot reach.
  static let range = 40...208
  static let step = 2
  /// Neutral tempo for an item declaring none.
  static let defaultBpm = 96

  /// The same band expressed in the unit the click counts in: a 6/8 piece at
  /// dotted crotchet = 80 is quaver = 240, which the crotchet ceiling of 208
  /// would refuse (#1499).
  static func range(unit: UInt8) -> ClosedRange<Int> {
    let scale = Int(unit)
    return (range.lowerBound * scale / 4)...(range.upperBound * scale / 4)
  }

  static func clamp(_ value: Int, unit: UInt8 = 4) -> Int {
    let band = range(unit: unit)
    return min(band.upperBound, max(band.lowerBound, value))
  }

  static func stepped(from value: Int, by delta: Int, unit: UInt8 = 4) -> Int {
    clamp(value + delta, unit: unit)
  }
}
