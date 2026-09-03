import SharedTypes

/// The named ways a bar can sound. A preset is a mask over the metre; the
/// core validates the mask, this only offers the common ones (T19).
enum ClickPattern: Hashable, CaseIterable {
  case everyBeat
  case groupStarts
  case downbeat
  case backbeat

  var title: String {
    switch self {
    case .everyBeat: "Every beat"
    case .groupStarts: "Group starts"
    case .downbeat: "Downbeat"
    case .backbeat: "2 and 4"
    }
  }

  static func offered(for metre: Metre) -> [ClickPattern] {
    var offered: [ClickPattern] = [.everyBeat]
    if metre.groups != nil { offered.append(.groupStarts) }
    offered.append(.downbeat)
    if metre.groups == nil && metre.beats >= 4 { offered.append(.backbeat) }
    return offered
  }

  func mask(for metre: Metre) -> UInt16 {
    switch self {
    case .everyBeat: Self.everyBeat(of: metre)
    case .downbeat: 1
    case .backbeat: 0b1010
    case .groupStarts:
      metre.groups.map { groups in
        var mask: UInt16 = 0
        var start = 0
        for group in groups {
          mask |= 1 << UInt16(start)
          start += Int(group)
        }
        return mask
      } ?? 1
    }
  }

  static func everyBeat(of metre: Metre) -> UInt16 {
    (1 << UInt16(metre.beats)) - 1
  }

  /// The preset a mask matches, or nil for a hand-toggled pattern.
  static func matching(_ sounding: UInt16, in metre: Metre) -> ClickPattern? {
    offered(for: metre).first { $0.mask(for: metre) == sounding }
  }
}
