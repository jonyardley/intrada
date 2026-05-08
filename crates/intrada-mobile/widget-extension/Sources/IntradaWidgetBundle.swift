import SwiftUI
import WidgetKit

/// Widget extension entry point.
///
/// Phase A (#474): placeholder bundle so the widget extension target
/// builds clean. Phase C swaps `IntradaPlaceholderWidget` for the
/// `ActivityWidget` that renders the Live Activity SwiftUI views (Lock
/// Screen card + Dynamic Island compact / expanded).
///
/// We need *something* in `body` because `WidgetBundle` requires at
/// least one widget for the bundle to be valid. The placeholder is a
/// home-screen widget that isn't surfaced to the user (no
/// `Configuration` flag enabling it as a home-screen option) — it's
/// purely there to keep the build green until ActivityKit lands.
@main
struct IntradaWidgetBundle: WidgetBundle {
    var body: some Widget {
        IntradaPlaceholderWidget()
    }
}

/// Phase A placeholder. Renders nothing useful; replaced by the
/// `ActivityWidget` in Phase C. See `specs/live-activity-plugin.md`.
struct IntradaPlaceholderWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: "intrada-placeholder",
            provider: IntradaPlaceholderProvider()
        ) { _ in
            Text("Intrada")
        }
        .configurationDisplayName("Intrada")
        .description("Phase A placeholder. Phase C adds the Live Activity.")
    }
}

struct IntradaPlaceholderProvider: TimelineProvider {
    func placeholder(in _: Context) -> IntradaPlaceholderEntry {
        IntradaPlaceholderEntry(date: Date())
    }

    func getSnapshot(in _: Context, completion: @escaping (IntradaPlaceholderEntry) -> Void) {
        completion(IntradaPlaceholderEntry(date: Date()))
    }

    func getTimeline(in _: Context, completion: @escaping (Timeline<IntradaPlaceholderEntry>) -> Void) {
        completion(Timeline(entries: [IntradaPlaceholderEntry(date: Date())], policy: .never))
    }
}

struct IntradaPlaceholderEntry: TimelineEntry {
    let date: Date
}
