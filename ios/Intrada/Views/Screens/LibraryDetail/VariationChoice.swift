import Foundation
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

  /// The library rows whose label holds the typed text, ignoring case.
  static func matches(_ query: String, in library: [VariationOptionView]) -> [VariationOptionView] {
    let text = trimmed(query)
    guard !text.isEmpty else { return [] }
    return library.filter { $0.label.localizedCaseInsensitiveContains(text) }
  }

  /// The typed text as a new variation, unless the library or this sheet
  /// already holds that label.
  func addable(_ query: String, in library: [VariationOptionView]) -> String? {
    let text = Self.trimmed(query)
    guard !text.isEmpty else { return nil }
    let taken = library.map(\.label) + newLabels
    guard !taken.contains(where: { $0.caseInsensitiveCompare(text) == .orderedSame }) else {
      return nil
    }
    return text
  }

  private static func trimmed(_ text: String) -> String {
    text.trimmingCharacters(in: .whitespacesAndNewlines)
  }
}
