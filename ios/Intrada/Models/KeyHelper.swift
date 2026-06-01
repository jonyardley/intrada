import Foundation
import SharedTypes

/// Pure circle-of-fifths logic for the `KeyPicker` — selection, spelling,
/// enharmonics, and display/accessibility strings. No SwiftUI/UIKit.
///
/// Keys are stored structured: `key` is the tonic (`"F#"`) and `modality` is the
/// core `Modality` (`.major`/`.minor`). Legacy freeform values (`"F# major"`)
/// are still parsed so old items display and self-heal to structured on save.
/// View-only by design (#819) — the mode type itself comes from the core.
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

  /// The enharmonic alternate spelling for the three ambiguous spokes.
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

  /// Resolve the wheel selection from stored fields: prefer structured
  /// (`key` tonic + `modality`), falling back to parsing a legacy combined
  /// string when `modality` is absent.
  static func selection(key: String, modality: Modality?) -> Selection? {
    if let modality, let s = ringFor(tonic: key, mode: modality) {
      return s
    }
    return parse(key)
  }

  /// Tap a spoke: returns the new `(tonic, modality)` and whether it was an
  /// enharmonic flip (vs a fresh selection). Tapping the already-selected
  /// enharmonic spoke flips the spelling; any other tap selects its default.
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

  static func modeWord(_ mode: Modality) -> String {
    switch mode {
    case .major: return "Major"
    case .minor: return "minor"
    }
  }

  /// Composed display for a stored key, e.g. "F♯ major". Falls back to the
  /// prettified raw value for legacy/unparseable keys with no modality.
  static func display(key: String?, modality: Modality?) -> String? {
    guard let key, !key.isEmpty else { return nil }
    if let modality {
      return "\(prettify(key)) \(modeWord(modality))"
    }
    return prettify(key)
  }

  /// Spoken label, e.g. "F sharp major". Enharmonic spokes announce both
  /// spellings since one tap selects and a second flips between them.
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

  /// Match a (tonic, mode) pair to a wheel spoke, or nil if the tonic isn't on
  /// the circle for that mode.
  private static func ringFor(tonic: String, mode: Modality) -> Selection? {
    guard let norm = normaliseTonic(tonic) else { return nil }
    for ring in 0..<12 {
      if primary(ring: ring, mode: mode) == norm {
        return Selection(ring: ring, mode: mode, spelling: norm)
      }
      if let alt = enharmonicAlt(ring: ring, mode: mode), alt == norm {
        return Selection(ring: ring, mode: mode, spelling: alt)
      }
    }
    return nil
  }

  /// Parse a legacy combined string ("F# major") into a selection.
  static func parse(_ raw: String) -> Selection? {
    let normalised =
      raw
      .replacingOccurrences(of: "\u{266F}", with: "#")
      .replacingOccurrences(of: "\u{266D}", with: "b")
      .trimmingCharacters(in: .whitespacesAndNewlines)
    if normalised.isEmpty { return nil }
    let lower = normalised.lowercased()
    let mode: Modality
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
    return ringFor(tonic: tonicRaw, mode: mode)
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
