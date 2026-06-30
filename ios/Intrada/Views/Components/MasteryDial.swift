import SwiftUI

/// The overall-mastery ring. On appear the ring draws and the number counts up
/// together (ease-out over 1.5s); under Reduce Motion both snap to their final
/// value. `value` is the mean of the library's per-item 1–10 scores.
struct MasteryDial: View {
  let value: Double
  var maxValue: Double = 10
  var size: CGFloat = 128
  private let ringWidth: CGFloat = 9

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
          style: StrokeStyle(lineWidth: ringWidth, lineCap: .round))
        .rotationEffect(.degrees(-90))
      VStack(spacing: 2) {
        CountingNumber(value: settled ? value : 0) { String(format: "%.1f", $0) }
          .font(IntradaFont.pageTitle(size * 0.297))
          .foregroundStyle(IntradaColor.ink)
        Text("of \(String(format: "%.1f", maxValue))".uppercased())
          .font(IntradaFont.eyebrow)
          .tracking(1.5)
          .foregroundStyle(IntradaColor.inkFaint)
      }
    }
    .padding(ringWidth / 2)
    .frame(width: size, height: size)
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
