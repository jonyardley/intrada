import SharedTypes
import Testing

@testable import Intrada

/// What a drop in the session builder's list tells the core (#1957, #2231):
/// the rule for what it means is the core's.
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
        entries: [.previewPiece], piece: nil, related: [], takenElsewhere: []),
      SetlistBlockView(
        groupId: "g1", pieceTitle: "Clair de Lune", relatedCount: 2, durationDisplay: "12 min",
        entries: block, piece: block.last, related: Array(block.dropLast()),
        takenElsewhere: ["ex-c"]),
      SetlistBlockView(
        groupId: nil, pieceTitle: nil, relatedCount: 0, durationDisplay: "",
        entries: [.previewStandaloneExercise], piece: nil, related: [], takenElsewhere: []),
    ]
    return Row.rows(for: blocks, collapsed: [], isEditing: false)
  }

  @Test("the rows the table below is written against")
  func theRowsAreTheOnesTheTableAssumes() {
    #expect(rows.map(\.id) == ["setlist-1", "header-g1", "g-a", "g-b", "add-g1", "g-s"])
  }

  /// A row by id; the generated row type is not `Sendable`, so a test argument
  /// cannot carry it.
  enum R: Sendable {
    case e(String)
    case h(String)
    case a(String)

    var ref: BuilderRowRef {
      switch self {
      case .e(let id): .entry(entryId: id)
      case .h(let id): .header(groupId: id)
      case .a(let id): .addRelated(groupId: id)
      }
    }
  }

  @Test(
    "a drop names the dragged row and the rows it lands between",
    arguments: [
      (5, 0, R.e("g-s"), R.e("setlist-1"), nil),
      (5, 3, .e("g-s"), .e("g-b"), .e("g-a")),
      (1, 6, .h("g1"), nil, .e("g-s")),
      (0, 2, .e("setlist-1"), .e("g-a"), .h("g1")),
      (2, 5, .e("g-a"), .e("g-s"), .a("g1")),
    ] as [(Int, Int, R, R?, R?)])
  func aDropNamesItsNeighbours(from: Int, destination: Int, moved: R, before: R?, after: R?) {
    #expect(
      Row.drop(in: rows, from: from, to: destination)
        == .session(.moveRow(moved: moved.ref, before: before?.ref, after: after?.ref)))
  }

  @Test("a drop from a row that is not there sends nothing")
  func aDropFromNowhere() {
    #expect(Row.drop(in: rows, from: 9, to: 0) == nil)
  }
}
