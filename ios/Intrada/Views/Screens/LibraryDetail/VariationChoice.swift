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

  static func matches(_ query: String, in library: [VariationOptionView]) -> [VariationOptionView] {
    variationMatches(query: query, labels: library.map(\.label)).compactMap { index in
      library.indices.contains(Int(index)) ? library[Int(index)] : nil
    }
  }

  /// The typed text as a new variation, unless the library or this sheet
  /// already holds that label.
  func addable(_ query: String, in library: [VariationOptionView]) -> String? {
    variationAddable(query: query, taken: library.map(\.label) + newLabels)
  }
}
