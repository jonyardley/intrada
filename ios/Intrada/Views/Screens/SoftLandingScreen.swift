import SharedTypes
import SwiftUI

/// The last frame of a prescribed session (#1323). Every word is the core's:
/// what a session that stopped at minute eight is worth is a domain judgement.
struct SoftLandingScreen: View {
  let state: LandingView
  var onDone: () -> Void

  @Environment(\.horizontalSizeClass) private var sizeClass

  private var scale: CoachScale { sizeClass == .regular ? .regular : .compact }
  private var gutter: CGFloat { scale == .compact ? IntradaSpacing.card : IntradaSpacing.stage }

  var body: some View {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      // Scrolls once the text outgrows the screen: Done is the only way past
      // this, so it may never be the thing that falls off the bottom (FeelScreen).
      GeometryReader { proxy in
        ScrollView {
          VStack(spacing: 0) {
            Spacer(minLength: IntradaSpacing.card)
            landing
            Spacer(minLength: IntradaSpacing.card)
            CoachAction(title: "Done", emphasis: .primary, action: onDone)
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

  private var landing: some View {
    VStack(spacing: IntradaSpacing.section) {
      Text(state.headline)
        .font(IntradaFont.verdict(scale.question * 0.78))
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)
      if let detail = state.detail {
        Text(detail)
          .font(IntradaFont.ambientStrong(scale == .compact ? 17 : 22))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
          .accessibilityLabel(spoken(detail))
      }
      if let note = state.note {
        Text(note)
          .font(IntradaFont.ambient(scale == .compact ? 14 : 18))
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
      }
    }
  }

  /// Spoken, the detail's clauses run together without its full stops.
  private func spoken(_ line: String) -> String {
    line.replacingOccurrences(of: ".", with: ",")
  }
}

#if DEBUG
  extension LandingView {
    /// Verbatim from `CoachState::landing_view`, so a preview reads what the
    /// device would show.
    static var previewShort: LandingView {
      LandingView(
        headline: "That's banked.", detail: "2 blocks, 12 minutes. 1 gate passed.",
        note: "Short is still practice, and it's on the record.")
    }

    static var previewFinished: LandingView {
      LandingView(
        headline: "That's the session.", detail: "3 blocks, 24 minutes. 3 gates passed.",
        note: nil)
    }

    static var previewNothingPlayed: LandingView {
      LandingView(
        headline: "Another time.", detail: nil,
        note: "Nothing played, so nothing's on the record. Today's plan is still there.")
    }
  }

  #Preview("Ended early") {
    SoftLandingScreen(state: .previewShort, onDone: {})
  }

  #Preview("Plan finished") {
    SoftLandingScreen(state: .previewFinished, onDone: {})
  }

  #Preview("Nothing played") {
    SoftLandingScreen(state: .previewNothingPlayed, onDone: {})
  }

  #Preview("Largest accessibility size") {
    SoftLandingScreen(state: .previewShort, onDone: {})
      .environment(\.dynamicTypeSize, .accessibility5)
  }
#endif
