import SharedTypes

// ── The whole chosen set, read back off the view; the core diffs it (#2232) ──

extension LinkedExerciseView {
  var linkEdits: [LinkEdit] {
    let whole = wholePiece ? [LinkEdit(exercise: .existing(id: id), sectionId: nil)] : []
    return whole + sections.map { LinkEdit(exercise: .existing(id: id), sectionId: $0.id) }
  }
}

extension LibraryItemView {
  /// The piece's set in card order, with `exerciseId`'s links replaced by
  /// `wholePiece` and `sectionIds`, in place.
  func pieceLinks(
    setting exerciseId: String, wholePiece: Bool, sectionIds: [String]
  ) -> [LinkEdit] {
    linkedExercises.flatMap { exercise -> [LinkEdit] in
      guard exercise.id == exerciseId else { return exercise.linkEdits }
      let whole = wholePiece ? [LinkEdit(exercise: .existing(id: exerciseId), sectionId: nil)] : []
      return whole + sectionIds.map { LinkEdit(exercise: .existing(id: exerciseId), sectionId: $0) }
    }
  }

  /// The piece's set with the exercises in `order`; any left out are dropped.
  func pieceLinks(order: [String]) -> [LinkEdit] {
    let byId = Dictionary(uniqueKeysWithValues: linkedExercises.map { ($0.id, $0) })
    return order.compactMap { byId[$0] }.flatMap(\.linkEdits)
  }

  /// The exercise's set across pieces: each piece in `pieceIds` keeps the links
  /// it has, and a piece new to the set links as a whole.
  func exerciseTargets(pieceIds: [String]) -> [LinkTarget] {
    let linked = Dictionary(
      usedIn.filter(\.linked).compactMap { row in row.piece.map { ($0.id, row) } },
      uniquingKeysWith: { first, _ in first })
    return pieceIds.flatMap { pieceId -> [LinkTarget] in
      guard let row = linked[pieceId] else { return [LinkTarget(pieceId: pieceId, sectionId: nil)] }
      let whole = row.wholePiece ? [LinkTarget(pieceId: pieceId, sectionId: nil)] : []
      return whole + row.sections.map { LinkTarget(pieceId: pieceId, sectionId: $0.id) }
    }
  }
}
