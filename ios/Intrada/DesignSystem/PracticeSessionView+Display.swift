import SharedTypes

extension PracticeSessionView {
  /// "3 pieces", "2 exercises", or "5 items" when the session spans both.
  var itemCountDisplay: String {
    let count = entries.count
    let noun: String
    if entries.allSatisfy({ $0.itemType == .piece }) {
      noun = count == 1 ? "piece" : "pieces"
    } else if entries.allSatisfy({ $0.itemType == .exercise }) {
      noun = count == 1 ? "exercise" : "exercises"
    } else {
      noun = count == 1 ? "item" : "items"
    }
    return "\(count) \(noun)"
  }
}
