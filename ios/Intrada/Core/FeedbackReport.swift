import Foundation
import Sentry
import os

private let logger = Logger(subsystem: "com.intrada.native", category: "feedback")

/// What Send feedback hands to Sentry (#598): the trimmed note, and the
/// screen the tester was on unless they left it out.
struct FeedbackReport: Equatable {
  let message: String
  let attachments: [Data]?

  init?(note: String, screenshot: Data?, includeScreenshot: Bool) {
    let message = note.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !message.isEmpty else { return nil }
    self.message = message
    self.attachments = includeScreenshot ? screenshot.map { [$0] } : nil
  }

  var sentryFeedback: SentryFeedback {
    SentryFeedback(
      message: message, name: nil, email: nil, source: .custom, attachments: attachments)
  }

  static func send(_ report: FeedbackReport) {
    // Builds without a DSN never start Sentry, so the note would vanish silently.
    guard SentrySDK.isEnabled else {
      logger.error("feedback not sent: Sentry is off in this build")
      return
    }
    SentrySDK.capture(feedback: report.sentryFeedback)
  }
}
