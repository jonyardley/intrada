import SharedTypes
import SwiftUI

/// Minutes used against the session length: the core's summary and state line,
/// and a bar with the share past the length in `danger`.
struct MinutesUsedMeter: View {
  let summary: String
  let stateLine: String
  let usedMins: Int
  let lengthMins: Int
  let state: FitState

  var body: some View {
    VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      ViewThatFits(in: .horizontal) {
        HStack(alignment: .firstTextBaseline) {
          summaryLabel
          Spacer(minLength: IntradaSpacing.controlGap)
          stateLabel
        }
        VStack(alignment: .leading, spacing: IntradaSpacing.xxSmall) {
          summaryLabel
          stateLabel
        }
      }
      .accessibilityElement(children: .combine)
      MinutesUsedTrack(usedMins: usedMins, lengthMins: lengthMins)
    }
  }

  private var isOver: Bool { state == .over }

  private var summaryLabel: some View {
    Text(summary)
      .font(IntradaFont.label)
      .monospacedDigit()
      .foregroundStyle(IntradaColor.ink)
  }

  private var stateLabel: some View {
    HStack(spacing: IntradaSpacing.xSmall) {
      if isOver {
        Image(systemName: "exclamationmark.circle")
          .iconSize(.badge, weight: .semibold)
          .accessibilityHidden(true)
      }
      Text(stateLine)
        .font(isOver ? IntradaFont.label : IntradaFont.secondary)
        .monospacedDigit()
    }
    .foregroundStyle(isOver ? IntradaColor.danger : IntradaColor.inkSecondary)
  }
}

private struct MinutesUsedTrack: View {
  let usedMins: Int
  let lengthMins: Int

  var body: some View {
    GeometryReader { proxy in
      let total = CGFloat(max(usedMins, lengthMins, 1))
      let inside = CGFloat(min(usedMins, lengthMins)) / total * proxy.size.width
      let over = CGFloat(max(usedMins - lengthMins, 0)) / total * proxy.size.width
      HStack(spacing: IntradaSpacing.xxSmall) {
        Capsule().fill(IntradaColor.ink).frame(width: inside)
        if over > 0 {
          Capsule().fill(IntradaColor.danger)
            .frame(width: max(over - IntradaSpacing.xxSmall, 0))
        }
        Spacer(minLength: 0)
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      .background(IntradaColor.track, in: Capsule())
    }
    .frame(height: 6)
    .accessibilityHidden(true)
  }
}

#if DEBUG
  #Preview("Minutes used meter") {
    VStack(spacing: IntradaSpacing.section) {
      MinutesUsedMeter(
        summary: "72 of 60 min", stateLine: "12 min over", usedMins: 72, lengthMins: 60,
        state: .over)
      MinutesUsedMeter(
        summary: "15 of 60 min, plus 2 untimed", stateLine: "45 min left", usedMins: 15,
        lengthMins: 60, state: .under)
      MinutesUsedMeter(
        summary: "60 of 60 min", stateLine: "Fits exactly", usedMins: 60, lengthMins: 60,
        state: .exact)
    }
    .padding(IntradaSpacing.section)
    .background(IntradaColor.paperTop)
  }
#endif
