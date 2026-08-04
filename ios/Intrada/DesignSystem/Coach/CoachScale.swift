import SwiftUI

/// The drill loop is read at two distances: a phone propped on the music desk,
/// and an iPad on a stand roughly a metre away. Every coach primitive takes one
/// of these rather than its own size arguments, so the whole loop grows together
/// — "the extra width buys size and air, not content" (`Drill Loop.dc.html`).
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

  // ── Type ──
  var drillTitle: CGFloat { self == .compact ? 30 : 74 }
  var question: CGFloat { self == .compact ? 36 : 64 }
  var tempo: CGFloat { self == .compact ? 52 : 76 }
  var eyebrow: CGFloat { self == .compact ? 11 : 14 }
  var clock: CGFloat { self == .compact ? 13 : 17 }
  var verdictDisc: CGFloat { self == .compact ? 118 : 168 }
}

private struct CoachScaleKey: EnvironmentKey {
  static let defaultValue = CoachScale.compact
}

extension EnvironmentValues {
  /// Set once by the drill screen from the horizontal size class, so a primitive
  /// nested anywhere in the loop scales without being handed the value.
  var coachScale: CoachScale {
    get { self[CoachScaleKey.self] }
    set { self[CoachScaleKey.self] = newValue }
  }
}
