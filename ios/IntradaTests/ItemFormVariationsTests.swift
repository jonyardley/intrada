import Foundation
import SharedTypes
import Testing

@testable import Intrada

/// What the Add and Edit forms send for an exercise's variation rows (#1783,
/// #2246). Whether a set is valid is the core's call; this is only what the
/// rows become.
@MainActor
struct ItemFormVariationsTests {
  private func rows(_ labels: [String]) -> [VariationRow] {
    labels.map { VariationRow(label: $0) }
  }

  private func set(_ event: ItemEvent) -> (ids: [String], labels: [String])? {
    guard case .edit(_, _, let ids, let labels) = event else { return nil }
    return (ids, labels)
  }

  @Test func aLoadedRowKeepsItsIdAndATypedRowIsALabel() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations.append(VariationRow(label: "Swung"))

    let sent = set(form.editEvent(id: "exercise-2"))

    #expect(sent?.ids == ["variation-c", "variation-f", "variation-bb"])
    #expect(sent?.labels == ["Swung"])
  }

  @Test func aSavedRowIsSentByIdAndItsLabelNever() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations[0].label = "Renamed"
    form.variations.swapAt(0, 2)

    let sent = set(form.editEvent(id: "exercise-2"))

    #expect(sent?.ids == ["variation-bb", "variation-f", "variation-c"])
    #expect(sent?.labels == [])
  }

  @Test func aRemovedRowLeavesTheItemsSet() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations.remove(at: 1)

    #expect(set(form.editEvent(id: "exercise-2"))?.ids == ["variation-c", "variation-bb"])
  }

  @Test func theKeyIsSentBesideSavedRows() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    let d = Key(letter: .d, accidental: .natural, mode: .major)
    form.key = d

    let event = form.editEvent(id: "exercise-2")

    guard case .edit(_, let input, let ids, _) = event else {
      Issue.record("expected one edit event")
      return
    }
    #expect(input.key == .set(key: d))
    #expect(ids == ["variation-c", "variation-f", "variation-bb"])
  }

  @Test func theCoreKeepsTheTypedRowsTrimmed() throws {
    #expect(try added(as: .exercise, typing: ["  C ", "", "   ", "G"]) == ["C", "G"])
  }

  @Test func theCoreKeepsNoRowsOnAPiece() throws {
    #expect(try added(as: .piece, typing: ["C"]) == [])
  }

  private func added(as kind: ItemKind, typing typed: [String]) throws -> [String] {
    let form = ItemFormModel(kind: .exercise)
    form.title = "Scales"
    form.variations = rows(typed)
    form.kind = kind
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(.item(.add(form.createInput())))
    return try #require(try bridge.rendered().items.first).variations.map(\.label)
  }

  @Test func clearingEveryRowSendsNoSavedRows() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations = rows(["", " "])

    let sent = set(form.editEvent(id: "exercise-2"))

    #expect(sent?.ids == [])
  }

  @Test func aDroppedRowLandsBeforeTheTarget() {
    var list = rows(["C", "G", "D"])

    list.move(list[2].id, before: list[0].id)

    #expect(list.map(\.label) == ["D", "C", "G"])
  }

  @Test func movingPastEitherEndDoesNothing() {
    var list = rows(["C", "G"])

    list.move(list[0].id, by: -1)
    list.move(list[1].id, by: 1)
    #expect(list.map(\.label) == ["C", "G"])

    list.move(list[0].id, by: 1)
    #expect(list.map(\.label) == ["G", "C"])
  }
}
