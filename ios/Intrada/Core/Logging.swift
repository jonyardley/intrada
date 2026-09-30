import Foundation
import Sentry
import os

private let logger = Logger(subsystem: "com.intrada.native", category: "core")

/// Per-event bridge timing for Instruments (#1801).
let bridgeSignposter = OSSignposter(subsystem: "com.intrada.native", category: .pointsOfInterest)

// Lets a test catch what `report` sends (#2019); the app never sets it.
let reportObserver = OSAllocatedUnfairLock<(@Sendable (Error, String?) -> Void)?>(
  initialState: nil)

/// Non-fatal errors `Store` swallows via `guarded` (the #846 silent-no-op
/// class). Logs to the unified log too, so they're visible in dev/CI where
/// Sentry has no DSN.
func report(_ error: Error, _ context: String? = nil) {
  reportObserver.withLock { $0 }?(error, context)
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
