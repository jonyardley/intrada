import Foundation

/// The one id the shell has to produce before the core hears about it: a
/// photo's file name, which has to exist on disk before `SetPhoto` can name it
/// (offline-first invariant 3). The core still refuses any id it is handed that
/// is not a ulid (`validate_photo_id`), and a real-bridge test runs a minted id
/// through that validator so this encoder cannot drift from Rust's.
enum Ulid {
  /// Crockford base32 — no I, L, O or U.
  private static let alphabet = Array("0123456789ABCDEFGHJKMNPQRSTVWXYZ")

  static func generate(
    millisecondsSinceEpoch: UInt64 = UInt64(Date().timeIntervalSince1970 * 1000),
    randomByte: () -> UInt8 = { UInt8.random(in: .min ... .max) }
  ) -> String {
    let timestamp = stride(from: 40, through: 0, by: -8).map {
      UInt8((millisecondsSinceEpoch >> UInt64($0)) & 0xFF)
    }
    return encode(timestamp + (0..<10).map { _ in randomByte() })
  }

  /// 16 bytes as 26 characters of 5 bits, left-padded to 130 — the two leading
  /// zero bits are what keeps a ulid's first character at or below '7'.
  private static func encode(_ bytes: [UInt8]) -> String {
    var accumulator: UInt32 = 0
    var pending = 2
    var out = ""
    out.reserveCapacity(26)
    for byte in bytes {
      accumulator = (accumulator << 8) | UInt32(byte)
      pending += 8
      while pending >= 5 {
        pending -= 5
        out.append(alphabet[Int((accumulator >> UInt32(pending)) & 0x1F)])
      }
    }
    return out
  }
}
