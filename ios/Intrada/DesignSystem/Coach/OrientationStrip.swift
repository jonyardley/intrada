import SharedTypes
import SwiftUI

/// Where you are, never how you did (spec decision 15). Quiet by being small
/// and static, not low-contrast; no ring or countdown, since a filling track
/// reads as a deadline.
struct OrientationStrip: View {
  /// `nil` on a block that has not run yet — a zeroed clock is noise, not
  /// orientation.
  let elapsedSeconds: Int?
  var ceilingSeconds: Int?
  /// One entry per block in today's session, in order. Empty at an altitude,
  /// which has no plan behind it to draw a track for.
  let blockKinds: [ItemKind]
  /// Zero-based index of the running block.
  let blockIndex: Int
  /// Set at one of Journey B's three altitudes: the chip takes the eyebrow's
  /// place, so what is being recorded is readable at the same glance as the
  /// clock, for the whole run (decision 16).
  var altitude: Altitude?
  /// The chevron that stops the drill — one gesture, no resident chrome.
  var onDismiss: (() -> Void)?

  @Environment(\.coachScale) private var scale

  private var blockCount: Int { blockKinds.count }

  var body: some View {
    VStack(spacing: 14) {
      HStack(spacing: IntradaSpacing.cardCompact) {
        if let onDismiss {
          Button(action: onDismiss) {
            Image(systemName: "chevron.down")
              .font(.system(size: scale == .compact ? 20 : 24, weight: .medium))
              .foregroundStyle(IntradaColor.inkSecondary)
              .frame(width: 44, height: 44)
              .contentShape(Rectangle())
          }
          .buttonStyle(PressRebound())
          .accessibilityLabel("Stop the drill")
        }
        Spacer(minLength: 0)
        eyebrow
        Spacer(minLength: 0)
        clock
      }
      // Past this it wraps, and a wrapped clock reads as an alarm.
      .dynamicTypeSize(...(.accessibility1))
      if blockCount > 0 {
        SegmentedProgress(
          types: blockKinds, filled: min(blockIndex + 1, blockCount),
          height: scale == .compact ? 4 : 5
        )
        // The eyebrow already says it, and "Item" is the wrong noun for a block.
        .accessibilityHidden(true)
      }
    }
    .accessibilityElement(children: .contain)
  }

  @ViewBuilder private var eyebrow: some View {
    if let altitude {
      AltitudeChip(altitude: altitude)
    } else {
      Text(blockCount > 0 ? "BLOCK \(blockIndex + 1) OF \(blockCount)" : "")
        .font(IntradaFont.ambientStrong(scale.eyebrow))
        .tracking(1.5)
        .foregroundStyle(IntradaColor.inkSecondary)
        .lineLimit(1)
        .minimumScaleFactor(0.7)
    }
  }

  @ViewBuilder private var clock: some View {
    if let elapsedSeconds { clockFace(elapsedSeconds) }
  }

  private func clockFace(_ elapsedSeconds: Int) -> some View {
    HStack(spacing: 0) {
      Text(Self.duration(elapsedSeconds))
        .font(IntradaFont.ambientStrong(scale.clock))
      if let ceilingSeconds {
        // Back on weight, not contrast: `inkFaint` (2.9:1) is banned in the
        // loop, read at arm's length in a dim room.
        Text(" / \(Self.duration(ceilingSeconds))")
      }
    }
    .font(IntradaFont.ambient(scale.clock))
    .foregroundStyle(IntradaColor.inkSecondary)
    .monospacedDigit()
    .lineLimit(1)
    .fixedSize(horizontal: true, vertical: false)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityClock(elapsedSeconds))
  }

  private static func duration(_ seconds: Int) -> String {
    String(format: "%d:%02d", seconds / 60, seconds % 60)
  }

  private func accessibilityClock(_ elapsedSeconds: Int) -> String {
    // An altitude has no block and no ceiling, so "spent on this block" would
    // name something that is not on screen.
    let subject = altitude == nil ? " spent on this block" : " so far"
    let elapsed = Self.duration(elapsedSeconds) + subject
    guard let ceilingSeconds else { return elapsed }
    return "\(elapsed), of \(Self.duration(ceilingSeconds))"
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: IntradaSpacing.stage) {
        OrientationStrip(
          elapsedSeconds: 724, ceilingSeconds: 360,
          blockKinds: [.piece, .exercise, .exercise, .piece, .piece],
          blockIndex: 1, onDismiss: {})
        OrientationStrip(
          elapsedSeconds: 192,
          blockKinds: [.exercise, .piece, .piece],
          blockIndex: 0)
        // A block that has not run yet: no clock at all.
        OrientationStrip(
          elapsedSeconds: nil,
          blockKinds: [.exercise, .piece, .piece],
          blockIndex: 0, onDismiss: {})
        // The three altitudes: no plan track, the chip in the eyebrow's place.
        OrientationStrip(
          elapsedSeconds: 252, blockKinds: [], blockIndex: 0,
          altitude: .runThrough, onDismiss: {})
        OrientationStrip(
          elapsedSeconds: 698, blockKinds: [], blockIndex: 0,
          altitude: .offPiste, onDismiss: {})
        OrientationStrip(
          elapsedSeconds: 1380, blockKinds: [], blockIndex: 0,
          altitude: .unmonitored, onDismiss: {})
        Spacer()
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
