import SwiftUI

/// The chrome every full-cover coach question wears: player paper, and a scroll
/// that engages only once the content outgrows the screen — the last control is
/// the only way off a cover with no interactive dismiss, so large text may never
/// push it out of reach. Extracted at the third copy (#1323). `DrillScreen` and
/// `OpenPlayScreen` are fixed layouts with nothing to scroll, and stay as they are.
struct CoachCoverScaffold<Content: View>: View {
  /// Handed the scale rather than reading it back out of the environment: the
  /// closure is composed in the caller's `body`, which runs before this sets it.
  @ViewBuilder var content: (CoachScale) -> Content

  @Environment(\.horizontalSizeClass) private var sizeClass

  private var scale: CoachScale { sizeClass == .regular ? .regular : .compact }
  private var gutter: CGFloat { scale == .compact ? IntradaSpacing.card : IntradaSpacing.stage }

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      GeometryReader { proxy in
        ScrollView {
          content(scale)
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
}
