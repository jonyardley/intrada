import SharedTypes

extension PlayView {
  /// What the musician called this stretch. A plain run-through is honest
  /// rather than blank: a piece, and an exercise with no variations, always
  /// records one (#1739 decision 3).
  var displayLabel: String { label ?? "No variation" }
}
