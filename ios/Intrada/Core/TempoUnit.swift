import SharedTypes

enum TempoUnit {
  static func spokenName(_ unit: UInt8) -> String {
    switch unit {
    case 2: "minim"
    case 8: "quaver"
    default: "crotchet"
    }
  }

  static func metreLabel(_ metre: Metre) -> String { "\(metre.beats)/\(metre.unit)" }
}
