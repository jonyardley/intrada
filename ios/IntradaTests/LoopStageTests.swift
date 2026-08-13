import SharedTypes
import Testing

@testable import Intrada

/// The drill loop's cover lifetime (#1256 Phase D). Every exit path asks
/// `isIdle` before tearing down, so what these pin is when the cover may close
/// — a question the core has set and nobody has answered must never be one of
/// the things a close takes with it.
struct LoopStageTests {
  private func prompt(_ blockId: String = "b1") -> FeelPrompt {
    FeelPrompt(blockId: blockId, title: "Freer rubato in the intro")
  }

  private func stage(
    drill: DrillView? = nil, feel: FeelPrompt? = nil, reflection: Bool = false,
    landing: LandingView? = nil
  ) -> LoopStage {
    LoopStage.of(drill: drill, feel: feel, reflection: reflection, landing: landing)
  }

  @Test func nothingRunningAndNothingAskedIsTheOnlyStageThatCloses() {
    #expect(stage().isIdle)
  }

  @Test func aFeelQuestionHoldsTheCoverOpenAfterTheSessionEnded() {
    let stage = stage(feel: prompt())
    #expect(stage == .feel(prompt()))
    #expect(!stage.isIdle, "the last block's question would go down with the cover")
  }

  @Test func theReflectionHoldsItOpenToo() {
    let stage = stage(reflection: true)
    #expect(stage == .reflection)
    #expect(!stage.isIdle)
  }

  /// The order the block boundary depends on: the block that just closed asks
  /// its question over the next block's entry card, which bills nothing while
  /// it is up.
  @Test func theQuestionOutranksTheBlockBehindIt() {
    #expect(
      stage(drill: .preview(phase: .blockEntry), feel: prompt(), reflection: true)
        == .feel(prompt()))
  }

  @Test func theDrillOutranksAReflectionLeftSetWhileABlockIsStillRunning() {
    let drill = DrillView.preview()
    #expect(stage(drill: drill, reflection: true) == .drill(drill))
  }

  /// Answering the feel does not close the cover while the reflection is still
  /// owed — the close asks both, in order, or it asks neither.
  @Test func answeringOneQuestionLeavesTheOtherOnScreen() {
    #expect(!stage(feel: prompt(), reflection: true).isIdle)
    #expect(stage(reflection: true) == .reflection)
    #expect(stage().isIdle, "and then it may close")
  }

  // ── The soft landing (#1323) ──

  /// A close that took the landing with it would be the hard exit #1323 is
  /// about.
  @Test func theLandingHoldsTheCoverOpenUntilItIsAcknowledged() {
    let stage = stage(landing: .previewShort)
    #expect(stage == .landing(.previewShort))
    #expect(!stage.isIdle)
  }

  @Test func theQuestionsAreAskedBeforeTheLanding() {
    #expect(stage(feel: prompt(), landing: .previewShort) == .feel(prompt()))
    #expect(stage(reflection: true, landing: .previewShort) == .reflection)
    #expect(stage(landing: .previewShort) == .landing(.previewShort))
    #expect(stage().isIdle, "and only an acknowledged landing lets the cover go")
  }

  @Test func theDrillOutranksALandingLeftSetWhileABlockIsStillRunning() {
    let drill = DrillView.preview()
    #expect(stage(drill: drill, landing: .previewShort) == .drill(drill))
  }
}
