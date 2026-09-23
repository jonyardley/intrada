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

  @Test(arguments: [
    (0, 40, "4 weeks ago: 40 minutes"),
    (3, 95, "Last week: 95 minutes"),
    (4, 82, "This week: 82 minutes"),
    (2, 1, "2 weeks ago: 1 minute"),
  ])
  func spokenLabelCountsBackFromTheLastBar(index: Int, minutes: Int, expected: String) {
    #expect(ConsistencyBars.spokenLabel(index: index, count: 5, minutes: minutes) == expected)
  }
}
