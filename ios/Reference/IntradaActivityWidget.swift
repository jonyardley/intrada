// Phase C of #474 — SwiftUI views for the Live Activity.
//
// Three surfaces, one ActivityConfiguration:
//   - Lock Screen / banner: full card with item title, position,
//     elapsed time, and progress bar.
//   - Dynamic Island compact: progress arc + elapsed time. Width is
//     ~20pt + leading symbol; we lean on a circular ProgressView with
//     a tiny inset elapsed time for at-a-glance reading.
//   - Dynamic Island expanded: same primary content as the lock
//     screen card but laid out for the wider expanded region.
//   - Dynamic Island minimal: single icon (the user is in the Dynamic
//     Island fast-switcher).
//
// All elapsed-time / progress views use `Text(timerInterval:)` and
// `ProgressView(timerInterval:)` so iOS handles per-second updates
// without IPC traffic. We push only on item advance.
//
// Spec: specs/live-activity-plugin.md

#if canImport(ActivityKit)
  import ActivityKit
#endif
import IntradaActivityShared
import SwiftUI
import WidgetKit

@available(iOS 16.1, *)
struct IntradaActivityWidget: Widget {
  var body: some WidgetConfiguration {
    ActivityConfiguration(for: IntradaActivityAttributes.self) { context in
      // ── Lock Screen / banner view ────────────────────────────────
      LockScreenView(state: context.state)
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .activityBackgroundTint(Color.black.opacity(0.85))
        .activitySystemActionForegroundColor(Color.white)
    } dynamicIsland: { context in
      DynamicIsland {
        // ── Expanded ── (regions: leading, trailing, center, bottom)
        DynamicIslandExpandedRegion(.leading) {
          Image(systemName: "music.note")
            .foregroundStyle(.tint)
            .font(.title2)
        }
        DynamicIslandExpandedRegion(.trailing) {
          ElapsedTimeText(startedAt: context.state.startedAt)
            .font(.title3.monospacedDigit())
            .foregroundStyle(.primary)
        }
        DynamicIslandExpandedRegion(.center) {
          VStack(alignment: .leading, spacing: 2) {
            Text(context.state.itemTitle)
              .font(.headline)
              .lineLimit(1)
            Text(context.state.positionLabel)
              .font(.caption)
              .foregroundStyle(.secondary)
              .lineLimit(1)
          }
        }
        DynamicIslandExpandedRegion(.bottom) {
          if let planned = context.state.plannedDurationSecs, planned > 0 {
            ProgressView(
              timerInterval: context.state.startedAt...endDate(
                from: context.state.startedAt, addingSecs: planned),
              countsDown: false,
              label: { EmptyView() },
              currentValueLabel: { EmptyView() }
            )
            .tint(.indigo)
          }
        }
      } compactLeading: {
        // Progress arc when planned duration is set; clock icon
        // otherwise. The arc gives a glanceable "how far through this
        // item am I" signal without any text.
        if let planned = context.state.plannedDurationSecs, planned > 0 {
          CompactArc(
            startedAt: context.state.startedAt,
            plannedDurationSecs: planned
          )
          .frame(width: 18, height: 18)
        } else {
          Image(systemName: "music.note")
            .foregroundStyle(.tint)
        }
      } compactTrailing: {
        ElapsedTimeText(startedAt: context.state.startedAt)
          .font(.caption.monospacedDigit())
          .foregroundStyle(.primary)
      } minimal: {
        Image(systemName: "music.note")
          .foregroundStyle(.tint)
      }
      .keylineTint(.indigo)
    }
  }
}

// MARK: - Lock Screen view

@available(iOS 16.1, *)
struct LockScreenView: View {
  let state: IntradaActivityAttributes.ContentState

  var body: some View {
    VStack(alignment: .leading, spacing: 8) {
      HStack(alignment: .firstTextBaseline) {
        VStack(alignment: .leading, spacing: 2) {
          Text(state.itemTitle)
            .font(.headline)
            .lineLimit(1)
          Text(state.positionLabel)
            .font(.subheadline)
            .foregroundStyle(.secondary)
            .lineLimit(1)
        }
        Spacer(minLength: 12)
        ElapsedTimeText(startedAt: state.startedAt)
          .font(.title2.monospacedDigit())
          .foregroundStyle(.primary)
      }

      if let planned = state.plannedDurationSecs, planned > 0 {
        ProgressView(
          timerInterval: state.startedAt...endDate(from: state.startedAt, addingSecs: planned),
          countsDown: false,
          label: { EmptyView() },
          currentValueLabel: { EmptyView() }
        )
        .progressViewStyle(.linear)
        .tint(.indigo)
      }
    }
  }
}

// MARK: - Shared time helpers

/// Auto-updating elapsed-time text — iOS handles per-second redraws
/// internally when the view uses `Text(timerInterval:)`. The activity
/// only pushes new state on item advance, not per second.
@available(iOS 16.0, *)
struct ElapsedTimeText: View {
  let startedAt: Date
  var body: some View {
    // Open-ended interval so the text counts up indefinitely from
    // `startedAt`. iOS uses the system clock to refresh.
    Text(timerInterval: startedAt...Date.distantFuture, countsDown: false)
  }
}

/// Tiny circular progress arc for the Dynamic Island compact-leading
/// region. Same auto-updating story as `ElapsedTimeText`.
@available(iOS 16.1, *)
struct CompactArc: View {
  let startedAt: Date
  let plannedDurationSecs: UInt32

  var body: some View {
    ProgressView(
      timerInterval: startedAt...endDate(from: startedAt, addingSecs: plannedDurationSecs),
      countsDown: false,
      label: { EmptyView() },
      currentValueLabel: { EmptyView() }
    )
    .progressViewStyle(.circular)
    .tint(.indigo)
  }
}

/// Helper: compute end-Date by adding `secs` to `startedAt`. Pulled out
/// because both lock-screen and Dynamic Island ProgressViews compute it
/// the same way.
@available(iOS 16.0, *)
fileprivate func endDate(from startedAt: Date, addingSecs secs: UInt32) -> Date {
  startedAt.addingTimeInterval(TimeInterval(secs))
}
