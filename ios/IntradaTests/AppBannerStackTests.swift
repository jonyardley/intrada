import Testing

@testable import Intrada

struct AppBannerStackTests {

  @Test func everyConditionShowsInShellOrder() {
    let banners = AppBanner.stack(
      halted: true, degraded: true, error: "Couldn't save that.", notice: "No tempo.",
      undoDelete: "Session deleted")
    #expect(
      banners == [
        .halted, .storageUnavailable, .error("Couldn't save that."), .notice("No tempo."),
        .undoDelete("Session deleted"),
      ]
    )
  }

  @Test func storageFailureAloneStillShows() {
    #expect(
      AppBanner.stack(halted: false, degraded: true, error: nil, notice: nil, undoDelete: nil)
        == [.storageUnavailable])
  }

  @Test func nothingToShowIsEmpty() {
    #expect(
      AppBanner.stack(halted: false, degraded: false, error: nil, notice: nil, undoDelete: nil)
        .isEmpty)
  }
}
