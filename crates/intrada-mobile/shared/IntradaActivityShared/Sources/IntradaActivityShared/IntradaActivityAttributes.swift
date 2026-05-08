// IntradaActivityAttributes — the ActivityAttributes type that backs
// the Live Activity for the active practice session. Lives in a shared
// SwiftPM target because Swift requires identical type identity (not
// just structural matching) on both sides of the
// `Activity<T>` / `ActivityConfiguration<T>` boundary:
//
//   - Plugin (`tauri-plugin-live-activity`) calls
//     `Activity<IntradaActivityAttributes>.request(...)` to start /
//     update / end activities.
//   - Widget extension (`IntradaLiveActivity`) declares
//     `ActivityConfiguration(for: IntradaActivityAttributes.self)` to
//     provide the SwiftUI views WidgetKit renders on the Lock Screen +
//     Dynamic Island.
//
// Both targets depend on this shared package (see their Package.swift /
// project.yml entries).
//
// `ActivityAttributes` is iOS 16.1+; we gate the conformance with
// `@available` so this file compiles on older targets (the rest of the
// app supports iOS 14.0+) without forcing the whole shared target's
// minimum up.

#if canImport(ActivityKit)
  import ActivityKit
#endif
import Foundation

#if canImport(ActivityKit)

  @available(iOS 16.1, *)
  public struct IntradaActivityAttributes: ActivityAttributes {

    /// Mutable state pushed by the plugin on item advance / planned
    /// duration changes. Drives the SwiftUI views in the widget
    /// extension. All four fields must be `Codable + Hashable` per
    /// ActivityAttributes' contract.
    public struct ContentState: Codable, Hashable {
      /// Item title shown as the primary text on the lock-screen card
      /// and the Dynamic Island expanded view.
      public var itemTitle: String

      /// "Item N of M" — secondary text. Pre-formatted by the
      /// shell-side lifecycle Effect rather than the widget so we can
      /// share the format with `MPNowPlayingInfoCenter` (background
      /// audio).
      public var positionLabel: String

      /// Wall-clock anchor for elapsed-time math. The widget's SwiftUI
      /// views use `Text(timerInterval:)` and
      /// `ProgressView(timerInterval:)` so iOS handles the per-second
      /// update without us pushing IPC traffic. We only push on item
      /// advance.
      public var startedAt: Date

      /// Drives the progress arc / bar. `nil` when the entry has no
      /// planned duration (free-form practice) — widget falls back to
      /// elapsed-only rendering.
      public var plannedDurationSecs: UInt32?

      public init(
        itemTitle: String,
        positionLabel: String,
        startedAt: Date,
        plannedDurationSecs: UInt32? = nil
      ) {
        self.itemTitle = itemTitle
        self.positionLabel = positionLabel
        self.startedAt = startedAt
        self.plannedDurationSecs = plannedDurationSecs
      }
    }

    /// Static for the duration of the session. Currently empty —
    /// reserved for future session-level metadata (theme, instrument,
    /// goal text) that doesn't change item-to-item.
    public init() {}
  }

#endif
