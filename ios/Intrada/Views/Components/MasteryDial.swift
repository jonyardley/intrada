import SwiftUI

/// The overall-mastery ring. On appear the ring draws and the number counts up
/// together (ease-out over 1.5s); under Reduce Motion both snap to their final
/// value. `value` is the mean of the library's per-item 1–10 scores.
struct MasteryDial: View {
  let value: Double
  var maxValue: Double = 10
  var size: CGFloat = 128
  private let ringWidth: CGFloat = 9

  // Numeral and caption scale with Dynamic Type, a fixed ring does not, so they
  // crossed its stroke (#1471). Floored at 1, clamped so it still fits a phone.
  @ScaledMetric(relativeTo: .largeTitle) private var typeScale: CGFloat = 1
  private var resolvedSize: CGFloat { size * min(max(typeScale, 1), 1.6) }

  @Environment(\.dynamicTypeSize) private var dynamicTypeSize
  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Environment(\.intradaMotionDisabled) private var motionDisabled
  @State private var shown = false

  private var fraction: CGFloat { min(1, max(0, CGFloat(value / maxValue))) }
  private var animates: Bool {
    !reduceMotion && !motionDisabled && !UITestFlags.animationsDisabled
  }
  private var settled: Bool { shown || !animates }

  var body: some View {
    ZStack {
      Circle()
        .stroke(IntradaColor.dialTrack, lineWidth: ringWidth)
      Circle()
        .trim(from: 0, to: settled ? fraction : 0)
        .stroke(
          LinearGradient.ringSweep,
          style: StrokeStyle(lineWidth: ringWidth, lineCap: .round)
        )
        .rotationEffect(.degrees(-90))
      // Past the clamp the caption still grows, so inset it to the inscribed
      // square and let it shrink. Accessibility sizes only: below them, no change.
      VStack(spacing: 2) {
        CountingNumber(value: settled ? value : 0) { String(format: "%.1f", $0) }
          .font(IntradaFont.pageTitle(size * 0.297))
          .foregroundStyle(IntradaColor.ink)
        Text("of \(String(format: "%.1f", maxValue))".uppercased())
          .font(IntradaFont.eyebrow)
          .tracking(1.5)
          .foregroundStyle(IntradaColor.inkFaint)
      }
      .lineLimit(dynamicTypeSize.isAccessibilitySize ? 1 : nil)
      .minimumScaleFactor(dynamicTypeSize.isAccessibilitySize ? 0.5 : 1)
      .padding(.horizontal, dynamicTypeSize.isAccessibilitySize ? resolvedSize * 0.16 : 0)
    }
    .padding(ringWidth / 2)
    .frame(width: resolvedSize, height: resolvedSize)
    .onAppear {
      guard animates, !shown else { return }
      withAnimation(.easeOut(duration: IntradaMotion.countUpDuration)) { shown = true }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(
      "Overall mastery \(String(format: "%.1f", value)) of \(String(format: "%.1f", maxValue))")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      MasteryDial(value: 3.4)
    }
  }
#endif
