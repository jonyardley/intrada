// Phase B of intrada#474 — placeholder Swift plugin. Resolves all three
// commands (begin / update / end) without touching ActivityKit yet, so
// the IPC roundtrip from JS → Rust → Swift can be exercised end-to-end.
// Phase C swaps these stubs for the real Activity<...>.request /
// activity.update / activity.end calls.
//
// Spec: specs/live-activity-plugin.md

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
  // Phase B: parse args to confirm shape, then resolve. Phase C
  // replaces each stub body with the corresponding ActivityKit call.

  @objc public func begin(_ invoke: Invoke) throws {
    let _ = try invoke.parseArgs(BeginArgs.self)
    invoke.resolve()
  }

  @objc public func update(_ invoke: Invoke) throws {
    let _ = try invoke.parseArgs(UpdateArgs.self)
    invoke.resolve()
  }

  @objc public func end(_ invoke: Invoke) {
    invoke.resolve()
  }
}

@_cdecl("init_plugin_live_activity")
func initPlugin() -> Plugin {
  return LiveActivityPlugin()
}
