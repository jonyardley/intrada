import Testing

@testable import Intrada

struct ConsistencyBarsTests {
  @Test(arguments: [
    (4, "4 weeks ago"),
    (3, "3 weeks ago"),
    (2, "2 weeks ago"),
    (1, "Last week"),
    (0, "This week"),
  ])
  func spokenWhenSaysHowLongAgo(weeksAgo: Int, expected: String) {
    #expect(ConsistencyBars.spokenWhen(weeksAgo: weeksAgo) == expected)
  }
}
