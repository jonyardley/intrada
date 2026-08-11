import SharedTypes
import SwiftUI

/// C1 — the feel moment. No block counter and no clock: the prompt carries the
/// target and nothing else, and a position the shell worked out for itself
/// would be the shell claiming a domain fact (#1315).
struct FeelScreen: View {
  let prompt: FeelPrompt
  var onFeel: (Feel) -> Void
  var onSkip: () -> Void

  @Environment(\.horizontalSizeClass) private var sizeClass

  private var scale: CoachScale { sizeClass == .regular ? .regular : .compact }
  private var gutter: CGFloat { scale == .compact ? IntradaSpacing.card : IntradaSpacing.stage }

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      // Scrolls once the text outgrows the screen: Skip is the only way past
      // this question, so it may never be the thing that falls off the bottom.
      GeometryReader { proxy in
        ScrollView {
          VStack(spacing: 0) {
            Spacer(minLength: IntradaSpacing.card)
            question
            Spacer(minLength: IntradaSpacing.card)
            CoachAction(title: "Skip", emphasis: .quiet, action: onSkip)
              .accessibilityHint("Leaves this block without a feel")
          }
          .padding(.horizontal, gutter)
          .padding(.bottom, IntradaSpacing.section)
          .frame(minHeight: proxy.size.height)
        }
        .scrollBounceBehavior(.basedOnSize)
      }
    }
    .environment(\.coachScale, scale)
    .dynamicTypeSize(.xSmall ... .accessibility5)
  }

  private var question: some View {
    VStack(spacing: IntradaSpacing.section) {
      VStack(spacing: IntradaSpacing.controlGap) {
        Text(prompt.title)
          .font(IntradaFont.ambientStrong(scale.eyebrow))
          .tracking(1.2)
          .textCase(.uppercase)
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
        Text("How did it feel?")
          .font(IntradaFont.verdict(scale.question * 0.78))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
      }
      FeelChips(onFeel: onFeel)
      // Deliberately not "changes nothing": decision 17 lets a feel retire a
      // target one day, and only the gate is ruled out here.
      Text("Kept with the session, and never scored against a gate.")
        .font(IntradaFont.ambient(scale == .compact ? 13 : 17))
        .foregroundStyle(IntradaColor.inkSecondary)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)
    }
  }
}

#if DEBUG
  private struct FeelPreview: View {
    var title = "Freer rubato in the intro"
    var body: some View {
      FeelScreen(
        prompt: FeelPrompt(blockId: "01BLOCK000000000000000003", title: title),
        onFeel: { _ in }, onSkip: {})
    }
  }

  #Preview("Feel moment") { FeelPreview() }

  #Preview("Largest accessibility size") {
    FeelPreview().environment(\.dynamicTypeSize, .accessibility5)
  }
#endif
