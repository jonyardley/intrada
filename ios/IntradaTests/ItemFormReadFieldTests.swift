import SharedTypes
import Testing

@testable import Intrada

/// The provenance half of the add form (#1436): which fields still hold what a
/// photographed page was read into. The core decides what a page *says*; this
/// is only about how long the form goes on claiming it.
@MainActor
struct ItemFormReadFieldTests {
  private func filled() -> ItemFormModel {
    let form = ItemFormModel(kind: .piece)
    form.fill(from: .readPage)
    return form
  }

  @Test func aReadPageFillsTheFieldsItCouldRead() {
    let form = filled()

    #expect(form.title == "Autumn Leaves")
    #expect(form.composer == "Joseph Kosmo")
    #expect(form.marking == "Moderato")
    #expect(form.bpm == "120")
  }

  @Test func everyFilledFieldSaysItCameFromThePhoto() {
    let form = filled()

    #expect(form.readFrom[.title] == false)
    #expect(form.readFrom[.marking] == false)
    #expect(form.readFrom[.bpm] == false)
  }

  /// The one thing the mark has to distinguish, and the whole reason the core
  /// sends `weak` rather than a confidence for the shell to threshold.
  @Test func aWeakReadIsMarkedAsOne() {
    #expect(filled().readFrom[.composer] == true)
  }

  /// The mark clears on the keystroke, not on submit: from the moment the user
  /// corrects a field, the value is theirs and must stop claiming to be the
  /// page's. Delete the write-through in `composer` and this passes forever.
  @Test func editingAFieldTakesItsMarkOff() {
    let form = filled()

    form.composer = "Joseph Kosma"

    #expect(form.readFrom[.composer] == nil)
    #expect(form.readFrom[.title] == false, "the fields they did not touch keep theirs")
  }

  /// A value typed before scanning is the user's. Deleting the `isEmpty`
  /// guards lets a scan overwrite the title they had already written.
  @Test func aScanNeverOverwritesWhatTheUserAlreadyTyped() {
    let form = ItemFormModel(kind: .piece)
    form.title = "The name I gave it"

    form.fill(from: .readPage)

    #expect(form.title == "The name I gave it")
    #expect(form.readFrom[.title] == nil, "it was never the photo's, so it carries no mark")
    #expect(form.composer == "Joseph Kosmo", "the fields they left empty still fill")
  }

  /// Nothing recognised is written by reading it — the non-goal the whole
  /// feature is built around. Pressing Add is what writes.
  @Test func fillingTheFormDoesNotSubmitIt() {
    let form = ItemFormModel(kind: .piece)
    #expect(!form.canSubmit)

    form.fill(from: .readPage)

    #expect(form.canSubmit, "it is now submittable, but only the user can submit it")
    #expect(form.createInput().title == "Autumn Leaves")
  }
}
