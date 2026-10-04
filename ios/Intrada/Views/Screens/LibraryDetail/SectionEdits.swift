import SharedTypes

/// The item screen writes its whole section list in one event (#2245), so every
/// change is the list as it stands with one row saved, removed or moved. The
/// core validates and reconciles; this only carries the rows back.
enum SectionEdits {
  static func edit(from section: SectionView) -> SectionEdit {
    SectionEdit(
      id: section.id, name: section.name, bars: bars(of: section), kind: section.kind,
      targetBpm: section.targetBpm.map(String.init) ?? "")
  }

  /// What the sheet's Bars field opens with: "19 to 20", "12", or empty.
  static func barsText(of section: SectionView) -> String {
    guard let first = section.firstBar else { return "" }
    guard let last = section.lastBar, last != first else { return "\(first)" }
    return "\(first) to \(last)"
  }

  static func saving(_ edit: SectionEdit, into sections: [SectionView]) -> [SectionEdit] {
    var edits = sections.map(Self.edit(from:))
    if let id = edit.id, let index = edits.firstIndex(where: { $0.id == id }) {
      edits[index] = edit
    } else {
      edits.append(edit)
    }
    return edits
  }

  static func removing(_ id: String, from sections: [SectionView]) -> [SectionEdit] {
    sections.filter { $0.id != id }.map(Self.edit(from:))
  }

  private static func bars(of section: SectionView) -> BarsInput {
    guard let first = section.firstBar else { return .blank }
    return .picked(first: first, last: section.lastBar ?? first)
  }
}
