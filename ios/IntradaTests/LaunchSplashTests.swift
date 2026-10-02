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
