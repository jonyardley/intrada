import SharedTypes

enum SectionEdits {
  /// What the sheet's Bars field opens with: "19 to 20", "12", or empty.
  static func barsText(of section: SectionView) -> String {
    guard let first = section.firstBar else { return "" }
    guard let last = section.lastBar, last != first else { return "\(first)" }
    return "\(first) to \(last)"
  }
}
