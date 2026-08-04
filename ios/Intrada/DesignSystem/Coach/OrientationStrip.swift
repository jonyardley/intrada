import SharedTypes
import SwiftUI

/// Where you are, never how you did (spec decision 15). Time elapsed, this
/// block's ceiling and which block is running are always visible and carry no
/// judgement — so it is made quiet by being *small and static*, not by being
/// low-contrast. No ring or countdown: a filling track reads as a deadline.
struct OrientationStrip: View {
  let elapsedSeconds: Int
  var ceilingSeconds: Int?
  /// One entry per block in today's session, in order — the segment strip is
  /// tinted by each block's kind, so it doubles as the shape of the session.
  let blockKinds: [ItemKind]
  /// Zero-based index of the running block.
  let blockIndex: Int
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
        Text("BLOCK \(blockIndex + 1) OF \(blockCount)")
          .font(IntradaFont.ambient(scale.eyebrow).weight(.semibold))
          .tracking(1.5)
          .foregroundStyle(IntradaColor.inkSecondary)
          .lineLimit(1)
          .minimumScaleFactor(0.7)
        Spacer(minLength: 0)
        clock
      }
      // Orientation grows with type, but only so far: past this it wraps, and a
      // wrapped clock reads as an alarm rather than the quiet fact it is.
      .dynamicTypeSize(...(.accessibility1))
      SegmentedProgress(
        types: blockKinds, filled: min(blockIndex + 1, blockCount),
        height: scale == .compact ? 4 : 5)
    }
    .accessibilityElement(children: .contain)
  }

  private var clock: some View {
    HStack(spacing: 0) {
      Text(SessionClock.clockDisplay(elapsedSeconds))
        .fontWeight(.semibold)
      if let ceilingSeconds {
        // The ceiling sits back on weight, not contrast: `inkFaint` (2.9:1) is
        // banned everywhere in the loop, which is read at arm's length in a dim
        // practice room. Unpadded too — it's a duration ("6:00"), and "06:00"
        // beside a running "12:04" reads as a second clock.
        Text(" / \(Self.duration(ceilingSeconds))")
      }
    }
    .font(IntradaFont.ambient(scale.clock))
    .foregroundStyle(IntradaColor.inkSecondary)
    .monospacedDigit()
    .lineLimit(1)
    .fixedSize(horizontal: true, vertical: false)
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(accessibilityClock)
  }

  private static func duration(_ seconds: Int) -> String {
    String(format: "%d:%02d", seconds / 60, seconds % 60)
  }

  private var accessibilityClock: String {
    let elapsed = "\(SessionClock.clockDisplay(elapsedSeconds)) elapsed"
    guard let ceilingSeconds else { return elapsed }
    return "\(elapsed) of \(Self.duration(ceilingSeconds))"
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
        Spacer()
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
