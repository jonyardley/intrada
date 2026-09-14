import SharedTypes
import Testing

@testable import Intrada

/// The shell-side derivation #1739 Phase B left: what the item-complete
/// sheet's rows say.
struct VariationScreenTests {

  private func play(
    _ id: String, _ label: String?, seconds: UInt64, tempo: UInt16? = nil,
    repCount: UInt8? = nil, repTarget: UInt8? = nil, isMarkable: Bool = true
  ) -> VariationPlayView {
    VariationPlayView(
      id: id, variationId: label.map { "v-\($0)" }, variationLabel: label, seconds: seconds,
      durationDisplay: "4m 10s", repTarget: repTarget, repCount: repCount,
      repTargetReached: nil, repHistory: nil, achievedTempo: tempo, clickPattern: nil,
      tempoDisplay: tempo,
      score: nil, isMarkable: isMarkable)
  }

  // ── The sheet's rows ──

  @Test("a row's duration is the play's own stamped seconds, open play included")
  func rowsReadEachPlaysOwnSeconds() {
    let rows = ReflectionPlay.rows(
      [play("p1", "C", seconds: 250), play("p2", "G", seconds: 510)])

    #expect(rows.map(\.durationDisplay) == ["04:10", "08:30"])
  }

  @Test("one play takes its own stamped seconds")
  func aSinglePlayTakesItsOwnSeconds() {
    let rows = ReflectionPlay.rows([play("p1", nil, seconds: 420)])

    #expect(rows.map(\.durationDisplay) == ["07:00"])
  }

  @Test("a row carries whether the core predicts it survives the drop")
  func rowsCarryWhetherTheyAreMarkable() {
    let rows = ReflectionPlay.rows([
      play("p1", "C", seconds: 250, isMarkable: true),
      play("p2", "G", seconds: 2, isMarkable: false),
    ])

    #expect(rows.map(\.isMarkable) == [true, false])
  }

  @Test("a row names its variation, and an unattributed play says so")
  func rowsCarryTheirLabel() {
    let rows = ReflectionPlay.rows(
      [play("p1", "C major", seconds: 0), play("p2", nil, seconds: 0)])

    #expect(rows.map(\.title) == ["C major", "No variation"])
  }

  @Test("a row shows repetitions only when the entry set a target")
  func rowMetaShowsRepetitionsOnlyAgainstATarget() {
    let withTarget = ReflectionPlay.rows(
      [play("p1", "C", seconds: 250, repCount: 8, repTarget: 10)])
    let without = ReflectionPlay.rows(
      [play("p1", "C", seconds: 250, repCount: 8)])

    #expect(withTarget.first?.meta == "04:10 · 8 of 10")
    #expect(without.first?.meta == "04:10")
  }

  @Test("a target with no repetitions banked reads as none of the target, not blank")
  func rowMetaCountsZeroAgainstTheTarget() {
    let rows = ReflectionPlay.rows([play("p1", "C", seconds: 250, repTarget: 10)])

    #expect(rows.first?.meta == "04:10 · 0 of 10")
  }

  // ── The read-only meta line ──

  @Test("a play's meta names only what it measured")
  func metaPartsStateOnlyWhatWasMeasured() {
    #expect(play("p", "C", seconds: 250).metaParts == ["4m 10s"])
    #expect(play("p", "C", seconds: 250, tempo: 96).metaParts == ["4m 10s", "96 bpm"])
    #expect(
      play("p", "C", seconds: 250, tempo: 96, repCount: 8, repTarget: 10).metaParts
        == ["4m 10s", "96 bpm", "8 of 10 reps"])
  }

  @Test("an unattributed play is named rather than left blank")
  func unattributedPlaysAreNamed() {
    #expect(play("p", nil, seconds: 0).displayLabel == "No variation")
    #expect(play("p", "B♭ major", seconds: 0).displayLabel == "B♭ major")
  }
}
