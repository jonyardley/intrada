import SharedTypes

extension LibraryItemView {
  /// The pieces that declare the link, not those only practised alongside.
  var linkedPieceIds: [String] {
    usedIn.filter(\.linked).compactMap { $0.piece?.id }
  }
}
