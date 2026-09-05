import SharedTypes
import Testing

@testable import Intrada

@MainActor
struct PracticePrioritiesTests {
  @Test("Nothing starred, so there is nothing to offer")
  func hiddenWithoutStars() throws {
    let viewModel = try #require(Store.previewPractice.viewModel)
    #expect(!PracticeScreen.showsPriorities(viewModel))
  }

  @Test("Starred and idle is the state that offers the route")
  func shownWhenIdle() throws {
    let viewModel = try #require(Store.previewPracticePriorities.viewModel)
    #expect(PracticeScreen.showsPriorities(viewModel))
  }

  // The core refuses the event outside Idle, so a button surviving into any of
  // these states would raise an error the user never caused (#981).
  @Test("Anything already under way hides it")
  func hiddenWhileSessionUnderWay() throws {
    let idle = try #require(Store.previewPracticePriorities.viewModel)

    var building = idle
    building.buildingSetlist = try #require(Store.previewBuilding.viewModel?.buildingSetlist)
    #expect(!PracticeScreen.showsPriorities(building))

    var active = idle
    active.activeSession = try #require(Store.previewActive.viewModel?.activeSession)
    #expect(!PracticeScreen.showsPriorities(active))

    var summary = idle
    summary.summary = try #require(Store.previewSummary.viewModel?.summary)
    #expect(!PracticeScreen.showsPriorities(summary))
  }
}
