import SharedTypes
import Testing

@testable import Intrada

struct UsualTimeLineTests {
  @Test(arguments: [
    (UInt32(8), true, "Usually 8 minutes"),
    (UInt32(1), true, "Usually 1 minute"),
    (UInt32(10), false, "10 minutes to begin with"),
    (UInt32(1), false, "1 minute to begin with"),
  ])
  func readsTheUsualTime(minutes: UInt32, fromHistory: Bool, expected: String) {
    let item = LibraryItemView.fixture(
      usualMinutes: minutes, usualMinutesFromHistory: fromHistory)
    #expect(item.usualTimeLine == expected)
  }
}
