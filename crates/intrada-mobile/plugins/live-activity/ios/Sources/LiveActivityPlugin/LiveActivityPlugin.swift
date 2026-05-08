// Phase C of intrada#474 — drives an ActivityKit Live Activity for the
// active practice session. Layers on top of the background-audio
// plugin (#309): same lifecycle, different surface.
//
// Design (per specs/live-activity-plugin.md):
// - Single in-flight activity at a time. Stored on the plugin instance
//   so `update` and `end` can target the right activity reference.
// - Wall-clock anchor in ContentState — the widget extension's SwiftUI
//   views use `Text(timerInterval:)` / `ProgressView(timerInterval:)` so
//   iOS handles the per-second update without IPC traffic. We push only
//   on item advance.
// - `dismissalPolicy: .immediate` on end — once the user puts the
//   instrument down, the activity disappears within ~1s. Apple's
//   default `.default` keeps it for ~8h which feels wrong for practice.
// - All ActivityKit calls gated with `@available(iOS 16.1, *)`.
//   Older devices let the plugin commands resolve; the activity is
//   just silently absent.

#if canImport(ActivityKit)
  import ActivityKit
#endif

import Foundation
import IntradaActivityShared
import SwiftRs
import Tauri
import UIKit
import WebKit

struct BeginArgs: Decodable {
  let item_title: String
  let position_label: String
  let started_at: String  // RFC3339 UTC
  let planned_duration_secs: UInt32?
}

struct UpdateArgs: Decodable {
  let item_title: String
  let position_label: String
  let started_at: String
  let planned_duration_secs: UInt32?
}

class LiveActivityPlugin: Plugin {
  // Currently-active activity reference. Held for the lifetime of a
  // session so `update` and `end` can target it. `Any?` (rather than
  // typed) so this stored property compiles on iOS < 16.1; the actual
  // type is `Activity<IntradaActivityAttributes>?` enforced via casts
  // at use sites.
  private var currentActivity: Any?

  // Cached because ISO8601DateFormatter is expensive to instantiate and
  // we re-parse on every `begin` / `update`.
  private static let rfc3339WithFractional: ISO8601DateFormatter = {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return f
  }()
  private static let rfc3339Plain: ISO8601DateFormatter = {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime]
    return f
  }()

  // Tauri dispatches `Invoke` calls on a serial per-plugin queue, but
  // ActivityKit's API is documented as main-thread-safe. We dispatch
  // every state mutation to the main queue to mirror background-audio's
  // pattern and avoid surprises.

  @objc public func begin(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(BeginArgs.self)
    let started = parseRfc3339(args.started_at) ?? Date()

    DispatchQueue.main.async { [weak self] in
      guard let self = self else { return }

      if #available(iOS 16.1, *) {
        // If a previous session left an activity hanging (app crash
        // mid-session, ActivityKit error path), end it before starting
        // the new one. iOS allows only one activity per attribute type
        // before throwing `.tooManyActivitiesForApplication`.
        self.endCurrentActivityIfAny()

        let state = IntradaActivityAttributes.ContentState(
          itemTitle: args.item_title,
          positionLabel: args.position_label,
          startedAt: started,
          plannedDurationSecs: args.planned_duration_secs
        )

        do {
          let activity = try Activity<IntradaActivityAttributes>.request(
            attributes: IntradaActivityAttributes(),
            contentState: state,
            pushType: nil
          )
          self.currentActivity = activity
          invoke.resolve()
        } catch {
          // Common failure modes:
          // - User disabled Live Activities in Settings
          // - Missing NSSupportsLiveActivities in Info.plist
          // - .tooManyActivitiesForApplication if cleanup above raced
          // The wall-clock timer + background-audio still work; only
          // the lock-screen card is missing. Surface to the bridge so
          // Sentry catches it (per the plugin's Rust-side capture).
          invoke.reject(
            "live-activity: Activity.request failed: \(error.localizedDescription)")
        }
      } else {
        // Older iOS: no Live Activities — resolve silently. Background
        // audio still gives the user a lock-screen Now Playing card.
        invoke.resolve()
      }
    }
  }

  @objc public func update(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(UpdateArgs.self)
    let started = parseRfc3339(args.started_at) ?? Date()

    DispatchQueue.main.async { [weak self] in
      guard let self = self else { return }

      if #available(iOS 16.1, *) {
        guard let activity = self.currentActivity as? Activity<IntradaActivityAttributes>
        else {
          // No activity in flight — `update` arrived without a matching
          // `begin`, or the activity already ended (e.g. user revoked
          // Live Activities mid-session). No-op rather than error: the
          // shell-side lifecycle Effect can race ahead of iOS state.
          invoke.resolve()
          return
        }

        let state = IntradaActivityAttributes.ContentState(
          itemTitle: args.item_title,
          positionLabel: args.position_label,
          startedAt: started,
          plannedDurationSecs: args.planned_duration_secs
        )

        Task {
          await activity.update(using: state)
          invoke.resolve()
        }
      } else {
        invoke.resolve()
      }
    }
  }

  @objc public func end(_ invoke: Invoke) {
    DispatchQueue.main.async { [weak self] in
      guard let self = self else { return }

      if #available(iOS 16.1, *) {
        self.endCurrentActivityIfAny()
      }
      invoke.resolve()
    }
  }

  // MARK: - Helpers

  @available(iOS 16.1, *)
  private func endCurrentActivityIfAny() {
    guard let activity = currentActivity as? Activity<IntradaActivityAttributes> else { return }
    // `Activity.end` requires a final ContentState even though the
    // activity dismisses immediately — reuse the last known state.
    let final = activity.contentState
    Task {
      await activity.end(using: final, dismissalPolicy: .immediate)
    }
    currentActivity = nil
  }

  private func parseRfc3339(_ s: String) -> Date? {
    if let d = Self.rfc3339WithFractional.date(from: s) { return d }
    return Self.rfc3339Plain.date(from: s)
  }
}

@_cdecl("init_plugin_live_activity")
func initPlugin() -> Plugin {
  return LiveActivityPlugin()
}
