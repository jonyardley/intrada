import SharedTypes
import Testing

@testable import Intrada

/// The welcome and the first-session steps through the real bridge (#846, #2116).
@MainActor
struct FirstRunBridgeTests {

  /// Starts the app and answers both loads with nothing, as a fresh install's store would.
  private func freshInstall() throws -> RowsBridge {
    let bridge = RowsBridge()
    for request in try bridge.update(.startApp) {
      guard case .persistence(let operation) = request.effect else { continue }
      switch operation {
      case .loadItems: _ = try bridge.resolve(request.id, persistenceOutput: .items([]))
      case .loadSessions: _ = try bridge.resolve(request.id, persistenceOutput: .sessions([]))
      default: continue
      }
    }
    return bridge
  }

  @Test func aFreshInstallShowsTheWelcomeAndTheStartHereCard() throws {
    let bridge = try freshInstall()

    #expect(
      try bridge.rendered().firstRun
        == FirstRunView(
          showsWelcome: true, showsStartHere: true, added: false, built: false, played: false,
          marked: false, firstItemTitle: nil))
  }

  @Test func skipSavesTheDismissalAndHidesTheWelcome() throws {
    let bridge = try freshInstall()

    let skip: FirstRunEvent = .skipWelcome
    let saved = try bridge.update(.firstRun(skip)).compactMap { request -> FirstRun? in
      if case .app(.saveFirstRun(let firstRun)) = request.effect { return firstRun }
      return nil
    }

    #expect(saved == [FirstRun(welcomeSeen: true)])
    #expect(try bridge.rendered().firstRun.showsWelcome == false)
  }

  @Test func aRestoredDismissalKeepsTheWelcomeAway() throws {
    let bridge = try freshInstall()

    _ = try bridge.update(.firstRun(.loaded(FirstRun(welcomeSeen: true))))

    let firstRun = try bridge.rendered().firstRun
    #expect(firstRun.showsWelcome == false)
    #expect(firstRun.showsStartHere, "the card does not depend on the welcome")
  }
}
