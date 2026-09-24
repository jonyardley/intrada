import IntradaCoreFFI
import SharedTypes
import XCTest

@testable import Intrada

@MainActor
final class LibraryBridgeTests: XCTestCase {

  // ── Real bridge (Swift↔Rust bincode round-trip) ────────────────────────

  /// Real-bridge bincode round-trip (#846): calls LiveBridge directly (not via
  /// Store) so a serialization throw surfaces instead of being swallowed by
  /// Store.send's `guarded`.
  func testRealBridgeEditAppliesToViewModel() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Original", kind: .piece, composer: "Bach", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))

    let afterAdd = try bridge.view()
    XCTAssertEqual(
      afterAdd.items.count, 1,
      "add should land: count=\(afterAdd.items.count) err=\(afterAdd.error ?? "nil")")
    let id = try XCTUnwrap(afterAdd.items.first?.id)

    // Mirrors ItemFormModel.updateInput(): every PATCH field set, type flipped.
    _ = try bridge.update(
      .item(
        .update(
          id: id,
          input: UpdateItem(
            title: "Renamed", kind: .exercise, composer: .some("Bach"), key: .some(nil),
            modality: .some(nil), tempo: .some(nil), notes: .some(nil), tags: nil, priority: nil))))

    let afterEdit = try bridge.view()
    XCTAssertEqual(
      afterEdit.items.first?.title, "Renamed",
      "edited title should apply (err=\(afterEdit.error ?? "nil"))")
    XCTAssertEqual(afterEdit.items.first?.itemType, .exercise, "edited type should apply")
  }

  /// Real-bridge wire pin (#846, #1467): a `Bool` that never made it across
  /// reads as `false`, so the row would say "steps" about a ladder of keys,
  /// no crash, no error, just the wrong noun.
  func testRealBridgeLadderIsKeysCrossesTheWire() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Shells", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(
      .item(.setVariants(id: id, labels: ["F major", "B\u{266D}"])))

    let keys = try bridge.view()
    XCTAssertEqual(
      keys.items.first { $0.id == id }?.ladderIsKeys, true,
      "a ladder of key names comes back as keys (err=\(keys.error ?? "nil"))")

    _ = try bridge.update(
      .item(.setVariants(id: id, labels: ["F major", "B\u{266D}", "Hands together"])))

    XCTAssertEqual(
      try bridge.view().items.first { $0.id == id }?.ladderIsKeys, false,
      "one non-key rung and the whole ladder is steps")
  }

  /// Moved from `VariationManagementUITests` (#1825): a removed variation is archived, not hard-deleted, but drops from the view.
  func testRealBridgeRemovingAVariationDropsItFromTheLadder() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Major Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)
    _ = try bridge.update(.item(.setVariants(id: id, labels: ["C", "G", "D", "A", "E"])))

    _ = try bridge.update(.item(.setVariants(id: id, labels: ["C", "G", "D", "E"])))

    let labels = try bridge.view().items.first { $0.id == id }?.variants.map(\.label)
    XCTAssertEqual(labels, ["C", "G", "D", "E"], "A is gone, the others keep their order")
  }

  /// Real-bridge pin for the Edit form's one-write ladder (#1783): the id on a
  /// row crosses the wire and keeps the row through a rename, beside a fresh
  /// row and a reorder in the same event.
  func testRealBridgeUpdateVariantsRenamesByIdInOneWrite() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Major Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: ["C", "G"]))))
    let item = try XCTUnwrap(try bridge.view().items.first)
    let cId = try XCTUnwrap(item.variants.first { $0.label == "C" }?.id)
    let gId = try XCTUnwrap(item.variants.first { $0.label == "G" }?.id)

    _ = try bridge.update(
      .item(
        .updateVariants(
          id: item.id,
          variants: [
            VariantEdit(id: gId, label: "Sol"), VariantEdit(id: nil, label: "A"),
            VariantEdit(id: cId, label: "C"),
          ])))

    let after = try XCTUnwrap(try bridge.view().items.first { $0.id == item.id })
    XCTAssertEqual(after.variants.map(\.label), ["Sol", "A", "C"])
    XCTAssertEqual(after.variants.first?.id, gId, "renamed in place, marks intact")
    XCTAssertFalse(after.showsKey, "an exercise in several keys has no single key")
  }

  /// The new `FormErrorField` case decodes on the wire (#846, #1831): a
  /// refused rung points the form at the Variations section.
  func testRealBridgeRefusedVariationPointsAtTheSection() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Major Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: ["C", "c"]))))

    let view = try bridge.view()
    XCTAssertNotNil(view.error)
    XCTAssertEqual(view.errorTarget, .piece(field: .variations))
    XCTAssertTrue(view.items.isEmpty, "nothing written")
  }

  /// The Add form has no saved exercise to read `showsKey` from, so it asks
  /// the core with its own row count (#1783 decision 1).
  func testExerciseFormShowsKeyOnlyWithNoRows() {
    XCTAssertTrue(exerciseFormShowsKey(liveVariantCount: 0))
    XCTAssertFalse(exerciseFormShowsKey(liveVariantCount: 1))
  }

  /// The Edit form's two events against the real core (#1783): a key folded into
  /// a new ladder must not come back as a second key, and a key typed after
  /// clearing the rows must not be refused while they still stand.
  func testRealBridgeEditFormFoldsAKeyInAndTakesOneBack() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Arpeggios", kind: .exercise, composer: nil, key: "G", modality: .major,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let keyed = try XCTUnwrap(try bridge.view().items.first)

    let adding = ItemFormModel(item: keyed)
    adding.variations = ["C", "F"].map { VariationRow(label: $0) }
    for event in adding.editEvents(id: keyed.id) {
      _ = try bridge.update(.item(event))
      XCTAssertNil(try bridge.view().error)
    }
    let laddered = try XCTUnwrap(try bridge.view().items.first { $0.id == keyed.id })
    XCTAssertEqual(
      laddered.variants.map(\.label), ["G", "C", "F"], "the key became the first rung")

    let refilling = ItemFormModel(item: laddered)
    refilling.variations = []
    refilling.key = "D"
    refilling.variations = [VariationRow(label: "A")]
    for event in refilling.editEvents(id: keyed.id) {
      _ = try bridge.update(.item(event))
      XCTAssertNil(try bridge.view().error, "a key chosen while no rows showed is not sent")
    }
    let refilled = try XCTUnwrap(try bridge.view().items.first { $0.id == keyed.id })
    XCTAssertEqual(refilled.variants.map(\.label), ["A"])

    let clearing = ItemFormModel(item: refilled)
    clearing.variations = []
    clearing.key = "D"
    for event in clearing.editEvents(id: keyed.id) {
      _ = try bridge.update(.item(event))
      XCTAssertNil(try bridge.view().error)
    }
    let rekeyed = try XCTUnwrap(try bridge.view().items.first { $0.id == keyed.id })
    XCTAssertTrue(rekeyed.variants.isEmpty)
    XCTAssertEqual(rekeyed.key, "D")
  }

  /// Real-bridge wire pin for the photo id (#846, #1355): the Swift serializer,
  /// the Rust deserializer and the `ViewModel` projection must all agree on the
  /// new `Item` field and the two new `ItemEvent` variants. A stub bridge
  /// cannot catch a break in any of the three.
  func testRealBridgePhotoIdCrossesTheWireBothWays() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Nocturne", kind: .piece, composer: "Chopin", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.item(.setPhoto(id: id, photoId: "01ARZ3NDEKTSV4RRFFQ69G5FAV")))

    let afterSet = try bridge.view()
    XCTAssertNil(afterSet.error, "setPhoto should cross the wire cleanly")
    XCTAssertEqual(
      afterSet.items.first { $0.id == id }?.photoId, "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "the id the screens read comes back through the projection")

    _ = try bridge.update(.item(.clearPhoto(id: id)))

    XCTAssertNil(
      try bridge.view().items.first { $0.id == id }?.photoId,
      "an absent Option must decode as absent, not as the previous value")
  }

  /// Every recognition type is new on the bincode wire, and `f32` is a new
  /// scalar on it. A stub bridge cannot catch a wire break (#846), so this
  /// drives the whole round trip: the effect out, a `RecognitionOutput` built
  /// in Swift back in, and the draft the form will read out of the projection.
  func testRealBridgeReadPhotoFillsTheDraft() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    let photoId = Ulid.generate()

    let requests = try bridge.update(.item(.readPhoto(photoId: photoId)))
    let request = try XCTUnwrap(
      requests.first { if case .recognition = $0.effect { return true } else { return false } },
      "readPhoto should emit a Recognition effect")
    guard case .recognition(.readPage(let asked)) = request.effect else {
      return XCTFail("expected readPage, got \(request.effect)")
    }
    XCTAssertEqual(asked, photoId, "the core names the file the shell just wrote")

    _ = try bridge.resolve(
      request.id,
      recognitionOutput: .page(
        PageReading(
          lines: [
            RecognisedLine(
              text: "Autumn Leaves", x: 0.1, y: 0.08, width: 0.8, height: 0.09, confidence: 0.93),
            RecognisedLine(
              text: "Music by Joseph Kosma", x: 0.1, y: 0.18, width: 0.8, height: 0.03,
              confidence: 0.4),
          ],
          suggested: nil)))

    let recognition = try bridge.view().photoRecognition
    XCTAssertEqual(recognition.status, .ready)
    XCTAssertEqual(recognition.photoId, photoId)
    let draft = try XCTUnwrap(recognition.draft)
    XCTAssertEqual(draft.title?.value, "Autumn Leaves")
    XCTAssertEqual(draft.composer?.value, "Joseph Kosma")
    XCTAssertEqual(draft.title?.source, .recognised)
    XCTAssertEqual(
      try XCTUnwrap(draft.title?.confidence), Float(0.93), accuracy: 0.0001,
      "an f32 has to survive the wire, not arrive as a rounded or byte-swapped number")
    XCTAssertFalse(draft.title?.weak ?? true, "the title was read cleanly")
    XCTAssertTrue(draft.composer?.weak ?? false, "the credit was read weakly")
  }

  /// `CreateItem` gained a field, and it is the field that stops the user
  /// photographing the same page twice (#1436).
  func testRealBridgeCreateCarriesTheScannedPage() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    let photoId = Ulid.generate()

    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Cry Me A River", kind: .piece, composer: "Arthur Hamilton", key: nil,
            modality: nil, tempo: nil, notes: nil, tags: [], photoId: photoId, variantLabels: []))))

    let view = try bridge.view()
    XCTAssertNil(view.error)
    XCTAssertEqual(
      view.items.first?.photoId, photoId,
      "the page the form was read off has to reach the piece it created")
  }

  /// The shell mints a photo's id (offline-first invariant 3) and the core
  /// refuses any id that is not a ulid, so `Ulid` and Rust's parser have to
  /// agree.
  func testRealBridgeAcceptsAUlidTheShellMinted() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Gymnopedie", kind: .piece, composer: "Satie", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)
    let minted = Ulid.generate()

    _ = try bridge.update(.item(.setPhoto(id: id, photoId: minted)))

    let after = try bridge.view()
    XCTAssertNil(after.error, "the core's validate_photo_id must accept what Ulid mints")
    XCTAssertEqual(after.items.first { $0.id == id }?.photoId, minted)
  }

  /// Real-bridge round trip for the item's metre (#1499): `SetMetre` carries an
  /// optional nested struct with an optional list inside it, the #846 shape.
  func testRealBridgeSetsAndClearsTheItemsMetre() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Take Five", kind: .piece, composer: "Desmond", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)
    let metre = Metre(beats: 5, unit: 4, groups: [3, 2])

    _ = try bridge.update(.item(.setMetre(id: id, metre: metre)))
    XCTAssertEqual(try bridge.view().items.first?.metre, metre)
    XCTAssertNil(try bridge.view().error)

    _ = try bridge.update(.item(.setMetre(id: id, metre: nil)))
    XCTAssertNil(try bridge.view().items.first?.metre)
  }

  /// but returns a nested `ChordChart` + `ScaffoldPreviewView` across the bincode
  /// wire, a shape a stub bridge can't exercise. A bad chart must surface an
  /// error and never store a partial.
  func testRealBridgeSetChordChartDerivesScaffoldPreview() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma", key: "G",
            modality: .minor, tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 | F7 | Bbmaj7 |")))
    let ok = try bridge.view()
    let piece = try XCTUnwrap(ok.items.first { $0.id == id })
    let preview = try XCTUnwrap(
      piece.scaffoldPreview, "charted piece derives a preview (err=\(ok.error ?? "nil"))")
    XCTAssertEqual(preview.key, "G")
    XCTAssertEqual(preview.specs.count, 5, "five generators")
    XCTAssertEqual(piece.chordChart?.sections.first?.bars.count, 3, "three bars round-trip")

    // A bad token surfaces an error and leaves the prior chart intact.
    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 | Hxyz |")))
    let bad = try bridge.view()
    XCTAssertNotNil(bad.error, "a parse error must surface, not vanish (#846)")
    XCTAssertEqual(
      bad.items.first { $0.id == id }?.chordChart?.sections.first?.bars.count, 3,
      "a failed parse never overwrites the stored chart")
  }

  /// Real-bridge commit (#1106): `CommitScaffold` carries a `Vec<ScaffoldKind>`:
  /// round-trip it through the live bincode bridge so the write payload can't
  /// silently misalign (#846), and assert the core materialises + links the
  /// selected exercises.
  func testRealBridgeCommitScaffoldLinksExercises() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma", key: "G",
            modality: .minor, tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)
    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 | F7 | Bbmaj7 |")))

    _ = try bridge.update(
      .item(.commitScaffold(pieceId: id, kinds: [.shells, .guideToneLines])))

    let after = try bridge.view()
    XCTAssertNil(after.error, "commit surfaces no error (err=\(after.error ?? "nil"))")
    let piece = try XCTUnwrap(after.items.first { $0.id == id })
    XCTAssertEqual(piece.linkedExercises.count, 2, "two selected kinds become linked exercises")
    let titles = Swift.Set(piece.linkedExercises.map(\.title))
    XCTAssertTrue(titles.contains("Shells") && titles.contains("Guide-tone lines"))

    // Re-committing the same kinds dedups: no duplicate exercises.
    _ = try bridge.update(.item(.commitScaffold(pieceId: id, kinds: [.shells])))
    let reran = try bridge.view()
    let shells = reran.items.filter { $0.title == "Shells" }
    XCTAssertEqual(shells.count, 1, "re-commit adds no duplicate (#1106 dedup)")
  }

  /// Real-bridge one-pass create (#1390): `AddPieceInFull` carries a nested
  /// `CreateItem`, an optional raw chart and a `Vec<ScaffoldEntry>` whose two
  /// variants have different payload shapes, so a stub bridge cannot prove the
  /// wire holds (#846). Pinned here before any screen sends it.
  func testRealBridgeAddPieceInFullCarriesChartAndExercises() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Shell voicings", kind: .exercise, composer: nil, key: "G",
            modality: nil, tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let existingId = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma", key: "G",
            modality: .minor, tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []),
          chart: "| Cm7 | F7 | Bbmaj7 |",
          exercises: [
            .existing(id: existingId),
            .new(
              CreateItem(
                title: "Enclosures", kind: .exercise, composer: nil, key: nil, modality: nil,
                tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: [])),
          ])))

    let after = try bridge.view()
    XCTAssertNil(after.error, "the one-pass create surfaces no error")
    let piece = try XCTUnwrap(after.items.first { $0.title == "Autumn Leaves" })
    XCTAssertEqual(
      piece.chordChart?.sections.first?.bars.count, 3,
      "the chart parsed on the way in, with no second event")
    XCTAssertEqual(
      piece.linkedExercises.map(\.title), ["Shell voicings", "Enclosures"],
      "chosen then written, in the order given: neither minting order nor sorted")

    // A bar the parser rejects writes nothing at all: no piece, no exercise.
    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Blue in Green", kind: .piece, composer: nil, key: "G", modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []),
          chart: "| Cm7 | Hxyz |",
          exercises: [
            .new(
              CreateItem(
                title: "Orphan", kind: .exercise, composer: nil, key: nil, modality: nil,
                tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))
          ])))

    let rejected = try bridge.view()
    XCTAssertNotNil(rejected.error, "a parse error must surface, not vanish (#846)")
    XCTAssertNil(
      rejected.items.first { $0.title == "Blue in Green" }, "no half-made piece is left behind")
    XCTAssertNil(rejected.items.first { $0.title == "Orphan" }, "and no orphan exercise either")
  }

  /// Real-bridge error target (#1595): three variants with different payload
  /// shapes, one of them a nested optional enum, so only the live bridge proves
  /// the wire holds (#846). The screens read this to point at the failure.
  func testRealBridgeRejectedCreateCarriesWhereItFailed() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Blue in Green", kind: .piece, composer: "Bill Evans", key: "G",
            modality: nil, tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []),
          chart: "| Cm7 | Hxyz |",
          exercises: [])))

    XCTAssertEqual(
      try bridge.view().errorTarget, .chartBar(barNumber: 2, token: "Hxyz"),
      "the bar and the token, so the chart section can highlight in place")

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Blue in Green", kind: .piece, composer: "Bill Evans", key: "G",
            modality: nil, tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []),
          chart: "swing feel",
          exercises: [])))

    XCTAssertEqual(
      try bridge.view().errorTarget, .chart,
      "prose with no bars fails at no bar, so the whole section is what is marked")

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Blue in Green", kind: .piece, composer: "Bill Evans", key: "G",
            modality: nil, tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []),
          chart: nil,
          exercises: [
            .new(
              CreateItem(
                title: "Enclosures", kind: .exercise, composer: nil, key: nil, modality: nil,
                tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: [])),
            .new(
              CreateItem(
                title: "   ", kind: .exercise, composer: nil, key: nil, modality: nil,
                tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: [])),
          ])))

    XCTAssertEqual(
      try bridge.view().errorTarget, .exercise(index: 1, field: .title),
      "the second row is the blank one, and its title is what to mark")

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Blue in Green", kind: .piece, composer: String(repeating: "x", count: 201),
            key: "G", modality: nil, tempo: nil, notes: nil, tags: [], photoId: nil,
            variantLabels: []),
          chart: nil,
          exercises: [])))

    XCTAssertEqual(try bridge.view().errorTarget, .piece(field: .composer))

    _ = try bridge.update(.item(.setVariants(id: "no-such-exercise", labels: ["C"])))

    let unrelated = try bridge.view()
    XCTAssertNotNil(unrelated.error, "a failure with no field still reports what went wrong")
    XCTAssertNil(
      unrelated.errorTarget, "but must not leave the form pointing at the last failure")
  }

  /// Real-bridge priority toggle (#763): the star sends an UpdateItem with every
  /// optional field "no change" (outer nil) and only `priority` set, a different
  /// bincode shape than the full edit, so round-trip it through the live bridge to
  /// catch an absent-vs-present wire break (#846).
  func testRealBridgePriorityToggleAppliesToViewModel() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Etude", kind: .piece, composer: "Chopin", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let item = try XCTUnwrap(try bridge.view().items.first)
    XCTAssertFalse(item.priority, "new items start non-priority")

    func toggle(_ on: Bool) -> Event {
      .item(
        .update(
          id: item.id,
          input: UpdateItem(
            title: item.title, kind: item.itemType,
            composer: nil, key: nil, modality: nil, tempo: nil, notes: nil,
            tags: nil, priority: on)))
    }

    _ = try bridge.update(toggle(true))
    let on = try bridge.view().items.first
    XCTAssertEqual(on?.priority, true, "star should flip priority on")
    XCTAssertEqual(on?.subtitle, "Chopin", "a priority-only update must not clobber other fields")

    _ = try bridge.update(toggle(false))
    XCTAssertEqual(try bridge.view().items.first?.priority, false, "star should flip priority off")
  }

  /// A related exercise sends its mode and tempo as parts, not one formatted
  /// string (#1939), so a positional slip would read one field as another.
  func testRealBridgeLinkedExerciseCarriesModalityAndTempoParts() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de Lune", kind: .piece, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Db Major Scale", kind: .exercise, composer: nil, key: "Db",
            modality: .major, tempo: Tempo(marking: "Andante", bpm: 72), notes: nil, tags: [],
            photoId: nil, variantLabels: []))))
    let items = try bridge.view().items
    let pieceId = try XCTUnwrap(items.first { $0.title == "Clair de Lune" }?.id)
    let exerciseId = try XCTUnwrap(items.first { $0.title == "Db Major Scale" }?.id)

    _ = try bridge.update(.item(.linkExercise(pieceId: pieceId, exerciseId: exerciseId)))

    let view = try bridge.view()
    XCTAssertNil(view.error, "the link must decode cleanly (err=\(view.error ?? "nil"))")
    let piece = try XCTUnwrap(view.items.first { $0.id == pieceId })
    let linked = try XCTUnwrap(piece.linkedExercises.first)
    XCTAssertEqual(linked.id, exerciseId)
    XCTAssertEqual(linked.key, "Db")
    XCTAssertEqual(linked.modality, .major)
    XCTAssertEqual(linked.tempoMarking, "Andante")
    XCTAssertEqual(linked.tempoBpm, 72)
    XCTAssertNil(linked.pieceContextScore)
  }

  /// A key saved before the wheel existed ("F# major", no modality) still
  /// lights its wedge (#2074); a slip in the trailing field reads as no key.
  func testRealBridgeLegacyKeyLightsItsWedge() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Scales", kind: .exercise, composer: nil, key: "F# major", modality: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))

    let view = try bridge.view()
    XCTAssertNil(view.error, "err=\(view.error ?? "nil")")
    let item = try XCTUnwrap(view.items.first)
    XCTAssertEqual(
      item.keySelection, KeyWheelSelection(ring: 6, modality: .major, spelling: "F#"))
  }

  /// An edit that names only the title and clears the composer leaves every
  /// other field alone (#1953): the update's absent fields must cross the wire
  /// as absent, not as a clear.
  func testRealBridgeTitleOnlyEditClearsTheComposerAndKeepsTheRest() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Nocturne", kind: .piece, composer: "Chopin", key: "E\u{266D}",
            modality: .major, tempo: Tempo(marking: "Andante", bpm: 92), notes: "Slowly",
            tags: ["romantic"], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(
      .item(
        .update(
          id: id,
          input: UpdateItem(
            title: "Nocturne Op. 9", kind: nil, composer: .some(nil), key: nil, modality: nil,
            tempo: nil, notes: nil, tags: nil, priority: nil))))

    let view = try bridge.view()
    XCTAssertNil(view.error, "err=\(view.error ?? "nil")")
    let item = try XCTUnwrap(view.items.first { $0.id == id })
    XCTAssertEqual(item.title, "Nocturne Op. 9")
    XCTAssertEqual(item.subtitle, "", "the composer was cleared")
    XCTAssertEqual(item.itemType, .piece)
    XCTAssertEqual(item.key, "E\u{266D}")
    XCTAssertEqual(item.modality, .major)
    XCTAssertEqual(item.tempoMarking, "Andante")
    XCTAssertEqual(item.tempoBpm, 92)
    XCTAssertEqual(item.notes, "Slowly")
    XCTAssertEqual(item.tags, ["romantic"])
  }

  // ── Real bridge (full-field contract, #846 class) ──────────────────────

  /// Real-bridge full-field round trip for `LibraryItemView` (#846): every
  /// real-bridge test above drives one setter and spot-checks the field it
  /// touches, so a wire break on a DIFFERENT field of this 24-field
  /// projection would pass every one of them. This pins every field the
  /// create/patch pair populates, plus the empty/nil shape of the rest, so a
  /// positional bincode break (which shifts later fields into garbage, the
  /// dangerous #846 case) fails here even when its own dedicated test stays
  /// green. The one case this cannot see is a trailing field dropping to its
  /// own default; that field's dedicated setter test above covers it.
  func testRealBridgeItemCreateAndPatchPreservesEveryField() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    let photoId = Ulid.generate()

    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Nocturne in E-flat", kind: .piece, composer: "Chopin", key: "E\u{266D}",
            modality: .major, tempo: Tempo(marking: "Andante", bpm: 92),
            notes: "Practise slowly, hands separately", tags: ["romantic", "chopin"],
            photoId: photoId, variantLabels: []))))

    let created = try XCTUnwrap(try bridge.view().items.first)
    XCTAssertFalse(created.id.isEmpty, "the core must mint an id")
    XCTAssertEqual(created.itemType, .piece)
    XCTAssertEqual(created.title, "Nocturne in E-flat")
    XCTAssertEqual(created.subtitle, "Chopin", "subtitle mirrors the composer")
    XCTAssertEqual(created.key, "E\u{266D}")
    XCTAssertEqual(created.modality, .major)
    XCTAssertEqual(created.tempoMarking, "Andante")
    XCTAssertEqual(created.tempoBpm, 92)
    XCTAssertEqual(created.notes, "Practise slowly, hands separately")
    XCTAssertEqual(created.tags, ["romantic", "chopin"])
    XCTAssertNotNil(
      SessionClock.parseRFC3339(created.createdAt), "createdAt must be a real RFC3339 stamp")
    XCTAssertNotNil(
      SessionClock.parseRFC3339(created.updatedAt), "updatedAt must be a real RFC3339 stamp")
    XCTAssertEqual(created.priority, false)
    XCTAssertTrue(created.linkedExercises.isEmpty)
    XCTAssertTrue(created.usedIn.isEmpty)
    XCTAssertNil(created.scaffoldPreview)
    XCTAssertNil(created.chordChart)
    XCTAssertNil(created.metre)
    XCTAssertTrue(created.variants.isEmpty)
    XCTAssertEqual(created.ladderIsKeys, false)
    XCTAssertEqual(created.photoId, photoId)
    XCTAssertNil(created.practice)

    // Every PATCH field flipped or cleared in one update (mirrors the shell's
    // ItemFormModel.updateInput()). A field the wire drops silently would
    // leave the OLD value here rather than throwing, so every one is checked.
    _ = try bridge.update(
      .item(
        .update(
          id: created.id,
          input: UpdateItem(
            title: "Nocturne in E-flat (revised)", kind: .piece,
            composer: .some("Chopin (ed. Cortot)"), key: .some("D"), modality: .some(nil),
            tempo: .some(nil), notes: .some(nil), tags: ["romantic", "edited"], priority: true))))

    let view = try bridge.view()
    XCTAssertNil(view.error, "the full patch must decode cleanly (#846)")
    let patched = try XCTUnwrap(view.items.first { $0.id == created.id })
    XCTAssertEqual(patched.title, "Nocturne in E-flat (revised)")
    XCTAssertEqual(patched.itemType, .piece, "kind unchanged, so it must still read piece")
    XCTAssertEqual(patched.subtitle, "Chopin (ed. Cortot)")
    XCTAssertEqual(patched.key, "D")
    XCTAssertNil(patched.modality, "modality was cleared")
    XCTAssertNil(patched.tempoMarking, "tempo was cleared")
    XCTAssertNil(patched.tempoBpm, "tempo was cleared")
    XCTAssertNil(patched.notes, "notes were cleared")
    XCTAssertEqual(patched.tags, ["romantic", "edited"])
    XCTAssertEqual(patched.priority, true)
    XCTAssertEqual(
      patched.photoId, photoId,
      "a patch that never mentions the photo must preserve it: guards the update reconciliation, not just the wire"
    )
  }

  /// Real-bridge full-field round trip for the item's nested shapes (#846,
  /// #1499, #1106): `ChordChart`'s chord-by-chord structure, the derived
  /// `ScaffoldPreviewView`, the piece's `Metre`, and the exercises
  /// `CommitScaffold` links are four different bincode shapes layered on one
  /// item. Existing real-bridge tests spot-check one field of each; this pins
  /// every field so a wire break in any of them can't hide behind the others
  /// looking fine.
  func testRealBridgeItemNestedShapesPreserveEveryField() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma", key: "G",
            modality: .minor, tempo: nil, notes: nil, tags: [], photoId: nil, variantLabels: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)
    let metre = Metre(beats: 3, unit: 4, groups: [3])

    _ = try bridge.update(.item(.setMetre(id: id, metre: metre)))
    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 F7 | Bbmaj7 Ebmaj7 | Am7b5 D7 | Gm6 |")))
    _ = try bridge.update(.item(.commitScaffold(pieceId: id, kinds: [.shells, .guideToneLines])))

    let view = try bridge.view()
    XCTAssertNil(view.error, "every nested write must decode cleanly (err=\(view.error ?? "nil"))")
    let piece = try XCTUnwrap(view.items.first { $0.id == id })

    XCTAssertEqual(piece.metre, metre)

    let chart = try XCTUnwrap(piece.chordChart)
    XCTAssertEqual(chart.key, "G")
    XCTAssertEqual(chart.modality, .minor)
    XCTAssertEqual(chart.sections.count, 1, "one line, no section labels")
    let bars = chart.sections[0].bars
    XCTAssertEqual(bars.count, 4)
    func chord(_ bar: Int, _ index: Int) -> ChordSymbol { bars[bar].chords[index].symbol }
    XCTAssertEqual(bars[0].chords.count, 2)
    XCTAssertEqual(
      chord(0, 0), ChordSymbol(root: 0, quality: .min7, extensions: [], bass: nil, raw: "Cm7"))
    XCTAssertEqual(
      chord(0, 1), ChordSymbol(root: 5, quality: .dom7, extensions: [], bass: nil, raw: "F7"))
    XCTAssertEqual(
      chord(1, 0), ChordSymbol(root: 10, quality: .maj7, extensions: [], bass: nil, raw: "Bbmaj7"))
    XCTAssertEqual(
      chord(1, 1), ChordSymbol(root: 3, quality: .maj7, extensions: [], bass: nil, raw: "Ebmaj7"))
    XCTAssertEqual(
      chord(2, 0), ChordSymbol(root: 9, quality: .min7b5, extensions: [], bass: nil, raw: "Am7b5"))
    XCTAssertEqual(
      chord(2, 1), ChordSymbol(root: 2, quality: .dom7, extensions: [], bass: nil, raw: "D7"))
    XCTAssertEqual(
      chord(3, 0), ChordSymbol(root: 7, quality: .min6, extensions: [], bass: nil, raw: "Gm6"))

    let preview = try XCTUnwrap(piece.scaffoldPreview)
    XCTAssertEqual(preview.key, "G")
    XCTAssertEqual(preview.specs.count, 5, "five generators")
    XCTAssertEqual(preview.fallbackTotal, 0, "every chord above is in the known vocabulary")

    XCTAssertEqual(piece.linkedExercises.count, 2)
    let byTitle = Dictionary(uniqueKeysWithValues: piece.linkedExercises.map { ($0.title, $0) })
    for title in ["Shells", "Guide-tone lines"] {
      let exercise = try XCTUnwrap(byTitle[title])
      XCTAssertFalse(exercise.id.isEmpty)
      XCTAssertEqual(
        exercise.key, "G", "a scaffold-derived exercise is generated in the piece's key")
      XCTAssertNil(exercise.modality)
      XCTAssertNil(exercise.tempoMarking)
      XCTAssertNil(exercise.tempoBpm)
      XCTAssertNil(exercise.practice, "never practised yet")
      XCTAssertNil(exercise.pieceContextScore, "never scored against this piece yet")
    }
  }
}
