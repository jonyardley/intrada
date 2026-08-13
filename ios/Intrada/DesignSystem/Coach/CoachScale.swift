import SwiftUI

/// The two distances the loop is read at: a phone on the music desk, an iPad on
/// a stand about a metre away. Every coach primitive sizes off this rather than
/// its own arguments, so the whole loop grows together.
enum CoachScale {
  case compact
  case regular

  // ── GateDots ──
  var dot: CGFloat { self == .compact ? 17 : 24 }
  var dotGap: CGFloat { self == .compact ? 8 : 12 }
  var dotCaption: CGFloat { self == .compact ? 15 : 21 }

  // ── BeatPosition ──
  var pip: CGFloat { self == .compact ? 15 : 21 }
  var pipGap: CGFloat { self == .compact ? 9 : 13 }
  var pipCaption: CGFloat { self == .compact ? 13 : 19 }

  // ── The one control size on the keys ──
  var target: CGFloat { self == .compact ? 66 : 96 }
  var targetRadius: CGFloat { self == .compact ? IntradaRadius.panel : IntradaRadius.hero }
  var targetLabel: CGFloat { self == .compact ? 16 : 24 }
  /// A3's demoted stuck row.
  var quietTarget: CGFloat { self == .compact ? 46 : 64 }
  var quietLabel: CGFloat { self == .compact ? 14 : 18 }

  // ── Type ──
  var drillTitle: CGFloat { self == .compact ? 30 : 74 }
  var question: CGFloat { self == .compact ? 36 : 64 }
  var tempo: CGFloat { self == .compact ? 52 : 76 }
  var eyebrow: CGFloat { self == .compact ? 11 : 14 }
  /// A full-cover question: read, rather than answered under the hands.
  var ask: CGFloat { question * 0.78 }
  /// The quiet line under it: what the ask costs, or what it doesn't.
  var support: CGFloat { self == .compact ? 14 : 18 }
  var clock: CGFloat { self == .compact ? 13 : 17 }
  var verdictDisc: CGFloat { self == .compact ? 118 : 168 }
}

private struct CoachScaleKey: EnvironmentKey {
  static let defaultValue = CoachScale.compact
}

extension EnvironmentValues {
  /// Set once by the drill screen from the horizontal size class.
  var coachScale: CoachScale {
    get { self[CoachScaleKey.self] }
    set { self[CoachScaleKey.self] = newValue }
  }
}
