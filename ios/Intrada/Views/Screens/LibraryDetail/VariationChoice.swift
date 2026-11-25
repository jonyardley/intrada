import Foundation
import IntradaCoreFFI
import SharedTypes

/// What the variations sheet has ticked before Done (#2247): library rows by
/// id, in the order ticked, and labels typed here that the core will reuse or
/// add to the library.
struct VariationChoice: Equatable {
  private(set) var ids: [String]
  private(set) var newLabels: [String] = []

  init(ids: [String]) {
    self.ids = ids
  }

  func isChosen(_ id: String) -> Bool {
    ids.contains(id)
  }

  mutating func toggle(_ id: String) {
    if let index = ids.firstIndex(of: id) {
      ids.remove(at: index)
    } else {
      ids.append(id)
    }
  }

  mutating func toggleNew(_ label: String) {
    if let index = newLabels.firstIndex(of: label) {
      newLabels.remove(at: index)
    } else {
      newLabels.append(label)
    }
  }

  /// The core matches and dedupes labels (#2754).
  static func matches(_ query: String, in library: [VariationOptionView]) -> [VariationOptionView] {
    let ids = variationMatches(
      query: query, options: library.map { VariationLabelArg(id: $0.id, label: $0.label) })
    let byId = Dictionary(library.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
    return ids.compactMap { byId[$0] }
  }

  /// The typed text as a new variation, unless the library or this sheet
  /// already holds that label.
  func addable(_ query: String, in library: [VariationOptionView]) -> String? {
    variationAddable(query: query, taken: library.map(\.label) + newLabels)
  }
}
