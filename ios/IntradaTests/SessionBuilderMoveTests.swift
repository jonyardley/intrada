import SharedTypes
import Testing

@testable import Intrada

/// Which move a drop in the session builder's list asks the core for (#1957).
/// Rows: 0 Clair de Lune alone, 1 the block's header, 2 Scales, 3 Broken
/// arpeggios, 4 "Add a related exercise", 5 Sight-reading alone.
@MainActor
struct SessionBuilderMoveTests {
  typealias Row = SessionBuilderScreen.BuilderRow

  private var rows: [Row] {
    let block: [SetlistEntryView] = [
      .previewGroupedScales, .previewGroupedArpeggios, .previewGroupedPiece,
    ]
    let blocks = [
      SetlistBlockView(
        groupId: nil, pieceTitle: nil, relatedCount: 0, durationDisplay: "",
        entries: [.previewPiece], takenElsewhere: []),
      SetlistBlockView(
        groupId: "g1", pieceTitle: "Clair de Lune", relatedCount: 2, durationDisplay: "12 min",
        entries: block, takenElsewhere: ["ex-c"]),
      SetlistBlockView(
        groupId: nil, pieceTitle: nil, relatedCount: 0, durationDisplay: "",
        entries: [.previewStandaloneExercise], takenElsewhere: []),
    ]
    return Row.rows(for: blocks, collapsed: [], isEditing: false)
  }

  @Test("the rows the table below is written against")
  func theRowsAreTheOnesTheTableAssumes() {
    #expect(rows.map(\.id) == ["setlist-1", "header-g1", "g-a", "g-b", "add-g1", "g-s"])
  }

  @Test(
    "a drop becomes the move the core is sent",
    arguments: [
      (5, 0, SessionBuilderScreen.BuilderMove.unit(entryId: "g-s", to: 0)),
      (5, 3, .unit(entryId: "g-s", to: 1)),
      (1, 6, .unit(entryId: "g-a", to: 2)),
      (0, 2, .unit(entryId: "setlist-1", to: 1)),
      (3, 2, .related(entryId: "g-b", to: 0)),
      (2, 4, .related(entryId: "g-a", to: 1)),
    ])
  func aDropBecomesAMove(from: Int, destination: Int, expected: SessionBuilderScreen.BuilderMove) {
    #expect(Row.move(in: rows, from: from, to: destination) == expected)
  }

  @Test(
    "a drop that changes nothing sends nothing",
    arguments: [
      (1, 3, "a header dropped among its own exercises stays put"),
      (1, 1, "a header dropped at its own place stays put"),
      (2, 0, "a related exercise dropped outside its block snaps home"),
      (2, 2, "a related exercise dropped where it was"),
      (0, 0, "a standalone dropped where it was"),
      (4, 0, "the add row does not move"),
      (9, 0, "a row that is not there"),
    ])
  func aDropThatChangesNothing(from: Int, destination: Int, why: String) {
    #expect(Row.move(in: rows, from: from, to: destination) == nil, "\(why)")
  }

  @Test("each move names the event it sends")
  func eachMoveNamesItsEvent() {
    #expect(
      SessionBuilderScreen.BuilderMove.unit(entryId: "g-s", to: 1).event
        == .session(.moveUnit(entryId: "g-s", newPosition: 1)))
    #expect(
      SessionBuilderScreen.BuilderMove.related(entryId: "g-b", to: 0).event
        == .session(.moveRelated(entryId: "g-b", newPosition: 0)))
  }
}
