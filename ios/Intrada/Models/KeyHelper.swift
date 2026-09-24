import Foundation
import SharedTypes

enum KeyHelper {
  struct Selection: Equatable {
    let ring: Int
    let mode: Modality
    let spelling: String
  }

  /// Major keys clockwise from 12 o'clock; 6 o'clock defaults to Gb over F#.
  static let circleMajor = ["C", "G", "D", "A", "E", "B", "Gb", "Db", "Ab", "Eb", "Bb", "F"]
  /// Relative minors, same spoke order; 6 o'clock defaults to Eb over D#.
  static let circleMinor = ["A", "E", "B", "F#", "C#", "G#", "Eb", "Bb", "F", "C", "G", "D"]

  static func primary(ring: Int, mode: Modality) -> String {
    switch mode {
    case .major: return circleMajor[ring]
    case .minor: return circleMinor[ring]
    }
  }

  static func enharmonicAlt(ring: Int, mode: Modality) -> String? {
    switch (mode, ring) {
    case (.major, 5): return "Cb"
    case (.major, 6): return "F#"
    case (.major, 7): return "C#"
    case (.minor, 5): return "Ab"
    case (.minor, 6): return "D#"
    case (.minor, 7): return "A#"
    default: return nil
    }
  }

  static func selection(key: String, modality: Modality?) -> Selection? {
    guard let modality else { return nil }
    return ringFor(tonic: key, mode: modality)
  }

  /// Tapping the already-selected enharmonic spoke flips its spelling; any
  /// other tap selects that spoke's default.
  static func nextOnTap(
    currentKey: String, currentModality: Modality?, ring: Int, mode: Modality
  ) -> (tonic: String, modality: Modality, flipped: Bool) {
    let prim = primary(ring: ring, mode: mode)
    if let sel = selection(key: currentKey, modality: currentModality), sel.ring == ring,
      sel.mode == mode, let alt = enharmonicAlt(ring: ring, mode: mode)
    {
      let other = sel.spelling == prim ? alt : prim
      return (other, mode, true)
    }
    return (prim, mode, false)
  }

  /// `#`→`♯`; a `b` only counts as `♭` when it follows a note letter (so mode
  /// words like "minor" are left untouched).
  static func prettify(_ value: String) -> String {
    var out = ""
    var prev: Character?
    for c in value {
      if c == "#" {
        out.append("\u{266F}")
      } else if c == "b", let p = prev, ("A"..."G").contains(p) {
        out.append("\u{266D}")
      } else {
        out.append(c)
      }
      prev = c
    }
    return out
  }

  static func modeWord(_ mode: Modality) -> String {
    switch mode {
    case .major: return "major"
    case .minor: return "minor"
    }
  }

  static func display(key: String?, modality: Modality?) -> String? {
    guard let key, !key.isEmpty else { return nil }
    if let modality {
      return "\(prettify(key)) \(modeWord(modality))"
    }
    return prettify(key)
  }

  /// Enharmonic spokes announce both spellings, since one tap selects and a
  /// second flips between them.
  static func wedgeAccessibilityLabel(ring: Int, mode: Modality) -> String {
    let prim = primary(ring: ring, mode: mode)
    if let alt = enharmonicAlt(ring: ring, mode: mode) {
      return "\(spokenTonic(prim)) or \(spokenTonic(alt)) \(modeWord(mode))"
    }
    return accessibilityLabel(prim, mode: mode)
  }

  static func accessibilityLabel(_ tonic: String, mode: Modality) -> String {
    "\(spokenTonic(tonic)) \(modeWord(mode))"
  }

  // ── Internal ──

  private static func ringFor(tonic: String, mode: Modality) -> Selection? {
    for ring in 0..<12 {
      if primary(ring: ring, mode: mode) == tonic {
        return Selection(ring: ring, mode: mode, spelling: tonic)
      }
      if let alt = enharmonicAlt(ring: ring, mode: mode), alt == tonic {
        return Selection(ring: ring, mode: mode, spelling: alt)
      }
    }
    return nil
  }

  private static func spokenTonic(_ tonic: String) -> String {
    var spoken = ""
    var prev: Character?
    for c in tonic {
      if c == "#" {
        spoken += " sharp"
      } else if c == "b", let p = prev, ("A"..."G").contains(p) {
        spoken += " flat"
      } else {
        spoken.append(c)
      }
      prev = c
    }
    return spoken
  }
}
