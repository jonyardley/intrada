import Foundation
import Testing

@testable import Intrada

struct FeedbackReportTests {
  private let screenshot = Data([0xFF, 0xD8, 0xFF])

  @Test(
    "a note with nothing but space is not sent",
    arguments: ["", " ", "\n\n", " \t\n "])
  func refusesAnEmptyNote(note: String) {
    #expect(FeedbackReport(note: note, screenshot: screenshot, includeScreenshot: true) == nil)
  }

  @Test("the note is sent without the space around it")
  func trimsTheNote() throws {
    let report = try #require(
      FeedbackReport(
        note: "  The timer kept going\n", screenshot: nil, includeScreenshot: true))
    #expect(report.message == "The timer kept going")
  }

  @Test("the screenshot goes only when the tester keeps it")
  func attachesTheScreenshotWhenKept() throws {
    let kept = try #require(
      FeedbackReport(note: "Looks off", screenshot: screenshot, includeScreenshot: true))
    let left = try #require(
      FeedbackReport(note: "Looks off", screenshot: screenshot, includeScreenshot: false))
    #expect(kept.attachments == [screenshot])
    #expect(left.attachments == nil)
  }

  @Test("from Profile there is no screenshot to attach")
  func sendsNoAttachmentWithoutAScreenshot() throws {
    let report = try #require(
      FeedbackReport(note: "Add a drone", screenshot: nil, includeScreenshot: true))
    #expect(report.attachments == nil)
  }
}
