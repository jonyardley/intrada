import Foundation
import IntradaCoreFFI
import SharedTypes

/// The notebook's database: the Rust store both phones share (#2432). It
/// opens the `intrada.sqlite` GRDB wrote, carrying on from GRDB's record of
/// the migrations already run.
final class SharedItemStore: Sendable {
  static let decodeContext = "store decode"

  private let store: StoreFfi

  private init(_ store: StoreFfi) {
    self.store = store
  }

  /// Application Support does not exist on a fresh install, and the store
  /// will not create a missing folder.
  static func onDisk() throws -> SharedItemStore {
    let dir = try FileManager.default.url(
      for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
    return try at(path: dir.appendingPathComponent("intrada.sqlite").path)
  }

  static func at(path: String) throws -> SharedItemStore {
    SharedItemStore(try StoreFfi.open(path: path))
  }

  /// For tests, previews and the fallback when the file will not open.
  static func inMemory() throws -> SharedItemStore {
    SharedItemStore(try StoreFfi.inMemory())
  }

  func handle(_ operation: PersistenceOperation) throws -> PersistenceOutput {
    let answer = try store.handle(operation: Data(try operation.bincodeSerialize()))
    for value in answer.unreadable {
      report(UnreadableStoredValue(description: value), Self.decodeContext)
    }
    if let reason = answer.error { throw StoreFailure(description: reason) }
    return try PersistenceOutput.bincodeDeserialize(input: [UInt8](answer.output))
  }

  struct UnreadableStoredValue: Error, CustomStringConvertible {
    let description: String
  }

  struct StoreFailure: Error, CustomStringConvertible {
    let description: String
  }
}
