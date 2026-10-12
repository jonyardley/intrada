import SharedTypes
import Testing

@testable import Intrada

@MainActor
struct PracticeSuggestionRestoreTests {
  @Test("Not dismissed, so there is nothing to bring back")
  func hiddenWhenNotDismissed() throws {
    let viewModel = try #require(Store.previewPracticeSuggestion.viewModel)
    #expect(!PracticeScreen.showsSuggestionRestore(viewModel, dismissed: false))
  }

  @Test("Dismissed but the core has nothing to suggest")
  func hiddenWithoutSuggestion() throws {
    let viewModel = try #require(Store.previewPractice.viewModel)
    #expect(!PracticeScreen.showsSuggestionRestore(viewModel, dismissed: true))
  }

  @Test("No view model at all")
  func hiddenWithNilViewModel() {
    #expect(!PracticeScreen.showsSuggestionRestore(nil, dismissed: true))
  }

  @Test("Dismissed and idle is the state that offers the way back")
  func shownWhenDismissedAndIdle() throws {
    let viewModel = try #require(Store.previewPracticeSuggestion.viewModel)
    #expect(PracticeScreen.showsSuggestionRestore(viewModel, dismissed: true))
  }

  @Test("Dismissed, but the core says the link may not show")
  func hiddenWhenTheCoreSaysNo() throws {
    var viewModel = try #require(Store.previewPracticeSuggestion.viewModel)
    viewModel.showsSuggestionRestore = false
    #expect(!PracticeScreen.showsSuggestionRestore(viewModel, dismissed: true))
  }
}
