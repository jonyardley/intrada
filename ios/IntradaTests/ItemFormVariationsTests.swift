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

  private func set(_ event: ItemEvent?) -> (ids: [String], labels: [String])? {
    guard case .updateItemVariations(_, let ids, let labels) = event else { return nil }
    return (ids, labels)
  }

  @Test func aLoadedRowKeepsItsIdAndATypedRowIsALabel() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations.append(VariationRow(label: " Swung "))

    let sent = set(form.editEvents(id: "exercise-2").last)

    #expect(sent?.ids == ["variation-c", "variation-f", "variation-bb"])
    #expect(sent?.labels == ["Swung"])
  }

  @Test func aRemovedRowLeavesTheItemsSet() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations.remove(at: 1)

    #expect(set(form.editEvents(id: "exercise-2").last)?.ids == ["variation-c", "variation-bb"])
  }

  @Test func theKeyIsSentBesideSavedRows() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    let d = Key(letter: .d, accidental: .natural, mode: .major)
    form.key = d

    #expect(form.updateInput().key == .some(d))
  }

  @Test func aRowNeverFilledIsLeftOutAndLabelsAreTrimmed() {
    let form = ItemFormModel(kind: .exercise)
    form.variations = rows(["  C ", "", "   ", "G"])

    #expect(form.createInput().variationLabels == ["C", "G"])
  }

  @Test func aPieceSendsNoRows() {
    let form = ItemFormModel(kind: .exercise)
    form.variations = rows(["C"])

    form.kind = .piece

    #expect(form.createInput().variationLabels.isEmpty)
  }

  @Test func withRowsTheFieldsGoFirst() {
    let form = ItemFormModel(item: .previewExercise)
    form.variations = rows(["Slow"])

    let events = form.editEvents(id: "exercise-1")

    #expect(events.count == 2)
    #expect(set(events.first) == nil, "fields first")
    #expect(set(events.last)?.labels == ["Slow"])
  }

  @Test func clearingEveryRowSendsTheEmptySet() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations = rows(["", " "])

    let events = form.editEvents(id: "exercise-2")

    #expect(events.count == 2)
    #expect(set(events.last)?.ids == [])
    #expect(set(events.last)?.labels == [])
  }

  @Test func anExerciseWhoseRowsDidNotChangeSendsOnlyItsFields() {
    #expect(ItemFormModel(item: .previewExercise).editEvents(id: "exercise-1").count == 1)
    #expect(
      ItemFormModel(item: .previewExerciseWithVariations).editEvents(id: "exercise-2").count == 1)
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
