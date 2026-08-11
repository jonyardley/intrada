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

  @Test func nothingRunningAndNothingAskedIsTheOnlyStageThatCloses() {
    #expect(LoopStage.of(drill: nil, feel: nil, reflection: false).isIdle)
  }

  @Test func aFeelQuestionHoldsTheCoverOpenAfterTheSessionEnded() {
    let stage = LoopStage.of(drill: nil, feel: prompt(), reflection: false)
    #expect(stage == .feel(prompt()))
    #expect(!stage.isIdle, "the last block's question would go down with the cover")
  }

  @Test func theReflectionHoldsItOpenToo() {
    let stage = LoopStage.of(drill: nil, feel: nil, reflection: true)
    #expect(stage == .reflection)
    #expect(!stage.isIdle)
  }

  /// The order the block boundary depends on: the block that just closed asks
  /// its question over the next block's entry card, which bills nothing while
  /// it is up.
  @Test func theQuestionOutranksTheBlockBehindIt() {
    let stage = LoopStage.of(drill: .preview(phase: .blockEntry), feel: prompt(), reflection: true)
    #expect(stage == .feel(prompt()))
  }

  @Test func theDrillOutranksAReflectionLeftSetWhileABlockIsStillRunning() {
    let drill = DrillView.preview()
    #expect(LoopStage.of(drill: drill, feel: nil, reflection: true) == .drill(drill))
  }

  /// Answering the feel does not close the cover while the reflection is still
  /// owed — the close asks both, in order, or it asks neither.
  @Test func answeringOneQuestionLeavesTheOtherOnScreen() {
    let both = LoopStage.of(drill: nil, feel: prompt(), reflection: true)
    #expect(!both.isIdle)
    let afterFeel = LoopStage.of(drill: nil, feel: nil, reflection: true)
    #expect(afterFeel == .reflection)
    #expect(LoopStage.of(drill: nil, feel: nil, reflection: false).isIdle, "and then it may close")
  }
}
