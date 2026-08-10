import SharedTypes
import SwiftUI

/// Which altitude is running, and therefore what is being recorded (decision
/// 16). Visible for the whole run, because absence of instrumentation is what
/// the user consented to — a chip that came and went would make the contract
/// something they had to remember rather than something they can read.
struct AltitudeChip: View {
  let altitude: Altitude

  @Environment(\.coachScale) private var scale

  var body: some View {
    HStack(spacing: 5) {
      Image(systemName: glyph)
        .font(.system(size: scale.eyebrow + 1, weight: .semibold))
      Text(label)
        .font(IntradaFont.ambientStrong(scale.eyebrow))
        .tracking(1.5)
        .lineLimit(1)
        .minimumScaleFactor(0.7)
    }
    .foregroundStyle(foreground)
    .padding(.horizontal, IntradaSpacing.controlGap)
    .padding(.vertical, 5)
    .background(background, in: Capsule())
    .accessibilityElement(children: .ignore)
    .accessibilityLabel(spoken)
  }

  /// Two clauses, not one: the name alone does not say what it costs, and the
  /// contract is the half the user is actually agreeing to.
  private var label: String {
    switch altitude {
    case .runThrough: "RUN-THROUGH · COUNTS"
    case .offPiste: "OFF-PISTE · TIME ONLY"
    case .unmonitored: "OFF THE RECORD"
    }
  }

  private var spoken: String {
    switch altitude {
    case .runThrough: "Run-through. Every section counts."
    case .offPiste: "Off-piste. Time is logged, nothing is scored."
    case .unmonitored: "Off the record. Minutes only, nothing is recorded."
    }
  }

  private var glyph: String {
    switch altitude {
    case .runThrough: "checkmark.circle"
    case .offPiste: "safari"
    case .unmonitored: "eye.slash"
    }
  }

  private var foreground: Color {
    switch altitude {
    case .runThrough: IntradaColor.pieceBadgeFg
    case .offPiste: IntradaColor.exerciseBadgeFg
    case .unmonitored: IntradaColor.inkSecondary
    }
  }

  private var background: Color {
    switch altitude {
    case .runThrough: IntradaColor.pieceBadgeBg
    case .offPiste: IntradaColor.exerciseBadgeBg
    case .unmonitored: IntradaColor.journalBadgeBg
    }
  }
}

#if DEBUG
  #Preview {
    ZStack {
      RadialGradient.playerPaper.ignoresSafeArea()
      VStack(spacing: IntradaSpacing.card) {
        AltitudeChip(altitude: .runThrough)
        AltitudeChip(altitude: .offPiste)
        AltitudeChip(altitude: .unmonitored)
        AltitudeChip(altitude: .runThrough)
          .environment(\.coachScale, .regular)
      }
    }
  }
#endif
