import SharedTypes

/// The keys sheet's tap rule over a set of keys (#2247): a tap adds the spoke's
/// key; on a spoke with two spellings the next tap switches to the other, and
/// the one after removes it. The spellings come from the core's wheel.
enum KeySet {
  static func chosenSpelling(in keys: [Key], ring: Int, mode: Modality) -> String? {
    keys.lazy.compactMap(KeyHelper.selection).first { $0.ring == ring && $0.mode == mode }?
      .spelling
  }

  static func tap(_ keys: [Key], ring: Int, mode: Modality) -> [Key] {
    guard
      let index = keys.firstIndex(where: {
        KeyHelper.selection($0).map { $0.ring == ring && $0.mode == mode } ?? false
      })
    else {
      guard let added = KeyHelper.nextOnTap(current: nil, ring: ring, mode: mode)?.key else {
        return keys
      }
      return keys + [added]
    }
    var keys = keys
    let onPrimary =
      KeyHelper.selection(keys[index])?.spelling == KeyHelper.primary(ring: ring, mode: mode)
    if onPrimary, KeyHelper.enharmonicAlt(ring: ring, mode: mode) != nil,
      let flipped = KeyHelper.nextOnTap(current: keys[index], ring: ring, mode: mode)?.key
    {
      keys[index] = flipped
    } else {
      keys.remove(at: index)
    }
    return keys
  }

  /// Adds every spoke of `mode` not already chosen, keeping the rest as they are.
  static func addingAll(_ mode: Modality, to keys: [Key]) -> [Key] {
    (0..<12).reduce(keys) { keys, ring in
      chosenSpelling(in: keys, ring: ring, mode: mode) == nil
        ? tap(keys, ring: ring, mode: mode) : keys
    }
  }
}
