import SharedTypes
import Testing

@testable import Intrada

/// What a link is for, and the link fixtures held to what the core projects
/// from the same links (#2232).
@MainActor
struct ExerciseLinkSetsTests {
  @Test(arguments: [
    (true, [String](), nil as String?),
    (false, ["A2"], "For A2"),
    (false, ["bars 19 to 20"], "For bars 19 to 20"),
    (false, ["bar 12"], "For bar 12"),
    (true, ["A2"], "For the whole piece and A2"),
    (false, ["A1", "B", "Coda"], "For A1, B and Coda"),
  ])
  func theCaptionNamesWhatTheLinkIsFor(wholePiece: Bool, labels: [String], expected: String?) {
    let sections = labels.map { LinkedSectionView(id: $0, label: $0, labelInText: $0) }
    #expect(sectionLinkCaption(wholePiece: wholePiece, sections: sections) == expected)
  }

  /// The Used in fixture holds to what the core projects from the same links.
  @Test func theUsedInSectionsFixtureMatchesTheCore() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    let fixture = LibraryItemView.previewExerciseUsedInSections
    let exercise = try add(bridge, fixture.title, .exercise)
    var targets: [LinkTarget] = []
    for row in fixture.usedIn {
      let piece = try #require(row.piece)
      let pieceId = try add(bridge, piece.title, .piece)
      try bridge.addSections(named: row.sections.map(\.label), to: pieceId)
      let ids = try #require(try bridge.rendered().items.first { $0.id == pieceId }?.sections)
        .map(\.id)
      if row.wholePiece { targets.append(LinkTarget(pieceId: pieceId, sectionId: nil)) }
      targets += ids.map { LinkTarget(pieceId: pieceId, sectionId: $0) }
    }

    _ = try bridge.update(.item(.setExerciseLinks(exerciseId: exercise, targets: targets)))

    let usedIn = try #require(try bridge.rendered().items.first { $0.id == exercise }?.usedIn)
    let byTitle = { (rows: [ExerciseUsageView]) in
      rows.sorted { ($0.piece?.title ?? "") < ($1.piece?.title ?? "") }.map(\.sectionsCaption)
    }
    #expect(byTitle(usedIn) == byTitle(fixture.usedIn))
    let captions = { (rows: [ExerciseUsageView]) in
      rows.sorted { ($0.piece?.title ?? "") < ($1.piece?.title ?? "") }.map(\.linkCaption)
    }
    #expect(captions(usedIn) == captions(fixture.usedIn))
  }

  private func add(_ bridge: RowsBridge, _ title: String, _ kind: ItemKind) throws -> String {
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: title, kind: kind, composer: nil, key: nil, tempo: nil, notes: nil,
            tags: [], photoId: nil, variationLabels: []))))
    return try #require(try bridge.rendered().items.first { $0.title == title }?.id)
  }

  /// The snapshot fixture holds to what the core projects from the same links
  /// (#1949).
  @Test func theSectionLinksFixtureMatchesTheCore() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    let fixture = LibraryItemView.previewPieceWithSectionLinks
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: fixture.title, kind: .piece, composer: nil, key: nil, tempo: nil,
            notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let pieceId = try #require(try bridge.rendered().items.first?.id)
    try bridge.addSections(
      fixture.sections.map { section in
        let bars = section.firstBar.flatMap { first in
          section.lastBar.map { BarsInput.picked(first: first, last: $0) }
        }
        return SectionEdit(
          id: nil, name: section.name, bars: bars ?? .blank, kind: section.kind,
          targetBpm: section.targetBpm.map(String.init) ?? "")
      }, to: pieceId)
    let sections = try #require(try bridge.rendered().items.first { $0.id == pieceId }?.sections)
    let idByLabel = Dictionary(uniqueKeysWithValues: sections.map { ($0.label, $0.id) })
    var changes: [LinkChange] = []
    for exercise in fixture.linkedExercises {
      _ = try bridge.update(
        .item(
          .add(
            CreateItem(
              title: exercise.title, kind: .exercise, composer: nil, key: nil, tempo: nil,
              notes: nil, tags: [], photoId: nil, variationLabels: []))))
      let id = try #require(try bridge.rendered().items.first { $0.title == exercise.title }?.id)
      changes.append(
        .set(
          exerciseId: id, wholePiece: exercise.wholePiece,
          sectionIds: exercise.sections.compactMap { idByLabel[$0.label] }))
    }

    for change in changes {
      _ = try bridge.update(.item(.changePieceLink(pieceId: pieceId, change: change)))
    }

    let card = try #require(try bridge.rendered().items.first { $0.id == pieceId }?.linkedExercises)
    #expect(card.map(\.sectionsCaption) == fixture.linkedExercises.map(\.sectionsCaption))
    #expect(card.map(\.linkCaption) == fixture.linkedExercises.map(\.linkCaption))
    #expect(card.map(\.wholePiece) == fixture.linkedExercises.map(\.wholePiece))
  }
}
