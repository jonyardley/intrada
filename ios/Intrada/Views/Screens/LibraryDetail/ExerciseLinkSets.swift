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

  /// The piece's set after its picker: kept exercises keep their sections in
  /// card order, a newly chosen one links whole after them in library order,
  /// then each written exercise.
  func pieceLinks(
    choosing selected: Swift.Set<String>, in library: [String], written: [ScaffoldEntry]
  )
    -> [LinkEdit]
  {
    let current = Swift.Set(linkedExercises.map(\.id))
    let kept = linkedExercises.filter { selected.contains($0.id) }.flatMap(\.linkEdits)
    let added = library.filter { selected.contains($0) && !current.contains($0) }
      .map { LinkEdit(exercise: .existing(id: $0), sectionId: nil) }
    return kept + added + written.map { LinkEdit(exercise: $0, sectionId: nil) }
  }

  /// The pieces that declare the link, not those only practised alongside.
  var linkedPieceIds: [String] {
    usedIn.filter(\.linked).compactMap { $0.piece?.id }
  }

  /// The exercise's set after its piece picker: kept pieces first, then new
  /// ones in library order.
  func exerciseTargets(choosing selected: Swift.Set<String>, in library: [String]) -> [LinkTarget] {
    let kept = linkedPieceIds.filter(selected.contains)
    let added = library.filter { selected.contains($0) && !kept.contains($0) }
    return exerciseTargets(pieceIds: kept + added)
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
