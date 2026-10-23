import SharedTypes
import SwiftUI

/// The section being played and its time left; once that runs out, the core's
/// move-on offer (#2315). The item's clock never stops, so ignoring it is fine.
struct SegmentClockRow: View {
  let segment: SegmentClockView
  /// Snapshots pass a fixed instant; production ticks off the wall clock.
  let referenceDate: Date?
  /// True while the move-on offer is up, so the screen can make room for it.
  @Binding var offering: Bool
  let onMove: () -> Void
  let onStay: () -> Void

  var body: some View {
    if let endsAt = SessionClock.parseRFC3339(segment.endsAt) {
      if let referenceDate {
        content(left: Int(endsAt.timeIntervalSince(referenceDate).rounded(.up)))
      } else {
        TimelineView(.periodic(from: .now, by: 1)) { context in
          content(left: Int(endsAt.timeIntervalSince(context.date).rounded(.up)))
        }
      }
    }
  }

  @ViewBuilder private func content(left: Int) -> some View {
    let offers = left <= 0 && segment.moveLabel != nil
    VStack(spacing: IntradaSpacing.controlGap) {
      HStack(spacing: IntradaSpacing.controlGap) {
        Text(segment.label)
          .font(IntradaFont.bodyMedium)
        Text("\(SessionClock.clockDisplay(left)) left")
          .font(IntradaFont.figure)
          .monospacedDigit()
          .foregroundStyle(IntradaColor.inkSecondary)
      }
      .foregroundStyle(IntradaColor.ink)
      .accessibilityElement(children: .combine)
      .accessibilityIdentifier("player.segment")

      sectionStrip

      if offers, let moveLabel = segment.moveLabel {
        BrandBarButton(action: onMove) {
          Text(moveLabel)
          Image(systemName: "arrow.right")
        }
        .accessibilityIdentifier("player.moveOn")
        if let stayLabel = segment.stayLabel {
          Button(action: onStay) {
            Text(stayLabel)
              .font(IntradaFont.bodyMedium)
              .foregroundStyle(IntradaColor.ink)
              .multilineTextAlignment(.center)
              .frame(maxWidth: .infinity, minHeight: 50)
              .padding(.horizontal, IntradaSpacing.cardCompact)
              .background(
                IntradaColor.cardFill, in: RoundedRectangle(cornerRadius: IntradaRadius.control)
              )
              .overlay(
                RoundedRectangle(cornerRadius: IntradaRadius.control)
                  .strokeBorder(IntradaColor.divider, lineWidth: 1))
          }
          .buttonStyle(.plain)
          .accessibilityIdentifier("player.stay")
        }
      }
    }
    .onChange(of: offers, initial: true) { _, next in offering = next }
  }

  /// Every section of the item, filled once done, with what Stay took (#2394).
  private var sectionStrip: some View {
    VStack(spacing: IntradaSpacing.segmentGap) {
      SegmentedProgress(
        count: segment.bars.count,
        filled: segment.bars.filter { $0.progress == .done }.count,
        label: segment.positionLabel)
      HStack(alignment: .top, spacing: IntradaSpacing.segmentGap) {
        ForEach(Array(segment.bars.enumerated()), id: \.offset) { _, bar in
          VStack(spacing: 0) {
            Text(bar.label)
              .lineLimit(1)
              .font(bar.progress == .running ? IntradaFont.smallMedium : IntradaFont.small)
              .foregroundStyle(
                bar.progress == .running ? IntradaColor.ink : IntradaColor.inkSecondary)
            if let caption = bar.caption {
              Text(caption)
                .lineLimit(1)
                .font(IntradaFont.small)
                .foregroundStyle(IntradaColor.inkSecondary)
            }
          }
          .frame(maxWidth: .infinity)
        }
      }
      .accessibilityHidden(true)
    }
  }
}
