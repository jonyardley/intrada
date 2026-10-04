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

  /// Spoke order clockwise from 12 o'clock.
  static func circle(_ mode: Modality) -> [Key] {
    (0..<12).compactMap { nextOnTap(current: nil, ring: $0, mode: mode)?.key }
  }

  static func primary(ring: Int, mode: Modality) -> String {
    wedge(ring: ring, mode: mode)?.primary ?? ""
  }

  static func enharmonicAlt(ring: Int, mode: Modality) -> String? {
    wedge(ring: ring, mode: mode)?.alt
  }

  /// A failed call is a wire break (#846): reported, and the wheel lights nothing.
  static func selection(_ key: Key) -> Selection? {
    bridged {
      try keyWheelSelection(key: Data(key.bincodeSerialize())).map {
        Selection(ring: Int($0.ring), mode: self.modality($0.mode), spelling: $0.spelling)
      }
    } ?? nil
  }

  static func nextOnTap(current: Key?, ring: Int, mode: Modality) -> (key: Key, flipped: Bool)? {
    guard let ring = UInt8(exactly: ring) else { return nil }
    return bridged {
      let bytes = try current.map { Data(try $0.bincodeSerialize()) }
      guard let tap = try keyNextOnTap(current: bytes, ring: ring, mode: wheelMode(mode)) else {
        return nil
      }
      return (try Key.bincodeDeserialize(input: [UInt8](tap.key)), tap.flipped)
    } ?? nil
  }

  /// The keys sheet's tap over a set of keys (#2372). A failed call is a wire
  /// break (#846): reported, and the keys stay as they were.
  static func tap(_ keys: [Key], ring: Int, mode: Modality) -> [Key] {
    guard let ring = UInt8(exactly: ring) else { return keys }
    return keySet(keys) { try keySetTap(keys: $0, ring: ring, mode: wheelMode(mode)) }
  }

  static func addingAll(_ mode: Modality, to keys: [Key]) -> [Key] {
    keySet(keys) { try keySetAddAll(keys: $0, mode: wheelMode(mode)) }
  }

  private static func keySet(_ keys: [Key], _ call: ([Data]) throws -> [Data]) -> [Key] {
    bridged {
      try call(keys.map { Data(try $0.bincodeSerialize()) })
        .map { try Key.bincodeDeserialize(input: [UInt8]($0)) }
    } ?? keys
  }

  /// "E♭ major", in the core's words.
  static func display(_ key: Key) -> String? {
    bridged { try keyLabel(key: Data(key.bincodeSerialize())) }
  }

  private static func bridged<T>(_ call: () throws -> T) -> T? {
    do {
      return try call()
    } catch {
      report(error, "bridge")
      return nil
    }
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
