import SharedTypes
import SwiftUI

/// C1 — the feel moment, at a block boundary where feel is the point. One
/// question, three words and a first-class Skip; the core decides whether it is
/// asked at all, so this screen never appears twice for one block and never
/// alongside a gate.
///
/// No block counter and no clock: the core's prompt carries the target and
/// nothing else, and a position the shell worked out for itself would be the
/// shell claiming a domain fact.
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
      VStack(spacing: 0) {
        Spacer(minLength: IntradaSpacing.card)
        question
        Spacer(minLength: IntradaSpacing.card)
        CoachAction(title: "Skip", emphasis: .quiet, action: onSkip)
          .accessibilityHint("Leaves this block without a feel")
      }
      .padding(.horizontal, gutter)
      .padding(.bottom, IntradaSpacing.section)
    }
    .environment(\.coachScale, scale)
    .dynamicTypeSize(.xSmall ... .accessibility5)
  }

  private var question: some View {
    VStack(spacing: IntradaSpacing.section) {
      VStack(spacing: IntradaSpacing.controlGap) {
        // The target in the user's own words, from the core's join — the whole
        // frame is that name and the question under it.
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
      // Decision 17 in plain words: qualitative capture never feeds mastery, so
      // the screen says as much rather than leaving it to be assumed.
      Text("Kept with the session. It changes nothing about what comes next.")
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
