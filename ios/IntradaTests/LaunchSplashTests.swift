import Foundation
import Testing

@testable import Intrada

/// The splash's values at the handoff's key times (#2277).
struct LaunchSplashTests {
  private func near(_ a: Double, _ b: Double) -> Bool { abs(a - b) < 0.001 }

  @Test(arguments: [
    (0.9, 0.55), (1.45, 1.0), (3.0, 1.0),
  ])
  func theIconPopsFromItsSmallSizeToFull(t: Double, scale: Double) {
    #expect(near(SplashFrame.at(t).iconScale, scale))
  }

  @Test func theIconOvershootsBeforeItSettles() {
    #expect(SplashFrame.at(1.3).iconScale > 1)
  }

  @Test(arguments: [
    (1.45, 0.0), (1.65, 0.5), (1.9, 0.0),
  ])
  func oneKeyDipsAndComesBackUp(t: Double, dip: Double) {
    #expect(near(SplashFrame.at(t).keyDip, dip))
  }

  @Test func thePressedKeyStaysInTheHighlighter() {
    #expect(SplashFrame.at(1.5).keyFill == 0)
    #expect(SplashFrame.at(2.5).keyFill == 1)
  }

  @Test func theStaffLinesDrawOneAfterAnother() {
    let early = SplashFrame.at(0.4).staff
    #expect(early == early.sorted(by: >))
    #expect(early[0] > early[4])
    #expect(SplashFrame.at(1.03).staff == [1, 1, 1, 1, 1])
  }

  @Test(arguments: [
    (3.1, 0.0), (3.85, 1.0),
  ])
  func theWordmarkRisesToTheWelcomeDuringTheHandoff(t: Double, travel: Double) {
    #expect(near(SplashFrame.at(t).wordTravel, travel))
  }

  @Test func theWordmarkAppearsBeforeTheSwipeFinishes() {
    let frame = SplashFrame.at(2.4)
    #expect(frame.wordOpacity == 1)
    #expect(frame.swipe < 1)
    #expect(SplashFrame.at(2.8).swipe == 1)
  }

  @Test func theSplashClearsAsTheHandoffStarts() {
    #expect(SplashFrame.at(3.0).splashOpacity == 1)
    #expect(SplashFrame.at(3.45).splashOpacity == 0)
    #expect(near(SplashFrame.at(3.45).splashLift, -20))
  }

  @Test func theWelcomeBuildsOneItemAtATime() {
    let items = SplashFrame.at(3.9).items.map(\.opacity)
    #expect(items == items.sorted(by: >))
    #expect(items[0] > 0)
    #expect(items[5] == 0)
  }

  @Test func everythingIsAtRestWhenTheSplashEnds() {
    let end = SplashFrame.at(SplashFrame.duration)
    #expect(end.items.allSatisfy { $0.opacity == 1 && $0.offset == 0 })
    #expect(end.wordTravel == 1)
    #expect(end.wordRise == 0)
    #expect(end.splashOpacity == 0)
  }

  /// The generated plist overwrites this key if `UILaunchScreen_Generation` comes back.
  @Test func theLaunchScreenIsPaper() {
    let screen = Bundle.main.object(forInfoDictionaryKey: "UILaunchScreen") as? [String: Any]
    #expect(screen?["UIColorName"] as? String == "LaunchPaper")
  }
}

/// The returning launch's values at the design's cue times (#2278).
struct ReturningSplashTests {
  private func near(_ a: Double, _ b: Double) -> Bool { abs(a - b) < 0.001 }
  private func frame(_ t: Double) -> SplashFrame { .returning(at: t, reduceMotion: false) }

  @Test func theStaveDrawsTopToBottomInHalfASecond() {
    let early = frame(0.2).staff
    #expect(early == early.sorted(by: >))
    #expect(early[0] > early[4])
    #expect(frame(0.51).staff == [1, 1, 1, 1, 1])
  }

  @Test func theIconSettlesWithoutOvershooting() {
    #expect(near(frame(0).iconScale, 0.92))
    #expect(stride(from: 0.0, through: 0.3, by: 0.01).allSatisfy { frame($0).iconScale <= 1 })
    #expect(frame(0.3).iconScale == 1)
    #expect(frame(0.3).iconOpacity == 1)
  }

  @Test func theKeyIsAlreadyLit() {
    #expect(frame(0.1).keyFill == 1)
    #expect(frame(0.1).keyDip == 0)
  }

  @Test func theWordmarkSettlesBeforeTheSwipeFinishes() {
    #expect(frame(0.4).wordOpacity == 1)
    #expect(frame(0.4).wordRise == 0)
    #expect(frame(0.4).swipe < 1)
    #expect(frame(0.55).swipe == 1)
  }

  @Test(arguments: [0.55, 0.75])
  func everythingHoldsBeforeTheLift(t: Double) {
    #expect(frame(t).splashOpacity == 1)
    #expect(frame(t).splashLift == 0)
    #expect(frame(t).paperOpacity == 1)
  }

  @Test func theSplashLiftsAwayAndThePaperClears() {
    #expect(frame(1.05).splashOpacity == 0)
    #expect(near(frame(1.05).splashLift, -12))
    #expect(frame(0.85).paperOpacity == 1)
    #expect(frame(1.2).paperOpacity == 0)
    #expect(SplashFrame.returningDuration(reduceMotion: false) == 1.2)
  }

  @Test func reduceMotionOnlyFadesThePaper() {
    let start = SplashFrame.returning(at: 0, reduceMotion: true)
    #expect(start.iconOpacity == 0)
    #expect(start.wordOpacity == 0)
    #expect(start.staff == [0, 0, 0, 0, 0])
    #expect(start.paperOpacity == 1)
    let end = SplashFrame.returning(at: IntradaMotion.reduceFade, reduceMotion: true)
    #expect(end.paperOpacity == 0)
    #expect(SplashFrame.returningDuration(reduceMotion: true) == IntradaMotion.reduceFade)
  }
}

/// Which launches play the returning splash (#2278), through the real bridge
/// so the welcome is only known once the library and history load.
@MainActor
struct ReturningSplashLaunchTests {
  private func defaults() -> UserDefaults {
    UserDefaults(suiteName: "returning-splash-\(UUID().uuidString)") ?? .standard
  }

  @Test func aFreshInstallWaitsForTheWelcomeInstead() async {
    let store = Store(bridge: LiveBridge(), sortDefaults: defaults())
    store.send(.startApp)
    #expect(await ReturningSplash.decide(for: store) == .done)
  }

  @Test func aReturningMusicianSeesIt() async {
    let suite = defaults()
    let first = Store(bridge: LiveBridge(), sortDefaults: suite)
    first.send(.startApp)
    await first.settle()
    first.send(.firstRun(.skipWelcome))

    let second = Store(bridge: LiveBridge(), sortDefaults: suite)
    second.send(.startApp)
    second.restorePersistedFirstRun()
    #expect(await ReturningSplash.decide(for: second) == .playing)
  }

  @Test func aSessionToReopenSkipsIt() async {
    let store = Store.previewStartHere
    store.recoverableSession = Store.previewPracticeRecovery.recoverableSession
    #expect(await ReturningSplash.decide(for: store) == .done)
  }
}
