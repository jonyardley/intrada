/// Not `Tempo`: the generated `SharedTypes.Tempo` already takes that name in the app module.
enum TempoScale {
  /// Narrower than the core's validation; a target outside it arrives clamped
  /// rather than showing a tempo the steppers cannot reach.
  static let range = 40...208
  static let step = 2
  /// Neutral tempo for an item declaring none.
  static let defaultBpm = 96

  static func clamp(_ value: Int) -> Int {
    min(range.upperBound, max(range.lowerBound, value))
  }

  static func stepped(from value: Int, by delta: Int) -> Int {
    clamp(value + delta)
  }
}
