import SharedTypes
import SwiftUI

/// The last frame of a prescribed session (#1323). Every word is the core's:
/// what a session that stopped at minute eight is worth is a domain judgement.
struct SoftLandingScreen: View {
  let state: LandingView
  var onDone: () -> Void

  var body: some View {
    CoachCoverScaffold { scale in
      VStack(spacing: 0) {
        Spacer(minLength: IntradaSpacing.card)
        landing(scale)
        Spacer(minLength: IntradaSpacing.card)
        CoachAction(title: "Done", emphasis: .primary, action: onDone)
      }
    }
  }

  private func landing(_ scale: CoachScale) -> some View {
    VStack(spacing: IntradaSpacing.section) {
      Text(state.headline)
        .font(IntradaFont.verdict(scale.ask))
        .foregroundStyle(IntradaColor.ink)
        .multilineTextAlignment(.center)
        .fixedSize(horizontal: false, vertical: true)
      if let detail = state.detail {
        Text(detail)
          .font(IntradaFont.ambientStrong(scale == .compact ? 17 : 22))
          .foregroundStyle(IntradaColor.ink)
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
      }
      if let note = state.note {
        Text(note)
          .font(IntradaFont.ambient(scale.support))
          .foregroundStyle(IntradaColor.inkSecondary)
          .multilineTextAlignment(.center)
          .fixedSize(horizontal: false, vertical: true)
      }
    }
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
