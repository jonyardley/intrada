import Foundation

/// mach_absolute_time helpers — the host-time domain AVAudioTime(hostTime:)
/// schedules against, so the click can be placed sample-accurately.
enum HostClock {
  static let timebase: mach_timebase_info_data_t = {
    var info = mach_timebase_info_data_t()
    mach_timebase_info(&info)
    return info
  }()

  static func now() -> UInt64 {
    mach_absolute_time()
  }

  /// Non-negative durations only — Double->UInt64 traps on negative input.
  /// Use `secondsBetween` (or `&+`/`&-`) for a signed offset.
  static func ticks(fromSeconds seconds: Double) -> UInt64 {
    precondition(
      seconds >= 0, "ticks(fromSeconds:) requires a non-negative duration, got \(seconds)")
    return UInt64(seconds * 1_000_000_000 * Double(timebase.denom) / Double(timebase.numer))
  }

  static func seconds(fromTicks ticks: UInt64) -> Double {
    Double(ticks) * Double(timebase.numer) / Double(timebase.denom) / 1_000_000_000
  }

  /// Signed tick delta as seconds — `lhs - rhs`, positive when lhs is later.
  static func secondsBetween(_ lhs: UInt64, _ rhs: UInt64) -> Double {
    if lhs >= rhs {
      return seconds(fromTicks: lhs - rhs)
    }
    return -seconds(fromTicks: rhs - lhs)
  }
}
