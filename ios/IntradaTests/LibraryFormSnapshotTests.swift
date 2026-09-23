import IntradaCoreFFI
import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Intrada

final class LibraryFormSnapshotTests: SnapshotTestCase {
  func testLibraryAddScreenWithError() {
    assertSnapshot(
      of: host(LibraryAddScreen(previewError: "A piece needs a composer.")), as: config)
  }

  func testLibraryEditScreenWithError() {
    assertSnapshot(
      of: host(LibraryEditScreen(item: .previewDetail, previewError: "A piece needs a composer.")),
      as: config)
  }

  func testLibraryAddScreen() {
    assertSnapshot(of: host(LibraryAddScreen()), as: config)
  }

  func testLibraryAddScreenExercise() {
    assertSnapshot(of: host(LibraryAddScreen(defaultKind: .exercise)), as: config)
  }

  /// #1783, #1831: rows in place of Key, which the core hides once a row exists,
  /// and a refused rung marking the section while the banner names it.
  func testLibraryAddScreenMarksTheVariations() {
    let form = ItemFormModel(kind: .exercise)
    form.title = "Arpeggios"
    form.variations = ["C", "F", "c"].map { VariationRow(label: $0) }
    form.formError = "Duplicate variation \u{201c}c\u{201d}"
    form.mark(.piece(field: .variations))
    assertSnapshot(of: host(LibraryAddScreen(previewForm: form)), as: config)
  }

  func testLibraryAddScreenStaged() {
    let form = ItemFormModel(kind: .piece)
    form.title = "Alice in Wonderland"
    form.composer = "Sammy Fain"
    form.chartText = "[A]\n| Dm7 | G7 | Cmaj7 | A7alt |\n| Dm7 | G7 | Cmaj7 | Cmaj7 |"
    form.stagedExercises = [
      .draft(
        id: UUID(), title: "Guide tones, ii to V to I", key: "C", modality: .major, bpm: "80"),
      .existing(id: "ex-1", title: "Shell voicings", meta: "C major"),
    ]
    assertSnapshot(of: host(LibraryAddScreen(previewForm: form)), as: config)
  }

  /// #1595: the banner says what, the field says where. The wash and the
  /// recoloured label are the whole treatment, so a pixel diff is what holds
  /// them.
  func testLibraryAddScreenMarksTheFieldAtFault() {
    let form = ItemFormModel(kind: .piece)
    form.title = "Alice in Wonderland"
    form.formError = "Composer must be between 1 and 200 characters"
    form.mark(.piece(field: .composer))
    assertSnapshot(of: host(LibraryAddScreen(previewForm: form)), as: config)
  }

  /// #1595: the whole form with a chart staged and two rows, the second one
  /// marked. This is what holds the wiring from the target the core sent to the
  /// row and section that carry it, which the component snapshots cannot see.
  func testLibraryAddScreenMarksTheStagedRowInContext() {
    let form = ItemFormModel(kind: .piece)
    form.title = "Alice in Wonderland"
    form.composer = "Sammy Fain"
    form.chartText = "| Dm7 | G7 | Cmaj7 | A7alt |"
    form.stagedExercises = [
      .existing(id: "ex-1", title: "Shell voicings", meta: "C major"),
      .draft(id: UUID(), title: "Untitled", key: "C", modality: .major, bpm: "80"),
    ]
    form.formError = "Title must be between 1 and 500 characters"
    form.mark(.exercise(index: 1, field: .title))
    assertSnapshot(of: host(LibraryAddScreen(previewForm: form)), as: tallFormConfig)
  }

  /// #1436: the composer was read weakly, so its mark's glyph is dimmed
  /// (#1458: the label itself stays at `inkSecondary`), and a pixel diff can
  /// hold that.
  func testLibraryAddScreenReadFromAPhoto() {
    assertSnapshot(of: host(addForm(from: .readPage)), as: config)
  }

  /// The mark clears on the keystroke, not on submit: the composer is the
  /// user's from the moment they correct it.
  func testLibraryAddScreenAfterEditingAReadField() {
    let form = ItemFormModel(kind: .piece)
    form.fill(from: .readPage)
    form.composer = "Joseph Kosma"
    assertSnapshot(of: host(addForm(form)), as: config)
  }

  /// A read that failed, or one the device could not run, must not render
  /// identically to one that worked.
  func testScanPageEntryStates() {
    let stub = UIGraphicsImageRenderer(size: CGSize(width: 60, height: 80)).image { context in
      UIColor(IntradaColor.surfaceSunken).setFill()
      context.fill(CGRect(origin: .zero, size: CGSize(width: 60, height: 80)))
    }
    let states: [(PhotoRecognitionStatus, Bool)] = [
      (.idle, false), (.reading, false), (.ready, false), (.ready, true),
      (.failed, false),
    ]
    let stack = VStack(spacing: IntradaSpacing.card) {
      ForEach(Array(states.enumerated()), id: \.offset) { _, state in
        ScanPageEntry(
          photoId: state.0 == .idle ? nil : "01JB0000000000000000000000",
          status: state.0, readNothing: state.1, onCaptured: { _ in },
          loadImage: { _ in stub })
      }
    }
    .padding(IntradaSpacing.card)
    .frame(width: 390)
    .background(PaperBackground())

    assertSnapshot(of: stack, as: .image(layout: .sizeThatFits))
  }

  /// #1595: the row the refused save named wears the wash and a danger bar in
  /// place of its kind bar, and only that row does.
  func testDraftItemRowAtFault() {
    let rows = VStack(spacing: 0) {
      DraftItemRow(title: "Shell voicings", meta: "C major", onRemove: {})
      HairlineDivider()
      DraftItemRow(
        title: "Untitled", meta: nil, faulted: true, faultedField: .title, onRemove: {})
    }
    .background(IntradaColor.cardFill)
    .padding(IntradaSpacing.card)
    .frame(width: 390)
    .background(PaperBackground())

    assertSnapshot(of: rows, as: .image(layout: .sizeThatFits))
  }

  /// #1595: a refused bar puts a danger edge on the chart block, the one thing
  /// that changes on a chart the shell cannot parse.
  func testStagedChartCardAtFault() {
    let card = VStack(spacing: IntradaSpacing.card) {
      StagedChartCard(
        text: "| Dm7 | G7 | Hxyz | Cmaj7 |", readWeakly: nil, faulted: true,
        faultedBarNumber: 3, onEdit: {}
      )
      .cardSurface()
      StagedChartCard(
        text: "| Dm7 | G7 | Cmaj7 | A7alt |", readWeakly: nil, onEdit: {}
      )
      .cardSurface()
    }
    .padding(IntradaSpacing.card)
    .frame(width: 390)
    .background(PaperBackground())

    assertSnapshot(of: card, as: .image(layout: .sizeThatFits))
  }

  func testDraftItemRows() {
    let rows = VStack(spacing: 0) {
      DraftItemRow(title: "Guide tones, ii to V to I", meta: "C major · 80 bpm", onRemove: {})
      HairlineDivider()
      DraftItemRow(title: "Shell voicings", meta: "C major", onRemove: {})
      HairlineDivider()
      DraftItemRow(title: "No key or tempo yet", meta: nil, onRemove: {})
    }
    .background(IntradaColor.cardFill)
    .padding(IntradaSpacing.card)
    .frame(width: 390)
    .background(PaperBackground())

    assertSnapshot(of: rows, as: .image(layout: .sizeThatFits))
  }

  private func addForm(from draft: PhotoDraft) -> some View {
    let form = ItemFormModel(kind: .piece)
    form.fill(from: draft)
    return addForm(form)
  }

  private func addForm(_ form: ItemFormModel) -> some View {
    ItemFormScaffold(
      form: form, title: "New Piece", confirmLabel: "Add", composerSuggestions: [],
      tagSuggestions: []
    ) {}
  }

  func testLibraryEditScreen() {
    assertSnapshot(of: host(LibraryEditScreen(item: .previewDetail)), as: config)
  }

  func testLibraryEditScreenExercise() {
    assertSnapshot(of: host(LibraryEditScreen(item: .previewExercise)), as: config)
  }

  /// #1783: the saved rows, ready to rename, reorder and remove.
  func testLibraryEditScreenExerciseWithVariations() {
    assertSnapshot(
      of: host(LibraryEditScreen(item: .previewExerciseWithVariations)), as: config)
  }

  func testKeyPickerCollapsed() {
    let pickers = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        VStack(spacing: 0) {
          KeyPicker(label: "Key", key: .constant(""), modality: .constant(nil))
        }.cardSurface()
        VStack(spacing: 0) {
          KeyPicker(label: "Key", key: .constant("Gb"), modality: .constant(.major))
        }.cardSurface()
      }
      .padding(16)
    }
    assertSnapshot(of: host(pickers), as: config)
  }

  func testKeyPickerExpandedEmpty() {
    let picker = ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        KeyPicker(
          label: "Key", key: .constant(""), modality: .constant(nil), initiallyExpanded: true)
      }
      .cardSurface()
      .padding(16)
    }
    assertSnapshot(of: host(picker), as: config)
  }

  func testKeyPickerExpandedEnharmonic() {
    let picker = ZStack {
      PaperBackground()
      VStack(spacing: 0) {
        KeyPicker(
          label: "Key", key: .constant("Gb"), modality: .constant(.major), initiallyExpanded: true)
      }
      .cardSurface()
      .padding(16)
    }
    assertSnapshot(of: host(picker), as: config)
  }

  func testAutocompleteField() {
    let pool = ["Bach", "Beethoven", "Brahms", "Chopin", "Debussy"]
    let fields = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        VStack(spacing: 0) {
          AutocompleteField(
            label: "Composer", text: .constant("B"), suggestions: pool,
            initiallyShowingSuggestions: true)
        }.cardSurface()
        VStack(spacing: 0) {
          AutocompleteField(label: "Composer", text: .constant("Ravel"), suggestions: pool)
        }.cardSurface()
      }
      .padding(16)
    }
    assertSnapshot(of: host(fields), as: config)
  }

  func testTagChipInput() {
    let pool = ["classical", "recital", "jazz", "warm-up", "technique", "etude"]
    let fields = ZStack {
      PaperBackground()
      VStack(spacing: 16) {
        VStack(spacing: 0) {
          TagChipInput(
            label: "Tags", tags: .constant(["classical", "recital"]), suggestions: pool,
            initiallyShowingSuggestions: true)
        }.cardSurface()
        VStack(spacing: 0) {
          TagChipInput(label: "Tags", tags: .constant([]), suggestions: pool)
        }.cardSurface()
      }
      .padding(16)
    }
    assertSnapshot(of: host(fields), as: config)
  }

  func testLinkedExercisePicker() {
    // Three exercises; the first is already related (pre-selected → check), the
    // rest show the outlined add control.
    let sheet = LinkedItemPickerSheet(
      kind: .exercise,
      available: [
        .previewExercise,
        LibraryItemFixture.view(
          id: "exercise-2", itemType: .exercise, title: "Db Major Scale", key: "Db",
          modality: .major),
        LibraryItemFixture.view(id: "exercise-3", itemType: .exercise, title: "Arpeggios in Db"),
      ],
      linkedIds: ["exercise-1"],
      onApply: { _, _ in })
    assertSnapshot(of: host(sheet), as: config)
  }

  // #1616: the create trigger plus a drafted row, alongside the ordinary list.
  func testLinkedExercisePickerWithDraft() {
    let sheet = LinkedItemPickerSheet(
      kind: .exercise,
      available: [.previewExercise],
      linkedIds: [],
      existingDrafts: [
        .draft(
          id: UUID(), title: "Guide tones, ii to V to I", key: "C", modality: .major, bpm: "80")
      ],
      onApply: { _, _ in })
    assertSnapshot(of: host(sheet), as: config)
  }

  func testLinkedPiecePicker() {
    let sheet = LinkedItemPickerSheet(
      kind: .piece,
      available: [.previewPiece, piece(id: "piece-9", "Blue Bossa", "Kenny Dorham")],
      linkedIds: [LibraryItemView.previewPiece.id],
      onApply: { _, _ in })
    assertSnapshot(of: host(sheet), as: config)
  }

  /// The four states a simulator can reach. There is no camera on one, so the
  /// live preview and the capture itself are only checkable on a device
  /// (#1460); what these pin is that the chrome over the backdrop stays legible
  /// and laid out.
  func testPageCameraStates() {
    let states = ZStack {
      IntradaColor.viewerBackdrop
      VStack(spacing: 24) {
        PageCameraShutter(disabled: false, onPress: {})
        PageCameraFailure(message: "Couldn't take the photo. Try again.")
        PageCameraBlocked(access: .denied, onOpenSettings: {})
        PageCameraUnstartable()
      }
    }
    assertSnapshot(of: host(states), as: config)
  }

  /// The confirm step, which is the fix: the page you approve is the page that
  /// gets stored, where the scanner kept one shot of several without saying so.
  func testPageCameraConfirm() {
    let confirm = ZStack {
      IntradaColor.viewerBackdrop
      CapturedPageConfirm(page: Self.page, onKeep: {}, onRetake: {})
        .padding(16)
    }
    assertSnapshot(of: host(confirm), as: config)
  }

  private func piece(id: String, _ title: String, _ composer: String) -> LibraryItemView {
    LibraryItemFixture.view(id: id, title: title, subtitle: composer)
  }
}
