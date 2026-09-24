import Foundation
import SharedTypes
import Testing

@testable import Intrada

/// The snapshot suite renders hand-built fixtures, never the live core (#1949).
/// Each test states the musician's input, runs it through `LiveBridge`, and
/// holds the fixture to what the core projects from it.
@MainActor
struct PreviewFixtureParityTests {

  private struct KeyAndTempo: Equatable {
    let title: String
    let key: String?
    let modality: Modality?
    let tempoMarking: String?
    let tempoBpm: UInt16?
  }

  private struct ExerciseInput {
    let title: String
    let key: String?
    let modality: Modality?
    let tempoBpm: UInt16?
  }

  private struct VariationRow: Equatable {
    let label: String
    let latestScore: UInt8?
    let isSolid: Bool
    let caption: String
  }

  private struct Ladder: Equatable {
    let key: String?
    let modality: Modality?
    let rows: [VariationRow]
    let solidCount: UInt64
    let isKeys: Bool
    let showsKey: Bool
  }

  private struct PickerRow: Equatable {
    let label: String
    let caption: String
    let isSolid: Bool
  }

  // ── Linked exercises ───────────────────────────────────────────────────

  @Test func aPiecesRelatedExercisesCarryTheKeyAndTempoTheCoreWrites() throws {
    let hanon = ExerciseInput(title: "Hanon No. 1", key: "C", modality: .major, tempoBpm: 108)
    let scale = ExerciseInput(title: "Db Major Scale", key: "Db", modality: .major, tempoBpm: nil)
    let piece = try projectedPiece(
      title: "Clair de Lune", key: "Db", modality: .major, marking: "Andante", bpm: 72,
      linking: [hanon, scale])

    #expect(fields(LibraryItemView.previewPiece) == fields(piece))
    #expect(
      LibraryItemView.previewPiece.linkedExercises.map(fields)
        == piece.linkedExercises.map(fields))
  }

  @Test func theLinkedExercisesSectionCarriesTheKeyAndTempoTheCoreWrites() throws {
    let piece = try projectedPiece(
      title: "Clair de Lune", key: "Db", modality: .major, marking: "Andante", bpm: 72,
      linking: [
        ExerciseInput(title: "Hanon No. 1", key: "C", modality: .major, tempoBpm: 108),
        ExerciseInput(title: "Db Major Scale", key: "Db", modality: .major, tempoBpm: nil),
        ExerciseInput(title: "Arpeggios in Db", key: nil, modality: nil, tempoBpm: nil),
      ])
    let fixture = LibraryItemView.previewDetailWithLinkedExercises

    #expect(fields(fixture) == fields(piece))
    #expect(fixture.linkedExercises.map(fields) == piece.linkedExercises.map(fields))
  }

  // ── Variation captions ─────────────────────────────────────────────────

  @Test func aLadderInThreeKeysCaptionsEachMarkAsTheCoreDoes() throws {
    let projected = try projectedExercise(
      title: LibraryItemView.previewExerciseWithVariations.title, key: "C", modality: .major,
      labels: ["C", "F", "B♭"], scores: ["C": 9, "F": 5])

    #expect(ladder(.previewExerciseWithVariations) == ladder(projected))
  }

  @Test func twelveKeysCaptionEachMarkAsTheCoreDoes() throws {
    let labels = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"]
    let projected = try projectedExercise(
      title: "Chromatic run", key: nil, modality: nil, labels: labels,
      scores: ["C": 9, "C♯": 9, "D": 9, "D♯": 9, "E": 6])

    #expect(ladder(.previewExerciseWithTwelveVariations) == ladder(projected))
  }

  @Test func inversionsNeverPlayedCaptionAsTheCoreDoes() throws {
    let projected = try projectedExercise(
      title: "Triad inversions", key: nil, modality: nil,
      labels: ["Root position", "1st inversion", "2nd inversion"], scores: [:])

    #expect(ladder(.previewExerciseWithNamedVariations) == ladder(projected))
  }

  @Test func longVariationNamesCaptionEachMarkAsTheCoreDoes() throws {
    let projected = try projectedExercise(
      title: "Arpeggios, four octaves", key: nil, modality: nil,
      labels: ["Hands together, two octaves", "Hands separately", "Slow, with the metronome"],
      scores: ["Hands together, two octaves": 8])

    #expect(ladder(.previewExerciseWithLongVariationName) == ladder(projected))
  }

  @Test func thePlayersVariationPickerCaptionsEachRowAsTheCoreDoes() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    let id = try addScoredExercise(
      bridge, title: LibraryItemView.previewExerciseWithVariations.title, key: "C",
      modality: .major,
      labels: ["C", "F", "B♭"], scores: ["C": 9, "F": 5])
    let inC = try variationId(bridge, item: id, label: "C")
    let inF = try variationId(bridge, item: id, label: "F")

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: id)))
    let entryId = try #require(try bridge.view().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: inC)))
    _ = try bridge.update(.session(.startSession(now: "2026-06-25T09:00:00Z")))
    _ = try bridge.update(
      .session(
        .switchVariation(
          entryId: entryId, variationId: inF, now: "2026-06-25T09:03:10Z", reading: .silent)))
    let active = try #require(try bridge.view().activeSession)
    let fixture = ActiveSessionView.previewActiveVariations

    #expect(fixture.currentVariationLabel == active.currentVariationLabel)
    #expect(fixture.currentVariations.map(picker) == active.currentVariations.map(picker))
  }

  // ── Driving the core ───────────────────────────────────────────────────

  private func projectedPiece(
    title: String, key: String?, modality: Modality?, marking: String?, bpm: UInt16?,
    linking exercises: [ExerciseInput]
  ) throws -> LibraryItemView {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    let pieceId = try add(
      bridge, title: title, kind: .piece, key: key, modality: modality,
      tempo: Tempo(marking: marking, bpm: bpm), labels: [])
    for exercise in exercises {
      let exerciseId = try add(
        bridge, title: exercise.title, kind: .exercise, key: exercise.key,
        modality: exercise.modality,
        tempo: exercise.tempoBpm.map { Tempo(marking: nil, bpm: $0) }, labels: [])
      _ = try bridge.update(.item(.linkExercise(pieceId: pieceId, exerciseId: exerciseId)))
    }
    return try item(bridge, pieceId)
  }

  private func projectedExercise(
    title: String, key: String?, modality: Modality?, labels: [String],
    scores: [String: UInt8]
  ) throws -> LibraryItemView {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)
    let id = try addScoredExercise(
      bridge, title: title, key: key, modality: modality, labels: labels, scores: scores)
    return try item(bridge, id)
  }

  private func addScoredExercise(
    _ bridge: LiveBridge, title: String, key: String?, modality: Modality?,
    labels: [String], scores: [String: UInt8]
  ) throws -> String {
    let id = try add(
      bridge, title: title, kind: .exercise, key: key, modality: modality, tempo: nil,
      labels: labels)
    let scored = try item(bridge, id).variants.filter { scores[$0.label] != nil }
    guard let first = scored.first else { return id }

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: id)))
    let entryId = try #require(try bridge.view().buildingSetlist?.entries.first?.id)
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: first.id)))
    _ = try bridge.update(.session(.startSession(now: stamp(minutes: 0))))
    for (index, variant) in scored.enumerated().dropFirst() {
      _ = try bridge.update(
        .session(
          .switchVariation(
            entryId: entryId, variationId: variant.id, now: stamp(minutes: index * 5),
            reading: .silent)))
    }
    let end = stamp(minutes: scored.count * 5)
    _ = try bridge.update(
      .session(.nextItem(now: end, nextItemStartedAt: end, reading: .silent)))

    let plays = try #require(try bridge.view().summary?.entries.first?.plays)
    #expect(plays.count == scored.count, "one play per scored variation")
    for play in plays {
      let label = try #require(scored.first { $0.id == play.variationId }?.label)
      let score = try #require(scores[label])
      _ = try bridge.update(
        .session(.updateEntryScore(entryId: entryId, playId: play.id, score: score)))
    }

    let requests = try bridge.update(.session(.saveSession(now: end)))
    let write = try #require(
      requests.first {
        if case .persistence(.saveSession) = $0.effect { return true } else { return false }
      })
    _ = try bridge.resolve(write.id, persistenceOutput: .ack)
    #expect(try bridge.view().error == nil, "the session saves cleanly")
    return id
  }

  private func add(
    _ bridge: LiveBridge, title: String, kind: ItemKind, key: String?, modality: Modality?,
    tempo: Tempo?, labels: [String]
  ) throws -> String {
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: title, kind: kind, composer: nil, key: key, modality: modality,
            tempo: tempo, notes: nil, tags: [], photoId: nil, variantLabels: labels))))
    let view = try bridge.view()
    return try #require(
      view.items.first { $0.title == title }?.id,
      "\(title) should land: \(view.error ?? "no error")")
  }

  private func item(_ bridge: LiveBridge, _ id: String) throws -> LibraryItemView {
    try #require(try bridge.view().items.first { $0.id == id })
  }

  private func variationId(_ bridge: LiveBridge, item id: String, label: String) throws -> String {
    try #require(try item(bridge, id).variants.first { $0.label == label }?.id)
  }

  private func stamp(minutes: Int) -> String {
    let start = Date(timeIntervalSince1970: 1_782_291_600)
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime]
    return formatter.string(from: start.addingTimeInterval(Double(minutes) * 60))
  }

  // ── Projections compared ───────────────────────────────────────────────

  private func fields(_ item: LibraryItemView) -> KeyAndTempo {
    KeyAndTempo(
      title: item.title, key: item.key, modality: item.modality,
      tempoMarking: item.tempoMarking, tempoBpm: item.tempoBpm)
  }

  private func fields(_ exercise: LinkedExerciseView) -> KeyAndTempo {
    KeyAndTempo(
      title: exercise.title, key: exercise.key, modality: exercise.modality,
      tempoMarking: exercise.tempoMarking, tempoBpm: exercise.tempoBpm)
  }

  private func ladder(_ item: LibraryItemView) -> Ladder {
    Ladder(
      key: item.key, modality: item.modality,
      rows: item.variants.map {
        VariationRow(
          label: $0.label, latestScore: $0.latestScore, isSolid: $0.isSolid, caption: $0.caption)
      },
      solidCount: item.solidVariationCount, isKeys: item.ladderIsKeys, showsKey: item.showsKey)
  }

  private func picker(_ row: PickerVariationView) -> PickerRow {
    PickerRow(label: row.label, caption: row.caption, isSolid: row.isSolid)
  }
}
