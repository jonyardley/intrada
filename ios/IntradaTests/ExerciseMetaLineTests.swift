import Foundation
import SharedTypes
import Testing

@testable import Intrada

@MainActor
struct ExerciseMetaLineTests {

  // ── A linked exercise's row ────────────────────────────────────────────

  private struct LinkedCase {
    let name: String
    let exercise: LinkedExerciseView
    let line: String?
    let spoken: String?
  }

  private let linkedCases: [LinkedCase] = [
    LinkedCase(
      name: "a modeless sharp", exercise: .fixture(key: "F#"), line: "F♯", spoken: "F♯"),
    LinkedCase(
      name: "key only", exercise: .fixture(key: "Db", modality: .major),
      line: "D♭ major", spoken: "D♭ major"),
    LinkedCase(
      name: "tempo only", exercise: .fixture(tempoBpm: 108),
      line: "♩ = 108", spoken: "108 beats per minute"),
    LinkedCase(name: "neither", exercise: .fixture(), line: nil, spoken: nil),
    LinkedCase(
      name: "marking plus bpm", exercise: .fixture(tempoMarking: "Allegro", tempoBpm: 132),
      line: "Allegro · ♩ = 132", spoken: "Allegro, 132 beats per minute"),
    LinkedCase(
      name: "key with marking plus bpm",
      exercise: .fixture(key: "C", modality: .minor, tempoMarking: "Allegro", tempoBpm: 132),
      line: "C minor · Allegro · ♩ = 132", spoken: "C minor, Allegro, 132 beats per minute"),
  ]

  @Test func aLinkedExerciseReadsItsKeyAndTempoOnOneLine() {
    for row in linkedCases {
      #expect(row.exercise.metaLine == row.line, "\(row.name)")
    }
  }

  @Test func aLinkedExerciseSpeaksItsKeyAndTempo() {
    for row in linkedCases {
      #expect(row.exercise.metaSpoken == row.spoken, "\(row.name)")
    }
  }

  // ── A new exercise staged on the add form ──────────────────────────────

  private struct StagedCase {
    let name: String
    let key: String
    let modality: Modality?
    let bpm: String
    let meta: String?
  }

  private let stagedCases: [StagedCase] = [
    StagedCase(
      name: "words typed as a tempo", key: "C", modality: .major, bpm: "fast",
      meta: "C major"),
    StagedCase(name: "nothing filled in", key: "", modality: nil, bpm: "", meta: nil),
    StagedCase(name: "a mode with no key", key: "", modality: .minor, bpm: "", meta: nil),
    StagedCase(name: "a padded tempo", key: "", modality: nil, bpm: " 96 ", meta: "♩ = 96"),
    StagedCase(
      name: "key and tempo", key: "Bb", modality: .minor, bpm: "80",
      meta: "B♭ minor · ♩ = 80"),
  ]

  @Test func aStagedExerciseShowsOnlyTheKeyAndTempoThatWillSave() {
    for row in stagedCases {
      let staged = StagedExercise.draft(
        id: UUID(), title: "Scale", key: row.key, modality: row.modality, bpm: row.bpm)
      #expect(staged.meta == row.meta, "\(row.name)")
    }
  }

  // ── Editing a legacy key ───────────────────────────────────────────────

  @Test func editingALegacyFreeformKeySeedsTheTonicAndModeTheCoreParsed() {
    let item = LibraryItemFixture.view(
      key: "F# major", modality: nil,
      keySelection: KeyWheelSelection(ring: 6, modality: .major, spelling: "F#"))

    let form = ItemFormModel(item: item)

    #expect(form.key == "F#")
    #expect(form.modality == .major)
  }
}
