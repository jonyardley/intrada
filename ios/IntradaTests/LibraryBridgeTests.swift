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
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Original", kind: .piece, composer: "Bach", key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))

    let afterAdd = try bridge.rendered()
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
            tempo: TempoInput(marking: nil, bpm: nil), notes: .some(nil),
            tags: nil, priority: nil))))

    let afterEdit = try bridge.rendered()
    XCTAssertEqual(
      afterEdit.items.first?.title, "Renamed",
      "edited title should apply (err=\(afterEdit.error ?? "nil"))")
    XCTAssertEqual(afterEdit.items.first?.itemType, .exercise, "edited type should apply")
  }

  func testRealBridgeWeekStripCrossesTheWireWithToday() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)

    let weeks = try bridge.rendered().practiceWeeks
    XCTAssertFalse(weeks.isEmpty, "the week strip should arrive on start")
    XCTAssertTrue(weeks.flatMap(\.days).contains { $0.isToday }, "one day should be today")
  }

  /// An item's keys cross the wire as values (#846, #2106): the spelling
  /// survives, and the core names each one.
  func testRealBridgeAnItemsKeysCrossTheWire() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Shells", kind: .exercise, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let id = try XCTUnwrap(try bridge.rendered().items.first?.id)
    let keys = [
      Key(letter: .e, accidental: .flat, mode: .major),
      Key(letter: .c, accidental: .sharp, mode: .minor),
    ]

    _ = try bridge.update(.item(.updateKeys(id: id, keys: keys)))

    let view = try bridge.rendered()
    let item = try XCTUnwrap(view.items.first { $0.id == id })
    let views: [ItemKeyView] = item.keys
    XCTAssertEqual(views.map(\.key), keys, "(err=\(view.error ?? "nil"))")
    XCTAssertEqual(item.keys.map(\.label), ["E\u{266D} major", "C\u{266F} minor"])
  }

  /// Moved from `VariationManagementUITests` (#1825): a variation taken off
  /// an item drops from its list but stays in the library (#2246).
  func testRealBridgeRemovingAVariationKeepsItInTheLibrary() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Major Scales", kind: .exercise, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil,
            variationLabels: ["Slow", "Swung", "Staccato"]))))
    let item = try XCTUnwrap(try bridge.rendered().items.first)
    let kept = item.variations.filter { $0.label != "Swung" }.map(\.id)

    _ = try bridge.update(
      .item(.updateItemVariations(id: item.id, variationIds: kept, newLabels: [])))

    let view = try bridge.rendered()
    XCTAssertEqual(
      view.items.first { $0.id == item.id }?.variations.map(\.label), ["Slow", "Staccato"],
      "Swung is gone, the others keep their order")
    XCTAssertTrue(view.variations.contains { $0.label == "Swung" }, "and still offered")
  }

  /// The library's variations load through the bincode wire on every launch
  /// after the first (#846, #2246): a tombstone stays hidden, a live row is
  /// offered, and a loaded library is not seeded again.
  func testRealBridgeLoadedVariationsReachTheView() throws {
    let bridge = RowsBridge()
    let load = try XCTUnwrap(
      try bridge.update(.startApp).first {
        if case .persistence(.loadVariations) = $0.effect { return true } else { return false }
      })
    let rows: [Variation] = [
      Variation(
        id: "v-swung", label: "Swung", updatedAt: "2026-10-01T09:00:00Z", deletedAt: nil),
      Variation(
        id: "v-gone", label: "Gone", updatedAt: "2026-10-01T09:00:00Z",
        deletedAt: "2026-10-02T09:00:00Z"),
    ]

    let after = try bridge.resolve(load.id, persistenceOutput: .variations(rows))

    let view = try bridge.rendered()
    XCTAssertNil(view.error)
    XCTAssertEqual(view.variations.map(\.id), ["v-swung"])
    XCTAssertEqual(view.variations.map(\.label), ["Swung"])
    XCTAssertFalse(
      after.contains {
        if case .persistence(.saveVariations) = $0.effect { return true } else { return false }
      }, "a library with rows is not seeded")
  }

  /// `VariationEvent` crosses the live bridge (#846, #2246): a rename keeps
  /// the id, so the marks stay; a delete takes it out of every list.
  func testRealBridgeRenamingAndDeletingAVariation() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    let requests = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Major Scales", kind: .exercise, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: ["Slow", "Swung"]))))
    let minted: [Variation] = requests.flatMap { request -> [Variation] in
      guard case .persistence(.saveVariations(let rows)) = request.effect else { return [] }
      return rows
    }
    XCTAssertEqual(minted.map(\.label), ["Slow", "Swung"], "the new rows are written")
    let item = try XCTUnwrap(try bridge.rendered().items.first)
    let rows: [VariationView] = item.variations
    let slowId = try XCTUnwrap(rows.first { $0.label == "Slow" }?.id)
    let swungId = try XCTUnwrap(rows.first { $0.label == "Swung" }?.id)

    let rename: VariationEvent = .rename(id: slowId, label: "Very slow")
    _ = try bridge.update(.variation(rename))
    let renamed = try XCTUnwrap(try bridge.rendered().items.first { $0.id == item.id })
    XCTAssertEqual(renamed.variations.first?.id, slowId, "renamed in place, marks intact")
    XCTAssertEqual(renamed.variations.first?.label, "Very slow")

    _ = try bridge.update(.variation(.delete(id: swungId)))
    let view = try bridge.rendered()
    XCTAssertNil(view.error)
    XCTAssertEqual(view.items.first { $0.id == item.id }?.variations.map(\.id), [slowId])
    let offered: [VariationOptionView] = view.variations
    XCTAssertFalse(offered.contains { $0.id == swungId })
  }

  /// `UpdateSections`, the new `Item` field and `SectionView` cross the real
  /// bincode bridge (#846, #2245).
  func testRealBridgeUpdateSectionsLabelsTypedAndPickedBars() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Rondo", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let item = try XCTUnwrap(try bridge.rendered().items.first)
    let typed = BarsInput.typed("1\u{2013}16")
    let spot = SectionKind.troubleSpot

    let requests = try bridge.update(
      .item(
        .updateSections(
          id: item.id,
          sections: [
            SectionEdit(id: nil, name: "A1", bars: typed, kind: .form, targetBpm: ""),
            SectionEdit(
              id: nil, name: "", bars: .picked(first: 12, last: 14), kind: spot, targetBpm: "72"),
          ])))

    let saved: [ItemSection] = requests.compactMap { request -> Item? in
      guard case .persistence(.saveItem(let item)) = request.effect else { return nil }
      return item
    }.flatMap(\.sections)
    XCTAssertEqual(
      saved.map(\.bars), [BarRange(first: 1, last: 16), BarRange(first: 12, last: 14)])
    XCTAssertEqual(saved.map(\.targetBpm), [nil, 72])

    let after = try XCTUnwrap(try bridge.rendered().items.first { $0.id == item.id })
    let views: [SectionView] = after.sections
    XCTAssertEqual(views.map(\.label), ["A1", "Bars 12 to 14"])
    XCTAssertEqual(views.map(\.barsCaption), ["Bars 1 to 16", nil])
    XCTAssertEqual(views.last?.kind, spot)
  }

  /// A refused bar range points the form at the sections (#2245).
  func testRealBridgeRefusedBarsPointAtTheSections() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Rondo", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let item = try XCTUnwrap(try bridge.rendered().items.first)

    _ = try bridge.update(
      .item(
        .updateSections(
          id: item.id,
          sections: [
            SectionEdit(id: nil, name: "A", bars: .typed("1-8"), kind: .form, targetBpm: "")
          ])))
    let before = try bridge.rendered().items.first?.sections

    _ = try bridge.update(
      .item(
        .updateSections(
          id: item.id,
          sections: [
            SectionEdit(id: nil, name: "B", bars: .typed("16-1"), kind: .form, targetBpm: "")
          ])))

    let view = try bridge.rendered()
    XCTAssertNotNil(view.error)
    XCTAssertEqual(view.errorTarget, .piece(field: .sections))
    XCTAssertEqual(view.items.first?.sections, before, "nothing written")
    XCTAssertEqual(before?.map(\.label), ["A"])
  }

  /// The `FormErrorField` case decodes on the wire (#846, #1831): a refused
  /// variation points the form at the Variations section.
  func testRealBridgeRefusedVariationPointsAtTheSection() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Major Scales", kind: .exercise, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil,
            variationLabels: [String(repeating: "x", count: 101)]))))

    let view = try bridge.rendered()
    XCTAssertNotNil(view.error)
    XCTAssertEqual(view.errorTarget, .piece(field: .variations))
    XCTAssertTrue(view.items.isEmpty, "nothing written")
  }

  /// A stub bridge cannot see a skewed `TempoInput` decode (#846, #2224).
  func testRealBridgeReadsTheTypedBpm() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    let add = { (title: String, bpm: String) in
      try bridge.update(
        .item(
          .add(
            CreateItem(
              title: title, kind: .piece, composer: nil, key: nil,
              tempo: TempoInput(marking: nil, bpm: bpm), notes: nil, tags: [], photoId: nil,
              variationLabels: []))))
    }

    let saved = try add("Nocturne", " 92 ").compactMap { request -> Item? in
      guard case .persistence(.saveItem(let item)) = request.effect else { return nil }
      return item
    }
    XCTAssertEqual(saved.map(\.tempo), [Tempo(marking: nil, bpm: 92)])
    XCTAssertNil(try bridge.rendered().error)

    _ = try add("Etude", "12a")
    let refused = try bridge.rendered()
    XCTAssertEqual(refused.error, "BPM must be a whole number between 1 and 400")
    XCTAssertEqual(refused.errorTarget, .piece(field: .tempo))
    XCTAssertNil(refused.items.first { $0.title == "Etude" }, "nothing written")
  }

  /// The draft crosses as Swift's own bincode (#2229): a Rust round trip cannot
  /// see Swift and Rust disagreeing about `PhotoDraft`'s bytes.
  func testFillFormFromReadDecodesTheDraftSwiftSerialised() throws {
    let fields: [FormFieldNow] = [
      FormFieldNow(field: .title, text: "The name I gave it", holdsRead: false),
      FormFieldNow(field: .composer, text: "", holdsRead: false),
      FormFieldNow(field: .marking, text: "", holdsRead: false),
      FormFieldNow(field: .bpm, text: "", holdsRead: false),
    ]

    let fills = try fillFormFromRead(
      draft: Data(PhotoDraft.readPage.bincodeSerialize()), fields: fields)

    XCTAssertEqual(
      fills,
      [
        FormFieldFill(field: .composer, value: "Joseph Kosmo", weak: true),
        FormFieldFill(field: .marking, value: "Moderato", weak: false),
        FormFieldFill(field: .bpm, value: "120", weak: false),
      ])
  }

  /// The Edit form's two events against the real core (#1783, #2246): the
  /// written key and the item's variations are set side by side, and neither
  /// takes the other's place.
  func testRealBridgeEditFormSetsTheKeyAndTheVariationsApart() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Arpeggios", kind: .exercise, composer: nil,
            key: Key(letter: .g, accidental: .natural, mode: .major),
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let keyed = try XCTUnwrap(try bridge.rendered().items.first)

    let editing = ItemFormModel(item: keyed)
    editing.key = Key(letter: .d, accidental: .natural, mode: .major)
    editing.variations = ["Slow", "Swung"].map { VariationRow(label: $0) }
    for event in editing.editEvents(id: keyed.id) {
      _ = try bridge.update(.item(event))
      XCTAssertNil(try bridge.rendered().error)
    }

    let edited = try XCTUnwrap(try bridge.rendered().items.first { $0.id == keyed.id })
    XCTAssertEqual(edited.variations.map(\.label), ["Slow", "Swung"])
    XCTAssertEqual(edited.key, Key(letter: .d, accidental: .natural, mode: .major))
  }

  /// Real-bridge wire pin for the photo id (#846, #1355): the Swift serializer,
  /// the Rust deserializer and the `ViewModel` projection must all agree on the
  /// new `Item` field and the two new `ItemEvent` variants. A stub bridge
  /// cannot catch a break in any of the three.
  func testRealBridgePhotoIdCrossesTheWireBothWays() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Nocturne", kind: .piece, composer: "Chopin", key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let id = try XCTUnwrap(try bridge.rendered().items.first?.id)

    _ = try bridge.update(.item(.setPhoto(id: id, photoId: "01ARZ3NDEKTSV4RRFFQ69G5FAV")))

    let afterSet = try bridge.rendered()
    XCTAssertNil(afterSet.error, "setPhoto should cross the wire cleanly")
    XCTAssertEqual(
      afterSet.items.first { $0.id == id }?.photoId, "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "the id the screens read comes back through the projection")

    _ = try bridge.update(.item(.clearPhoto(id: id)))

    XCTAssertNil(
      try bridge.rendered().items.first { $0.id == id }?.photoId,
      "an absent Option must decode as absent, not as the previous value")
  }

  /// Every recognition type is new on the bincode wire, and `f32` is a new
  /// scalar on it. A stub bridge cannot catch a wire break (#846), so this
  /// drives the whole round trip: the effect out, a `RecognitionOutput` built
  /// in Swift back in, and the draft the form will read out of the projection.
  func testRealBridgeReadPhotoFillsTheDraft() throws {
    let bridge = RowsBridge()
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

    let recognition = try bridge.rendered().photoRecognition
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
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    let photoId = Ulid.generate()

    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Cry Me A River", kind: .piece, composer: "Arthur Hamilton", key: nil,
            tempo: nil, notes: nil, tags: [], photoId: photoId, variationLabels: []))))

    let view = try bridge.rendered()
    XCTAssertNil(view.error)
    XCTAssertEqual(
      view.items.first?.photoId, photoId,
      "the page the form was read off has to reach the piece it created")
  }

  /// The shell mints a photo's id (offline-first invariant 3) and the core
  /// refuses any id that is not a ulid, so `Ulid` and Rust's parser have to
  /// agree.
  func testRealBridgeAcceptsAUlidTheShellMinted() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Gymnopedie", kind: .piece, composer: "Satie", key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let id = try XCTUnwrap(try bridge.rendered().items.first?.id)
    let minted = Ulid.generate()

    _ = try bridge.update(.item(.setPhoto(id: id, photoId: minted)))

    let after = try bridge.rendered()
    XCTAssertNil(after.error, "the core's validate_photo_id must accept what Ulid mints")
    XCTAssertEqual(after.items.first { $0.id == id }?.photoId, minted)
  }

  /// Real-bridge round trip for the item's metre (#1499): `SetMetre` carries an
  /// optional nested struct with an optional list inside it, the #846 shape.
  func testRealBridgeSetsAndClearsTheItemsMetre() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Take Five", kind: .piece, composer: "Desmond", key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let id = try XCTUnwrap(try bridge.rendered().items.first?.id)
    let metre = Metre(beats: 5, unit: 4, groups: [3, 2])

    _ = try bridge.update(.item(.setMetre(id: id, metre: metre)))
    XCTAssertEqual(try bridge.rendered().items.first?.metre, metre)
    XCTAssertNil(try bridge.rendered().error)

    _ = try bridge.update(.item(.setMetre(id: id, metre: nil)))
    XCTAssertNil(try bridge.rendered().items.first?.metre)
  }

  /// but returns a nested `ChordChart` + `ScaffoldPreviewView` across the bincode
  /// wire, a shape a stub bridge can't exercise. A bad chart must surface an
  /// error and never store a partial.
  func testRealBridgeSetChordChartDerivesScaffoldPreview() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma",
            key: Key(letter: .g, accidental: .natural, mode: .minor), tempo: nil, notes: nil,
            tags: [], photoId: nil, variationLabels: []))))
    let id = try XCTUnwrap(try bridge.rendered().items.first?.id)

    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 | F7 | Bbmaj7 |")))
    let ok = try bridge.rendered()
    let piece = try XCTUnwrap(ok.items.first { $0.id == id })
    let preview = try XCTUnwrap(
      piece.scaffoldPreview, "charted piece derives a preview (err=\(ok.error ?? "nil"))")
    XCTAssertEqual(preview.key, "G")
    XCTAssertEqual(preview.specs.count, 5, "five generators")
    XCTAssertEqual(piece.chordChart?.sections.first?.bars.count, 3, "three bars round-trip")

    // A bad token surfaces an error and leaves the prior chart intact.
    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 | Hxyz |")))
    let bad = try bridge.rendered()
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
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma",
            key: Key(letter: .g, accidental: .natural, mode: .minor), tempo: nil, notes: nil,
            tags: [], photoId: nil, variationLabels: []))))
    let id = try XCTUnwrap(try bridge.rendered().items.first?.id)
    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 | F7 | Bbmaj7 |")))

    _ = try bridge.update(
      .item(.commitScaffold(pieceId: id, kinds: [.shells, .guideToneLines])))

    let after = try bridge.rendered()
    XCTAssertNil(after.error, "commit surfaces no error (err=\(after.error ?? "nil"))")
    let piece = try XCTUnwrap(after.items.first { $0.id == id })
    XCTAssertEqual(piece.linkedExercises.count, 2, "two selected kinds become linked exercises")
    let titles = Swift.Set(piece.linkedExercises.map(\.title))
    XCTAssertTrue(titles.contains("Shells") && titles.contains("Guide-tone lines"))

    // Re-committing the same kinds dedups: no duplicate exercises.
    _ = try bridge.update(.item(.commitScaffold(pieceId: id, kinds: [.shells])))
    let reran = try bridge.rendered()
    let shells = reran.items.filter { $0.title == "Shells" }
    XCTAssertEqual(shells.count, 1, "re-commit adds no duplicate (#1106 dedup)")
  }

  /// Real-bridge one-pass create (#1390): `AddPieceInFull` carries a nested
  /// `CreateItem`, an optional raw chart and a `Vec<ScaffoldEntry>` whose two
  /// variants have different payload shapes, so a stub bridge cannot prove the
  /// wire holds (#846). Pinned here before any screen sends it.
  func testRealBridgeAddPieceInFullCarriesChartAndExercises() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Shell voicings", kind: .exercise, composer: nil,
            key: Key(letter: .g, accidental: .natural, mode: nil), tempo: nil, notes: nil, tags: [],
            photoId: nil, variationLabels: []))))
    let existingId = try XCTUnwrap(try bridge.rendered().items.first?.id)

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma",
            key: Key(letter: .g, accidental: .natural, mode: .minor), tempo: nil, notes: nil,
            tags: [], photoId: nil, variationLabels: []),
          chart: "| Cm7 | F7 | Bbmaj7 |",
          exercises: [
            .existing(id: existingId),
            .new(
              CreateItem(
                title: "Enclosures", kind: .exercise, composer: nil, key: nil,
                tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: [])),
          ])))

    let after = try bridge.rendered()
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
            title: "Blue in Green", kind: .piece, composer: nil,
            key: Key(letter: .g, accidental: .natural, mode: nil),
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []),
          chart: "| Cm7 | Hxyz |",
          exercises: [
            .new(
              CreateItem(
                title: "Orphan", kind: .exercise, composer: nil, key: nil,
                tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))
          ])))

    let rejected = try bridge.rendered()
    XCTAssertNotNil(rejected.error, "a parse error must surface, not vanish (#846)")
    XCTAssertNil(
      rejected.items.first { $0.title == "Blue in Green" }, "no half-made piece is left behind")
    XCTAssertNil(rejected.items.first { $0.title == "Orphan" }, "and no orphan exercise either")
  }

  /// Real-bridge error target (#1595): three variants with different payload
  /// shapes, one of them a nested optional enum, so only the live bridge proves
  /// the wire holds (#846). The screens read this to point at the failure.
  func testRealBridgeRejectedCreateCarriesWhereItFailed() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Blue in Green", kind: .piece, composer: "Bill Evans",
            key: Key(letter: .g, accidental: .natural, mode: nil), tempo: nil, notes: nil, tags: [],
            photoId: nil, variationLabels: []),
          chart: "| Cm7 | Hxyz |",
          exercises: [])))

    XCTAssertEqual(
      try bridge.rendered().errorTarget, .chartBar(barNumber: 2, token: "Hxyz"),
      "the bar and the token, so the chart section can highlight in place")

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Blue in Green", kind: .piece, composer: "Bill Evans",
            key: Key(letter: .g, accidental: .natural, mode: nil), tempo: nil, notes: nil, tags: [],
            photoId: nil, variationLabels: []),
          chart: "swing feel",
          exercises: [])))

    XCTAssertEqual(
      try bridge.rendered().errorTarget, .chart,
      "prose with no bars fails at no bar, so the whole section is what is marked")

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Blue in Green", kind: .piece, composer: "Bill Evans",
            key: Key(letter: .g, accidental: .natural, mode: nil), tempo: nil, notes: nil, tags: [],
            photoId: nil, variationLabels: []),
          chart: nil,
          exercises: [
            .new(
              CreateItem(
                title: "Enclosures", kind: .exercise, composer: nil, key: nil,
                tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: [])),
            .new(
              CreateItem(
                title: "   ", kind: .exercise, composer: nil, key: nil,
                tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: [])),
          ])))

    XCTAssertEqual(
      try bridge.rendered().errorTarget, .exercise(index: 1, field: .title),
      "the second row is the blank one, and its title is what to mark")

    _ = try bridge.update(
      .item(
        .addPieceInFull(
          piece: CreateItem(
            title: "Blue in Green", kind: .piece, composer: String(repeating: "x", count: 201),
            key: Key(letter: .g, accidental: .natural, mode: nil), tempo: nil, notes: nil, tags: [],
            photoId: nil,
            variationLabels: []),
          chart: nil,
          exercises: [])))

    XCTAssertEqual(try bridge.rendered().errorTarget, .piece(field: .composer))

    _ = try bridge.update(.item(.updateKeys(id: "no-such-exercise", keys: [])))

    let unrelated = try bridge.rendered()
    XCTAssertNotNil(unrelated.error, "a failure with no field still reports what went wrong")
    XCTAssertNil(
      unrelated.errorTarget, "but must not leave the form pointing at the last failure")
  }

  /// Real-bridge priority toggle (#763): the star sends an UpdateItem with every
  /// optional field "no change" (outer nil) and only `priority` set, a different
  /// bincode shape than the full edit, so round-trip it through the live bridge to
  /// catch an absent-vs-present wire break (#846).
  func testRealBridgePriorityToggleAppliesToViewModel() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Etude", kind: .piece, composer: "Chopin", key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    let item = try XCTUnwrap(try bridge.rendered().items.first)
    XCTAssertFalse(item.priority, "new items start non-priority")

    func toggle(_ on: Bool) -> Event {
      .item(
        .update(
          id: item.id,
          input: UpdateItem(
            title: item.title, kind: item.itemType,
            composer: nil, key: nil, tempo: nil, notes: nil,
            tags: nil, priority: on)))
    }

    _ = try bridge.update(toggle(true))
    let on = try bridge.rendered().items.first
    XCTAssertEqual(on?.priority, true, "star should flip priority on")
    XCTAssertEqual(on?.subtitle, "Chopin", "a priority-only update must not clobber other fields")

    _ = try bridge.update(toggle(false))
    XCTAssertEqual(
      try bridge.rendered().items.first?.priority, false, "star should flip priority off")
  }

  /// A related exercise sends its mode and tempo as parts, not one formatted
  /// string (#1939), so a positional slip would read one field as another.
  func testRealBridgeLinkedExerciseCarriesModalityAndTempoParts() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Clair de Lune", kind: .piece, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Db Major Scale", kind: .exercise, composer: nil,
            key: Key(letter: .d, accidental: .flat, mode: .major),
            tempo: TempoInput(marking: "Andante", bpm: "72"), notes: nil,
            tags: [],
            photoId: nil, variationLabels: []))))
    let items = try bridge.rendered().items
    let pieceId = try XCTUnwrap(items.first { $0.title == "Clair de Lune" }?.id)
    let exerciseId = try XCTUnwrap(items.first { $0.title == "Db Major Scale" }?.id)

    try bridge.link(exercise: exerciseId, to: pieceId)

    let view = try bridge.rendered()
    XCTAssertNil(view.error, "the link must decode cleanly (err=\(view.error ?? "nil"))")
    let piece = try XCTUnwrap(view.items.first { $0.id == pieceId })
    let linked = try XCTUnwrap(piece.linkedExercises.first)
    XCTAssertEqual(linked.id, exerciseId)
    XCTAssertEqual(linked.key, Key(letter: .d, accidental: .flat, mode: .major))
    XCTAssertEqual(linked.keyLabel, "D\u{266D} major")
    XCTAssertEqual(linked.tempoMarking, "Andante")
    XCTAssertEqual(linked.tempoBpm, 72)
    XCTAssertNil(linked.pieceContextScore)
  }

  /// A key lights its wedge (#2074); a slip in the trailing field reads as no key.
  func testRealBridgeAKeyLightsItsWedge() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Scales", kind: .exercise, composer: nil,
            key: Key(letter: .f, accidental: .sharp, mode: .major),
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))

    let view = try bridge.rendered()
    XCTAssertNil(view.error, "err=\(view.error ?? "nil")")
    let item = try XCTUnwrap(view.items.first)
    XCTAssertEqual(
      item.keySelection, KeyWheelSelection(ring: 6, modality: .major, spelling: "F#"))
  }

  /// An edit that names only the title and clears the composer leaves every
  /// other field alone (#1953): the update's absent fields must cross the wire
  /// as absent, not as a clear.
  func testRealBridgeTitleOnlyEditClearsTheComposerAndKeepsTheRest() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Nocturne", kind: .piece, composer: "Chopin",
            key: Key(letter: .e, accidental: .flat, mode: .major),
            tempo: TempoInput(marking: "Andante", bpm: "92"), notes: "Slowly",
            tags: ["romantic"], photoId: nil, variationLabels: []))))
    let id = try XCTUnwrap(try bridge.rendered().items.first?.id)

    _ = try bridge.update(
      .item(
        .update(
          id: id,
          input: UpdateItem(
            title: "Nocturne Op. 9", kind: nil, composer: .some(nil), key: nil,
            tempo: nil, notes: nil, tags: nil, priority: nil))))

    let view = try bridge.rendered()
    XCTAssertNil(view.error, "err=\(view.error ?? "nil")")
    let item = try XCTUnwrap(view.items.first { $0.id == id })
    XCTAssertEqual(item.title, "Nocturne Op. 9")
    XCTAssertEqual(item.subtitle, "", "the composer was cleared")
    XCTAssertEqual(item.itemType, .piece)
    XCTAssertEqual(item.key, Key(letter: .e, accidental: .flat, mode: .major))
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
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    let photoId = Ulid.generate()

    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Nocturne in E-flat", kind: .piece, composer: "Chopin",
            key: Key(letter: .e, accidental: .flat, mode: .major),
            tempo: TempoInput(marking: "Andante", bpm: "92"),
            notes: "Practise slowly, hands separately", tags: ["romantic", "chopin"],
            photoId: photoId, variationLabels: []))))

    let created = try XCTUnwrap(try bridge.rendered().items.first)
    XCTAssertFalse(created.id.isEmpty, "the core must mint an id")
    XCTAssertEqual(created.itemType, .piece)
    XCTAssertEqual(created.title, "Nocturne in E-flat")
    XCTAssertEqual(created.subtitle, "Chopin", "subtitle mirrors the composer")
    XCTAssertEqual(created.key, Key(letter: .e, accidental: .flat, mode: .major))
    XCTAssertEqual(created.keyLabel, "E\u{266D} major")
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
    XCTAssertTrue(created.variations.isEmpty)
    XCTAssertTrue(created.keys.isEmpty)
    XCTAssertTrue(created.sections.isEmpty)
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
            composer: .some("Chopin (ed. Cortot)"),
            key: .some(Key(letter: .d, accidental: .natural, mode: nil)),
            tempo: TempoInput(marking: nil, bpm: nil), notes: .some(nil),
            tags: ["romantic", "edited"], priority: true))))

    let view = try bridge.rendered()
    XCTAssertNil(view.error, "the full patch must decode cleanly (#846)")
    let patched = try XCTUnwrap(view.items.first { $0.id == created.id })
    XCTAssertEqual(patched.title, "Nocturne in E-flat (revised)")
    XCTAssertEqual(patched.itemType, .piece, "kind unchanged, so it must still read piece")
    XCTAssertEqual(patched.subtitle, "Chopin (ed. Cortot)")
    XCTAssertEqual(
      patched.key, Key(letter: .d, accidental: .natural, mode: nil), "a key with no mode")
    XCTAssertEqual(patched.keyLabel, "D")
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
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma",
            key: Key(letter: .g, accidental: .natural, mode: .minor), tempo: nil, notes: nil,
            tags: [], photoId: nil, variationLabels: []))))
    let id = try XCTUnwrap(try bridge.rendered().items.first?.id)
    let metre = Metre(beats: 3, unit: 4, groups: [3])

    _ = try bridge.update(.item(.setMetre(id: id, metre: metre)))
    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 F7 | Bbmaj7 Ebmaj7 | Am7b5 D7 | Gm6 |")))
    _ = try bridge.update(.item(.commitScaffold(pieceId: id, kinds: [.shells, .guideToneLines])))

    let view = try bridge.rendered()
    XCTAssertNil(view.error, "every nested write must decode cleanly (err=\(view.error ?? "nil"))")
    let piece = try XCTUnwrap(view.items.first { $0.id == id })

    XCTAssertEqual(piece.metre, metre)

    let chart = try XCTUnwrap(piece.chordChart)
    XCTAssertEqual(chart.key, Key(letter: .g, accidental: .natural, mode: .minor))
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
        exercise.key, Key(letter: .g, accidental: .natural, mode: .minor),
        "a scaffold-derived exercise is generated in the piece's key")
      XCTAssertNil(exercise.tempoMarking)
      XCTAssertNil(exercise.tempoBpm)
      XCTAssertNil(exercise.practice, "never practised yet")
      XCTAssertNil(exercise.pieceContextScore, "never scored against this piece yet")
    }
  }

  // ── Section links (#2248) ──

  private func add(_ bridge: RowsBridge, _ title: String, _ kind: ItemKind) throws -> String {
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: title, kind: kind, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: []))))
    return try XCTUnwrap(try bridge.rendered().items.first { $0.title == title }?.id)
  }

  private func sectionIds(
    _ bridge: RowsBridge, on piece: String, _ names: [String]
  ) throws -> [String: String] {
    _ = try bridge.update(
      .item(
        .updateSections(
          id: piece,
          sections: names.map {
            SectionEdit(id: nil, name: $0, bars: .blank, kind: .form, targetBpm: "")
          })))
    let views = try XCTUnwrap(try bridge.rendered().items.first { $0.id == piece }?.sections)
    return Dictionary(uniqueKeysWithValues: views.map { ($0.name, $0.id) })
  }

  /// Loaded links, a tombstone among them, cross into the core (#846).
  func testRealBridgeLoadedLinksReachThePieceAndItsExercise() throws {
    let bridge = RowsBridge()
    let load = try XCTUnwrap(
      try bridge.update(.startApp).first {
        if case .persistence(.loadItems) = $0.effect { return true } else { return false }
      })
    var piece = LibraryItemFixture.record(id: "p1", title: "Nocturne")
    piece.sections = [
      ItemSection(
        id: "s-a1", name: "A1", bars: nil, kind: .form, targetBpm: nil, position: 0,
        updatedAt: "2026-10-04T09:00:00Z", deletedAt: nil),
      ItemSection(
        id: "s-spot", name: "", bars: BarRange(first: 19, last: 20), kind: .troubleSpot,
        targetBpm: nil, position: 1, updatedAt: "2026-10-04T09:00:00Z", deletedAt: nil),
    ]
    piece.exerciseLinks = [
      ExerciseLink(
        id: "l1", exerciseId: "e1", sectionId: nil, position: 0,
        updatedAt: "2026-10-04T09:00:00Z", deletedAt: nil),
      ExerciseLink(
        id: "l2", exerciseId: "e1", sectionId: "s-a1", position: 1,
        updatedAt: "2026-10-04T09:05:00Z", deletedAt: "2026-10-04T09:05:00Z"),
      ExerciseLink(
        id: "l3", exerciseId: "e1", sectionId: "s-spot", position: 2,
        updatedAt: "2026-10-04T09:05:00Z", deletedAt: nil),
    ]
    let exercise = LibraryItemFixture.record(id: "e1", title: "Thirds", kind: .exercise)

    _ = try bridge.resolve(load.id, persistenceOutput: .items([piece, exercise]))

    let rows = try bridge.rendered().items
    let card = try XCTUnwrap(rows.first { $0.id == "p1" }?.linkedExercises.first)
    XCTAssertEqual(card.id, "e1")
    XCTAssertTrue(card.wholePiece)
    let sections: [LinkedSectionView] = card.sections
    XCTAssertEqual(sections.map(\.id), ["s-spot"], "the tombstone stays hidden")
    XCTAssertEqual(sections.map(\.label), ["Bars 19 to 20"])
    XCTAssertEqual(sections.map(\.labelInText), ["bars 19 to 20"])
    XCTAssertEqual(rows.first { $0.id == "e1" }?.usedIn.map(\.linked), [true])
  }

  /// Links made in the core, and both views of them, cross the bridge (#846).
  func testRealBridgeLinksAnExerciseToSectionsOfTwoPieces() throws {
    let bridge = RowsBridge()
    _ = try bridge.update(.startApp)
    let nocturne = try add(bridge, "Nocturne", .piece)
    let etude = try add(bridge, "Étude", .piece)
    let thirds = try add(bridge, "Thirds", .exercise)
    let nocturneSections = try sectionIds(bridge, on: nocturne, ["A1", "A2"])
    let coda = try XCTUnwrap(try sectionIds(bridge, on: etude, ["Coda"])["Coda"])
    let a1 = try XCTUnwrap(nocturneSections["A1"])
    let a2 = try XCTUnwrap(nocturneSections["A2"])

    let targets: [LinkTarget] = [
      LinkTarget(pieceId: nocturne, sectionId: a2), LinkTarget(pieceId: etude, sectionId: coda),
    ]
    let requests = try bridge.update(
      .item(.setExerciseLinks(exerciseId: thirds, targets: targets)))

    let written: [ExerciseLink] = requests.flatMap { request -> [Item] in
      guard case .persistence(.saveItems(let items)) = request.effect else { return [] }
      return items
    }.flatMap(\.exerciseLinks)
    XCTAssertEqual(Swift.Set(written.map(\.sectionId)), [a2, coda])
    XCTAssertEqual(written.map(\.exerciseId), [thirds, thirds])
    XCTAssertTrue(written.allSatisfy { $0.deletedAt == nil })

    let linked = try bridge.rendered()
    XCTAssertNil(linked.error)
    let card = try XCTUnwrap(linked.items.first { $0.id == nocturne }?.linkedExercises.first)
    let cardSections: [LinkedSectionView] = card.sections
    XCTAssertEqual(card.id, thirds)
    XCTAssertFalse(card.wholePiece)
    XCTAssertEqual(cardSections.map(\.label), ["A2"])
    XCTAssertEqual(cardSections.map(\.labelInText), ["A2"])
    let usedIn = try XCTUnwrap(linked.items.first { $0.id == thirds }?.usedIn)
    XCTAssertEqual(
      Swift.Set(usedIn.flatMap { $0.sections.map(\.label) }), ["A2", "Coda"],
      "the exercise shows both links")
    XCTAssertTrue(usedIn.allSatisfy { $0.linked && !$0.wholePiece })

    let links: [LinkEdit] = [
      LinkEdit(exercise: .existing(id: thirds), sectionId: nil),
      LinkEdit(
        exercise: .new(
          CreateItem(
            title: "Broken octaves", kind: .exercise, composer: nil, key: nil,
            tempo: nil, notes: nil, tags: [], photoId: nil, variationLabels: [])),
        sectionId: a1),
    ]
    _ = try bridge.update(.item(.setPieceLinks(pieceId: nocturne, links: links)))

    let after = try bridge.rendered()
    XCTAssertNil(after.error)
    let rows = try XCTUnwrap(after.items.first { $0.id == nocturne }?.linkedExercises)
    XCTAssertEqual(rows.map(\.title), ["Thirds", "Broken octaves"])
    XCTAssertEqual(rows.map(\.wholePiece), [true, false])
    XCTAssertEqual(rows.map { $0.sections.map(\.label) }, [[], ["A1"]], "A2 left the set")
  }
}
