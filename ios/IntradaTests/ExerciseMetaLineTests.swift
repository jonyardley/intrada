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
      name: "a modeless sharp",
      exercise: .fixture(key: Key(letter: .f, accidental: .sharp, mode: nil)),
      line: "F♯", spoken: "F♯"),
    LinkedCase(
      name: "key only", exercise: .fixture(key: Key(letter: .d, accidental: .flat, mode: .major)),
      line: "D♭ major", spoken: "D♭ major"),
    LinkedCase(
      name: "tempo only",
      exercise: .fixture(tempoLine: "♩ = 108", tempoLineSpoken: "108 beats per minute"),
      line: "♩ = 108", spoken: "108 beats per minute"),
    LinkedCase(name: "neither", exercise: .fixture(), line: nil, spoken: nil),
    LinkedCase(
      name: "key with marking plus bpm",
      exercise: .fixture(
        key: Key(letter: .c, accidental: .natural, mode: .minor),
        tempoLine: "Allegro · ♩ = 132", tempoLineSpoken: "Allegro, 132 beats per minute"),
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
    let key: Key?
    let bpm: String
    let meta: String?
  }

  private let stagedCases: [StagedCase] = [
    StagedCase(
      name: "words typed as a tempo", key: Key(letter: .c, accidental: .natural, mode: .major),
      bpm: "fast", meta: "C major"),
    StagedCase(name: "nothing filled in", key: nil, bpm: "", meta: nil),
    StagedCase(name: "a padded tempo", key: nil, bpm: " 96 ", meta: "♩ = 96"),
    StagedCase(
      name: "key and tempo", key: Key(letter: .b, accidental: .flat, mode: .minor), bpm: "80",
      meta: "B♭ minor · ♩ = 80"),
  ]

  @Test func aStagedExerciseShowsOnlyTheKeyAndTempoThatWillSave() {
    for row in stagedCases {
      let staged = StagedExercise.draft(
        id: UUID(), title: "Scale", key: row.key, bpm: row.bpm)
      #expect(staged.meta == row.meta, "\(row.name)")
    }
  }

  // ── Editing a key ──────────────────────────────────────────────────────

  @Test func editingAnItemSeedsTheFormWithItsKey() {
    let key = Key(letter: .f, accidental: .sharp, mode: .major)
    let form = ItemFormModel(item: LibraryItemFixture.view(key: key))

    #expect(form.key == key)
  }
}
