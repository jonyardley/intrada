import Foundation
import Sentry
import os

private let logger = Logger(subsystem: "com.intrada.native", category: "core")

/// Non-fatal errors `Store` swallows via `guarded` (the #846 silent-no-op
/// class). Logs to the unified log too, so they're visible in dev/CI where
/// Sentry has no DSN.
func report(_ error: Error, _ context: String? = nil) {
  if let context {
    logger.error("\(context, privacy: .public): \(String(describing: error), privacy: .public)")
  } else {
    logger.error("\(String(describing: error), privacy: .public)")
  }
  SentrySDK.capture(error: error) { scope in
    guard let context else { return }
    scope.setTag(value: context, key: "report_context")
    scope.setFingerprint(["{{ default }}", context])
  }
}
