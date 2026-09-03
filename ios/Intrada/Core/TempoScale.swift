/// The one home for the tempo dial the shell offers (#1402): the click, the
/// reflection sheet's stepper and their tests all read these. Named to stay clear
/// of the generated `SharedTypes.Tempo`.
enum TempoScale {
  /// Narrower than the core's 1 to 400 BPM validation; a target outside it
  /// arrives clamped rather than showing a tempo the steppers cannot reach.
  static let range = 40...208
  static let step = 2
  /// Neutral tempo for an item declaring none.
  static let defaultBpm = 96

  /// Moves an out-of-range value toward the range, never past it further.
  static func clamp(_ value: Int) -> Int {
    min(range.upperBound, max(range.lowerBound, value))
  }

  static func stepped(from value: Int, by delta: Int) -> Int {
    clamp(value + delta)
  }
}
