// Phase C of #474 — Live Activity widget bundle entry point.
//
// The widget extension target's deployment is iOS 16.1+ (set by
// `add-live-activity-target.rb` via `IPHONEOS_DEPLOYMENT_TARGET`), so
// `IntradaActivityWidget` is unconditionally available — no
// `@available` fallback needed at the bundle level. WidgetBundleBuilder
// doesn't support `if` control flow anyway, so this is also the only
// shape the result builder accepts.
//
// Spec: specs/live-activity-plugin.md

import SwiftUI
import WidgetKit

@main
struct IntradaWidgetBundle: WidgetBundle {
  var body: some Widget {
    IntradaActivityWidget()
  }
}
