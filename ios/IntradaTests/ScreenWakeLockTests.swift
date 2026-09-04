import SwiftUI
import Testing
import UIKit

@testable import Intrada

/// A hold nobody releases is invisible until the battery goes, so the release
/// paths (leaving the player, backgrounding) are what these constrain.
@MainActor
struct ScreenWakeLockTests {
  private final class Flag {
    var writes: [Bool] = []
  }

  private static func lock() -> (ScreenWakeLock, Flag) {
    let flag = Flag()
    return (ScreenWakeLock(setFlag: { flag.writes.append($0) }), flag)
  }

  @Test("a playing session holds the screen awake")
  func holdsWhileTheSessionPlays() {
    let (lock, flag) = Self.lock()
    lock.update(sessionActive: true, phase: .active)
    #expect(lock.isHeld)
    #expect(flag.writes == [true])
  }

  @Test("leaving the player releases the hold")
  func releasesWhenThePlayerGoesAway() {
    let (lock, flag) = Self.lock()
    lock.update(sessionActive: true, phase: .active)
    lock.release()
    #expect(!lock.isHeld)
    #expect(flag.writes == [true, false])
  }

  @Test("backgrounding releases the hold, and coming back retakes it")
  func releasesOnBackgroundAndRetakesOnReturn() {
    let (lock, flag) = Self.lock()
    lock.update(sessionActive: true, phase: .active)
    lock.update(sessionActive: true, phase: .background)
    #expect(!lock.isHeld, "a session left open overnight must not hold the timer")
    lock.update(sessionActive: true, phase: .active)
    #expect(lock.isHeld)
    #expect(flag.writes == [true, false, true], "and no repeat writes in between")
  }

  @Test("a glance at Control Centre keeps the hold")
  func keepsTheHoldWhileInactive() {
    let (lock, _) = Self.lock()
    lock.update(sessionActive: true, phase: .active)
    lock.update(sessionActive: true, phase: .inactive)
    #expect(lock.isHeld)
  }

  @Test("no session, no hold", arguments: [ScenePhase.active, .inactive, .background])
  func neverHoldsWithoutASession(phase: ScenePhase) {
    let (lock, flag) = Self.lock()
    lock.update(sessionActive: false, phase: phase)
    #expect(!lock.isHeld)
    #expect(flag.writes.isEmpty)
  }

  @Test("the production applier drives UIApplication")
  func productionApplierDrivesUIApplication() {
    let lock = ScreenWakeLock.system()
    lock.update(sessionActive: true, phase: .active)
    #expect(UIApplication.shared.isIdleTimerDisabled)
    lock.update(sessionActive: true, phase: .background)
    #expect(!UIApplication.shared.isIdleTimerDisabled)
  }
}

/// Every test above injects a closure and mounts nothing, so all of them would
/// pass with the player wired to no lock at all. These mount the real screen,
/// and are the gate on `initial: true`, without which nothing was held for a
/// session started in the foreground (#1513 review) — the everyday path.
@MainActor
struct ScreenWakeLockWiringTests {
  /// The test host has no `UIWindowScene`, so `scenePhase` needs supplying.
  private func mount(_ lock: ScreenWakeLock) -> UIHostingController<AnyView> {
    IntradaFonts.register()
    let controller = UIHostingController(
      rootView: AnyView(
        FocusPlayerScreen()
          .environment(Store.previewActive)
          .environment(\.screenWakeLock, lock)
          .environment(\.scenePhase, .active)))
    let window = UIWindow(frame: CGRect(x: 0, y: 0, width: 390, height: 844))
    window.rootViewController = controller
    window.isHidden = false
    controller.view.layoutIfNeeded()
    return controller
  }

  /// Nilling the window's root does not fire `onDisappear`; this does.
  private func dismiss(_ controller: UIHostingController<AnyView>) {
    controller.rootView = AnyView(EmptyView())
    controller.view.layoutIfNeeded()
    RunLoop.current.run(until: Date().addingTimeInterval(0.1))
  }

  @Test("mounting the player takes the hold, and leaving it gives it back")
  func mountingThePlayerHoldsTheIdleTimer() {
    var writes: [Bool] = []
    let controller = mount(ScreenWakeLock { writes.append($0) })

    #expect(writes == [true], "a mounted player must take the hold on appearance")

    dismiss(controller)
    #expect(writes == [true, false], "leaving the player must give the hold back")
  }

  @Test("a mounted player holds the real idle timer")
  func mountedPlayerHoldsTheRealIdleTimer() {
    let controller = mount(.system())
    #expect(UIApplication.shared.isIdleTimerDisabled)

    dismiss(controller)
    #expect(!UIApplication.shared.isIdleTimerDisabled)
  }
}
