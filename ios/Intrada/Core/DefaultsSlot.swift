import Foundation

/// One bincode blob in UserDefaults; its key must change whenever the blob's shape does (#1345).
struct DefaultsSlot {
  let key: String
  let defaults: UserDefaults

  func read() -> [UInt8]? {
    defaults.data(forKey: key).map { [UInt8]($0) }
  }

  func write(_ bytes: [UInt8]) {
    defaults.set(Data(bytes), forKey: key)
  }

  func clear() {
    defaults.removeObject(forKey: key)
  }
}
