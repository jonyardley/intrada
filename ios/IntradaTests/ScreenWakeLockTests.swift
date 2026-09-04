import SwiftUI
import Testing

@testable import Intrada

/// A hold nobody releases is invisible until the battery goes, so the release
/// paths (leaving the player, backgrounding) are what these constrain.
struct ScreenWakeLockTests {
  private final class Flag: @unchecked Sendable {
    var writes: [Bool] = []
  }

  private static func lock() -> (ScreenWakeLock, Flag) {
    let flag = Flag()
    return (ScreenWakeLock(setFlag: { flag.writes.append($0) }), flag)
  }

  @Test("a playing session holds the screen awake")
  func holdsWhileTheSessionPlays() async {
    let (lock, flag) = await MainActor.run { Self.lock() }
    await lock.update(sessionActive: true, phase: .active)
    #expect(lock.isHeld)
    #expect(flag.writes == [true])
  }

  @Test("leaving the player releases the hold")
  func releasesWhenThePlayerGoesAway() async {
    let (lock, flag) = await MainActor.run { Self.lock() }
    await lock.update(sessionActive: true, phase: .active)
    await lock.release()
    #expect(!lock.isHeld)
    #expect(flag.writes == [true, false])
  }

  @Test("backgrounding releases the hold, and coming back retakes it")
  func releasesOnBackgroundAndRetakesOnReturn() async {
    let (lock, flag) = await MainActor.run { Self.lock() }
    await lock.update(sessionActive: true, phase: .active)
    await lock.update(sessionActive: true, phase: .background)
    #expect(!lock.isHeld, "a session left open overnight must not hold the timer")
    await lock.update(sessionActive: true, phase: .active)
    #expect(lock.isHeld)
    // No repeats: the flag is process-global, and the player calls update on
    // every scene-phase change and every item transition.
    #expect(flag.writes == [true, false, true])
  }

  @Test("a glance at Control Centre keeps the hold")
  func keepsTheHoldWhileInactive() async {
    let (lock, _) = await MainActor.run { Self.lock() }
    await lock.update(sessionActive: true, phase: .active)
    await lock.update(sessionActive: true, phase: .inactive)
    #expect(lock.isHeld)
  }

  @Test("no session, no hold", arguments: [ScenePhase.active, .inactive, .background])
  func neverHoldsWithoutASession(phase: ScenePhase) async {
    let (lock, flag) = await MainActor.run { Self.lock() }
    await lock.update(sessionActive: false, phase: phase)
    #expect(!lock.isHeld)
    #expect(flag.writes.isEmpty)
  }
}
