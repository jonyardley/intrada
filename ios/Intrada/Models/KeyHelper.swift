import Foundation
import IntradaCoreFFI
import SharedTypes

/// The circle of fifths and its tap rule come from the core (#2226); this
/// file only formats what it answers.
enum KeyHelper {
  struct Selection: Equatable {
    let ring: Int
    let mode: Modality
    let spelling: String
  }

  private static let wedges = keyWheel()

  /// Spoke order clockwise from 12 o'clock, the "Add 12 keys" presets' order.
  static func circle(_ mode: Modality) -> [String] {
    wedges.filter { $0.mode == wheelMode(mode) }.map(\.primary)
  }

  static func primary(ring: Int, mode: Modality) -> String {
    wedge(ring: ring, mode: mode)?.primary ?? ""
  }

  static func enharmonicAlt(ring: Int, mode: Modality) -> String? {
    wedge(ring: ring, mode: mode)?.alt
  }

  static func selection(key: String, modality: Modality?) -> Selection? {
    keyWheelSelection(key: key, mode: modality.map(wheelMode)).map {
      Selection(ring: Int($0.ring), mode: self.modality($0.mode), spelling: $0.spelling)
    }
  }

  static func nextOnTap(
    currentKey: String, currentModality: Modality?, ring: Int, mode: Modality
  ) -> (tonic: String, modality: Modality, flipped: Bool)? {
    guard let ring = UInt8(exactly: ring),
      let tap = keyNextOnTap(
        currentKey: currentKey, currentMode: currentModality.map(wheelMode), ring: ring,
        mode: wheelMode(mode))
    else { return nil }
    return (tap.tonic, modality(tap.mode), tap.flipped)
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

  private static func wedge(ring: Int, mode: Modality) -> WheelWedge? {
    wedges.first { Int($0.ring) == ring && $0.mode == wheelMode(mode) }
  }

  private static func wheelMode(_ mode: Modality) -> WheelMode {
    switch mode {
    case .major: return .major
    case .minor: return .minor
    }
  }

  private static func modality(_ mode: WheelMode) -> Modality {
    switch mode {
    case .major: return .major
    case .minor: return .minor
    }
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
