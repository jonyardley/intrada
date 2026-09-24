import Testing

@testable import Intrada

struct AppBannerStackTests {

  @Test func everyConditionShowsInShellOrder() {
    let banners = AppBanner.stack(
      halted: true, degraded: true, error: "Couldn't save that.", notice: "No tempo.")
    #expect(
      banners == [
        .halted, .storageUnavailable, .error("Couldn't save that."), .notice("No tempo."),
      ]
    )
  }

  @Test func storageFailureAloneStillShows() {
    #expect(
      AppBanner.stack(halted: false, degraded: true, error: nil, notice: nil)
        == [.storageUnavailable])
  }

  @Test func nothingToShowIsEmpty() {
    #expect(AppBanner.stack(halted: false, degraded: false, error: nil, notice: nil).isEmpty)
  }
}
