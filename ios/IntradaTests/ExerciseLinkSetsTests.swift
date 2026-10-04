import SharedTypes
import Testing

@testable import Intrada

/// Each screen sends its whole set of links; these check the set it reads back
/// off the view keeps every row it was not asked to change (#2248, #2232).
@MainActor
struct ExerciseLinkSetsTests {
  private func edit(_ exercise: String, _ section: String?) -> LinkEdit {
    LinkEdit(exercise: .existing(id: exercise), sectionId: section)
  }

  @Test(arguments: [
    (true, [String](), nil as String?),
    (false, ["A2"], "For A2"),
    (true, ["A2"], "For the whole piece and A2"),
    (false, ["A1", "B", "Coda"], "For A1, B and Coda"),
  ])
  func theCaptionNamesWhatTheLinkIsFor(wholePiece: Bool, labels: [String], expected: String?) {
    let sections = labels.map { LinkedSectionView(id: $0, label: $0) }
    #expect(sectionLinkCaption(wholePiece: wholePiece, sections: sections) == expected)
  }

  @Test func choosingSectionsForOneExerciseKeepsTheOthersInPlace() {
    let piece = LibraryItemView.previewPieceWithSectionLinks
    let links = piece.pieceLinks(
      setting: "exercise-thirds", wholePiece: true, sectionIds: ["s1", "s3"])
    #expect(
      links == [
        edit("exercise-octaves", nil), edit("exercise-octaves", "s4"),
        edit("exercise-thirds", nil), edit("exercise-thirds", "s1"),
        edit("exercise-thirds", "s3"),
      ])
  }

  @Test func reorderingKeepsEachExercisesSectionsAndDropsTheOnesLeftOut() {
    let piece = LibraryItemView.previewPieceWithSectionLinks
    #expect(
      piece.pieceLinks(order: ["exercise-thirds", "exercise-octaves"]) == [
        edit("exercise-thirds", "s3"), edit("exercise-octaves", nil),
        edit("exercise-octaves", "s4"),
      ])
    #expect(piece.pieceLinks(order: ["exercise-octaves"]).count == 2)
  }

  @Test func aKeptPieceKeepsItsSectionsAndANewOneLinksWhole() {
    let exercise = LibraryItemView.previewExerciseUsedInSections
    #expect(
      exercise.exerciseTargets(pieceIds: ["piece-1", "piece-9"]) == [
        LinkTarget(pieceId: "piece-1", sectionId: "s4"),
        LinkTarget(pieceId: "piece-9", sectionId: nil),
      ])
    #expect(
      exercise.exerciseTargets(pieceIds: ["piece-2"]) == [
        LinkTarget(pieceId: "piece-2", sectionId: nil),
        LinkTarget(pieceId: "piece-2", sectionId: "s9"),
      ])
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
    _ = try bridge.update(
      .item(
        .updateSections(
          id: pieceId,
          sections: fixture.sections.map { section in
            let bars = section.firstBar.flatMap { first in
              section.lastBar.map { BarsInput.picked(first: first, last: $0) }
            }
            return SectionEdit(
              id: nil, name: section.name, bars: bars ?? .blank, kind: section.kind,
              targetBpm: section.targetBpm.map(String.init) ?? "")
          })))
    let sections = try #require(try bridge.rendered().items.first { $0.id == pieceId }?.sections)
    let idByLabel = Dictionary(uniqueKeysWithValues: sections.map { ($0.label, $0.id) })
    var links: [LinkEdit] = []
    for exercise in fixture.linkedExercises {
      _ = try bridge.update(
        .item(
          .add(
            CreateItem(
              title: exercise.title, kind: .exercise, composer: nil, key: nil, tempo: nil,
              notes: nil, tags: [], photoId: nil, variationLabels: []))))
      let id = try #require(try bridge.rendered().items.first { $0.title == exercise.title }?.id)
      if exercise.wholePiece { links.append(edit(id, nil)) }
      for section in exercise.sections { links.append(edit(id, idByLabel[section.label])) }
    }

    _ = try bridge.update(.item(.setPieceLinks(pieceId: pieceId, links: links)))

    let card = try #require(try bridge.rendered().items.first { $0.id == pieceId }?.linkedExercises)
    #expect(card.map(\.sectionsCaption) == fixture.linkedExercises.map(\.sectionsCaption))
    #expect(card.map(\.wholePiece) == fixture.linkedExercises.map(\.wholePiece))
  }
}
