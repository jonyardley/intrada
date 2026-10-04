import Foundation
import Sentry
import SwiftUI
import UIKit

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
    SentrySDK.capture(feedback: report.sentryFeedback)
  }
}

extension UIWindow {
  /// JPEG keeps a full-screen capture well under Sentry's attachment limit.
  @MainActor func feedbackScreenshot() -> Data? {
    let image = UIGraphicsImageRenderer(bounds: bounds).image { _ in
      drawHierarchy(in: bounds, afterScreenUpdates: false)
    }
    return image.jpegData(compressionQuality: 0.7)
  }

  // Overridden here because SwiftUI owns the window, so it cannot be subclassed.
  override open func motionEnded(_ motion: UIEvent.EventSubtype, with event: UIEvent?) {
    super.motionEnded(motion, with: event)
    if motion == .motionShake { FeedbackPresenter.presentOnShake(from: self) }
  }
}

/// Shake opens the form over whatever is showing, the player and other
/// sheets included, which a `.sheet` on the root view cannot do.
@MainActor
enum FeedbackPresenter {
  static func presentOnShake(from window: UIWindow) {
    guard var top = window.rootViewController else { return }
    while let presented = top.presentedViewController { top = presented }
    guard !(top is FeedbackHostingController) else { return }
    let screenshot = window.feedbackScreenshot()
    top.present(FeedbackHostingController(screenshot: screenshot), animated: true)
  }
}

private final class FeedbackHostingController: UIHostingController<FeedbackSheet> {
  init(screenshot: Data?) {
    super.init(rootView: FeedbackSheet(screenshot: screenshot))
  }

  @available(*, unavailable)
  required init?(coder: NSCoder) { nil }
}
