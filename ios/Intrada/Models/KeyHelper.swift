import Foundation
import SharedTypes

/// Keys are stored structured (`key` tonic + core `Modality`); legacy freeform
/// values ("F# major") are still parsed so old items display and self-heal on
/// save. View-only by design (#819) — the mode type comes from the core.
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
    if let modality, let s = ringFor(tonic: key, mode: modality) {
      return s
    }
    return parse(key)
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
    case .major: return "Major"
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
