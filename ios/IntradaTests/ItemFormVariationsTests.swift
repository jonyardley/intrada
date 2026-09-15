import Foundation
import SharedTypes
import Testing

@testable import Intrada

/// What the Add and Edit forms send for an exercise's variation rows (#1783).
/// Whether a ladder is valid is the core's call; this is only what the rows become.
@MainActor
struct ItemFormVariationsTests {
  private func rows(_ labels: [String]) -> [VariationRow] {
    labels.map { VariationRow(label: $0) }
  }

  private func ladder(_ event: ItemEvent?) -> [VariantEdit]? {
    guard case .updateVariants(_, let edits) = event else { return nil }
    return edits
  }

  @Test func aLoadedRowKeepsItsIdThroughARenameAndANewRowHasNone() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations[0].label = "Do"
    form.variations.append(VariationRow(label: "G"))

    let edits = ladder(form.editEvents(id: "exercise-2").last)

    #expect(edits?.map(\.id) == ["variation-c", "variation-f", "variation-bb", nil])
    #expect(edits?.map(\.label) == ["Do", "F", "B♭", "G"])
  }

  @Test func aSavedRowBlankedIsSentForTheCoreToRefuse() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations[0].label = "  "

    let edits = ladder(form.editEvents(id: "exercise-2").last)

    #expect(edits?.map(\.id) == ["variation-c", "variation-f", "variation-bb"])
    #expect(edits?.first?.label == "", "never a silent removal of a row with marks")
  }

  @Test func keyHiddenOverSavedRowsIsNotSent() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.key = "D"

    #expect(form.updateInput().key == nil)
  }

  @Test func aRowNeverFilledIsLeftOutAndLabelsAreTrimmed() {
    let form = ItemFormModel(kind: .exercise)
    form.variations = rows(["  C ", "", "   ", "G"])

    #expect(form.createInput().variantLabels == ["C", "G"])
  }

  @Test func aPieceSendsNoRowsAndShowsKey() {
    let form = ItemFormModel(kind: .exercise)
    form.variations = rows(["C"])
    #expect(!form.showsKey)

    form.kind = .piece

    #expect(form.createInput().variantLabels.isEmpty)
    #expect(form.showsKey)
  }

  @Test func keyHidesOnceARowExistsEvenBeforeItIsFilled() {
    let form = ItemFormModel(kind: .exercise)
    #expect(form.showsKey)

    form.variations = rows([""])

    #expect(!form.showsKey)
  }

  @Test func withRowsTheFieldsGoFirst() {
    let form = ItemFormModel(item: .previewExercise)
    form.variations = rows(["C"])

    let events = form.editEvents(id: "exercise-1")

    #expect(events.count == 2)
    #expect(form.updateInput().key == .some("C"), "sent, so the core can fold it into the rows")
    #expect(ladder(events.first) == nil, "fields first")
    #expect(ladder(events.last)?.map(\.label) == ["C"])
  }

  @Test func clearingEveryRowSendsTheEmptyLadderFirst() {
    let form = ItemFormModel(item: .previewExerciseWithVariations)
    form.variations = rows(["", " "])

    let events = form.editEvents(id: "exercise-2")

    #expect(events.count == 2)
    #expect(ladder(events.first) == [], "rows first, so a key after them is accepted")
  }

  @Test func anExerciseThatNeverHadRowsSendsOnlyItsFields() {
    let events = ItemFormModel(item: .previewExercise).editEvents(id: "exercise-1")

    #expect(events.count == 1)
    #expect(ladder(events.first) == nil)
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
