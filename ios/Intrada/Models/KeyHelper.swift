import Foundation

/// Pure circle-of-fifths logic for the `KeyPicker` — parsing, formatting,
/// enharmonic spelling, and display/accessibility strings. No SwiftUI/UIKit.
///
/// The stored value is a freeform ASCII string ("F# major", "Gb minor"); the UI
/// shows the prettified ♯/♭ form. Single source of truth for the wheel's
/// behaviour, kept view-free so it can be unit-tested and (later) lifted into
/// `intrada-core` for Android reuse — see the tracked follow-up.
enum KeyHelper {
  enum Mode: Equatable {
    case major
    case minor
  }

  struct Selection: Equatable {
    let ring: Int
    let mode: Mode
    let spelling: String
  }

  /// Major keys clockwise from 12 o'clock (C at top), by fifths.
  static let circleMajor = ["C", "G", "D", "A", "E", "B", "F#", "Db", "Ab", "Eb", "Bb", "F"]
  /// Relative minors, same spoke order.
  static let circleMinor = ["A", "E", "B", "F#", "C#", "G#", "D#", "Bb", "F", "C", "G", "D"]

  static func primary(ring: Int, mode: Mode) -> String {
    switch mode {
    case .major: return circleMajor[ring]
    case .minor: return circleMinor[ring]
    }
  }

  /// The enharmonic alternate spelling for the three ambiguous spokes.
  static func enharmonicAlt(ring: Int, mode: Mode) -> String? {
    switch (mode, ring) {
    case (.major, 5): return "Cb"
    case (.major, 6): return "Gb"
    case (.major, 7): return "C#"
    case (.minor, 5): return "Ab"
    case (.minor, 6): return "Eb"
    case (.minor, 7): return "A#"
    default: return nil
    }
  }

  static func format(tonic: String, mode: Mode) -> String {
    "\(tonic) \(modeWord(mode))"
  }

  /// Parse a freeform key string into a wheel selection, or `nil` if it is
  /// empty / not a recognised major-or-minor key.
  static func parse(_ raw: String) -> Selection? {
    let normalised =
      raw
      .replacingOccurrences(of: "\u{266F}", with: "#")  // ♯
      .replacingOccurrences(of: "\u{266D}", with: "b")  // ♭
      .trimmingCharacters(in: .whitespacesAndNewlines)
    if normalised.isEmpty { return nil }

    let lower = normalised.lowercased()
    let mode: Mode
    let tonicRaw: String
    if lower.hasSuffix("minor") {
      mode = .minor
      tonicRaw = String(normalised.dropLast(5))
    } else if lower.hasSuffix("major") {
      mode = .major
      tonicRaw = String(normalised.dropLast(5))
    } else {
      return nil
    }

    guard let tonic = normaliseTonic(tonicRaw) else { return nil }
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

  /// Given the current stored value and a tapped spoke, return the next
  /// canonical value and whether it was an enharmonic flip (vs a fresh
  /// selection). Tapping the already-selected enharmonic spoke flips the
  /// spelling; any other tap selects the spoke's default spelling.
  static func nextValueOnTap(current: String, ring: Int, mode: Mode) -> (
    value: String, flipped: Bool
  ) {
    let prim = primary(ring: ring, mode: mode)
    if let sel = parse(current), sel.ring == ring, sel.mode == mode {
      if let alt = enharmonicAlt(ring: ring, mode: mode) {
        let other = sel.spelling == prim ? alt : prim
        return (format(tonic: other, mode: mode), true)
      }
      return (format(tonic: prim, mode: mode), false)
    }
    return (format(tonic: prim, mode: mode), false)
  }

  /// ASCII → display form: `#`→`♯`, and an accidental `b` (one following a note
  /// letter) → `♭`. Mode words are left untouched.
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

  /// Spoken label, e.g. "F sharp major". Enharmonic spokes announce both
  /// spellings since one tap selects and a second flips between them.
  static func wedgeAccessibilityLabel(ring: Int, mode: Mode) -> String {
    let prim = primary(ring: ring, mode: mode)
    if let alt = enharmonicAlt(ring: ring, mode: mode) {
      return "\(spokenTonic(prim)) or \(spokenTonic(alt)) \(modeWord(mode))"
    }
    return accessibilityLabel(prim, mode: mode)
  }

  static func accessibilityLabel(_ tonic: String, mode: Mode) -> String {
    "\(spokenTonic(tonic)) \(modeWord(mode))"
  }

  // ── Internal helpers ──

  private static func modeWord(_ mode: Mode) -> String {
    switch mode {
    case .major: return "major"
    case .minor: return "minor"
    }
  }

  private static func normaliseTonic(_ raw: String) -> String? {
    let chars = Array(raw.trimmingCharacters(in: .whitespaces))
    guard let first = chars.first, let letter = first.uppercased().first,
      ("A"..."G").contains(letter)
    else {
      return nil
    }
    if chars.count == 1 { return String(letter) }
    guard chars.count == 2 else { return nil }
    switch chars[1] {
    case "#": return "\(letter)#"
    case "b", "B": return "\(letter)b"
    default: return nil
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
