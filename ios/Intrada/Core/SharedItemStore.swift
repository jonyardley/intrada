import Foundation
import IntradaCoreFFI
import SharedTypes

/// Persistence the Store resolves against: a protocol so tests can inject a failing fake (#816).
protocol ItemStore: Sendable {
  func handle(_ operation: PersistenceOperation) throws -> PersistenceOutput
}

/// The notebook's database: the Rust store both phones share (#2432). It
/// opens the `intrada.sqlite` GRDB wrote, carrying on from GRDB's record of
/// the migrations already run.
final class SharedItemStore: ItemStore {
  static let decodeContext = "store decode"

  private let store: StoreFfiProtocol

  init(_ store: StoreFfiProtocol) {
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
