import SharedTypes
import Testing

@testable import Intrada

/// The item-complete sheet's tempo rows: what reopens as set by hand after a
/// resume, and which rows go out with the answers (#2137, #2230).
@MainActor
struct ReflectionTempoRowsTests {
  private let quavers = ClickState(
    metre: Metre(beats: 6, unit: 8, groups: [3, 3]), sounding: 0b001001)

  private func row(tempo: UInt16, setByHand: Bool) -> ReflectionTempoView {
    ReflectionTempoView(
      playId: "p1", tempo: tempo, click: quavers, band: TempoBand(unit: 8, min: 40, max: 400),
      setByHand: setByHand)
  }

  @Test(
    arguments: [
      (setByHand: true, move: nil, userSet: true, sent: 88),
      (setByHand: false, move: nil, userSet: false, sent: nil),
      (setByHand: false, move: 100, userSet: true, sent: 100),
    ] as [(setByHand: Bool, move: Int?, userSet: Bool, sent: UInt16?)])
  func aRowReopensAndGoesOutAsTheMusicianLeftIt(
    setByHand: Bool, move: Int?, userSet: Bool, sent: UInt16?
  ) {
    let row = row(tempo: 88, setByHand: setByHand)
    var tracked = TrackedTempo(row: row)
    #expect(tracked.bpm == 88)
    if let move { tracked.set(move) }

    #expect(tracked.userSet == userSet)
    let drafts = TrackedTempo.handSet([row], tracked: ["p1": tracked])
    #expect(drafts == sent.map { [DraftTempo(playId: "p1", tempo: $0, click: quavers)] } ?? [])
  }
}
