import SharedTypes

extension PlayView {
  /// What the musician called this stretch. A plain run-through is honest
  /// rather than blank: a piece, and an exercise with no variations, always
  /// records one (#1739 decision 3).
  var displayLabel: String { label ?? "No variation" }

  /// Facts only, and only the ones this play actually recorded: a tempo
  /// appears when it was measured, repetitions when there was a target.
  var metaParts: [String] {
    var parts = [durationDisplay]
    if let achievedTempo { parts.append("\(achievedTempo) bpm") }
    if let repTarget { parts.append("\(repCount ?? 0) of \(repTarget) reps") }
    return parts
  }
}
