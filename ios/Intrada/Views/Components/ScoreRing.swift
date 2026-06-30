import SwiftUI

struct ScoreRing: View {
  let score: Int?
  var size: CGFloat = 46

  @Environment(\.accessibilityReduceMotion) private var reduceMotion
  @Environment(\.intradaMotionDisabled) private var motionDisabled
  @State private var shown = false

  private var isUnrated: Bool { score == nil || score == 0 }
  private var clampedScore: Int { score.map { max(1, min(10, $0)) } ?? 0 }
  private var fraction: CGFloat { CGFloat(clampedScore) / 10 }
  private var lineWidth: CGFloat { max(3, size * 0.09) }

  private var animates: Bool {
    !reduceMotion && !motionDisabled && !UITestFlags.animationsDisabled
  }
  private var settled: Bool { shown || !animates }

  var body: some View {
    ZStack {
      Circle()
        .stroke(IntradaColor.masteryTrack, lineWidth: lineWidth)
      if !isUnrated {
        Circle()
          .trim(from: 0, to: settled ? fraction : 0)
          .stroke(
            IntradaColor.masteryFill,
            style: StrokeStyle(lineWidth: lineWidth, lineCap: .round))
          .rotationEffect(.degrees(-90))
      }
      if isUnrated {
        Text("–")
          .font(IntradaFont.pageTitle(size * 0.36))
          .foregroundStyle(IntradaColor.inkFaint)
      } else {
        Text("\(clampedScore)")
          .font(IntradaFont.pageTitle(size * 0.36))
          .foregroundStyle(IntradaColor.ink)
      }
    }
    .padding(lineWidth / 2)
    .frame(width: size, height: size)
    .onAppear {
      guard animates, !shown else { return }
      withAnimation(.easeOut(duration: IntradaMotion.countUpDuration)) { shown = true }
    }
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(isUnrated ? "Not yet rated" : "Score \(clampedScore) of 10")
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      HStack(spacing: 16) {
        ForEach([0, 1, 4, 7, 10], id: \.self) { ScoreRing(score: $0) }
        ScoreRing(score: nil)
      }
      .padding(IntradaSpacing.card)
    }
  }
#endif
