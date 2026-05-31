import Foundation
import Sentry

/// Report a non-fatal error to Sentry (a no-op when Sentry has no DSN). Used for
/// bridge/serialization failures the UI can't recover from — so a stale-bindings
/// break surfaces instead of a silently frozen screen.
func report(_ error: Error) {
  SentrySDK.capture(error: error)
}
