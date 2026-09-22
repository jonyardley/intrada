import Foundation
import IntradaCoreFFI
import SharedTypes

/// The sole owner of bincode + the UniFFI core handle. Everything else in the
/// app works against the `CoreBridge` protocol with plain Swift values.
/// (UniFFI de-capitalizes the `CoreFFI` Rust type to `CoreFfi` in Swift.)
final class LiveBridge: CoreBridge {
  private let core = CoreFfi()

  func update(_ event: Event) throws -> [Request] {
    let input = Data(try event.bincodeSerialize())
    let out = try call { try core.update(data: input) }
    return try Requests.bincodeDeserialize(input: [UInt8](out)).value
  }

  func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request] {
    let input = Data(try persistenceOutput.bincodeSerialize())
    let out = try call { try core.resolve(id: id, data: input) }
    return try Requests.bincodeDeserialize(input: [UInt8](out)).value
  }

  func resolve(_ id: UInt32, recognitionOutput: RecognitionOutput) throws -> [Request] {
    let input = Data(try recognitionOutput.bincodeSerialize())
    let out = try call { try core.resolve(id: id, data: input) }
    return try Requests.bincodeDeserialize(input: [UInt8](out)).value
  }

  func resolveEmpty(_ id: UInt32) throws -> [Request] {
    let out = try call { try core.resolve(id: id, data: Data()) }
    return try Requests.bincodeDeserialize(input: [UInt8](out)).value
  }

  func view() throws -> ViewModel {
    try ViewModel.bincodeDeserialize(input: [UInt8](try call { try core.view() }))
  }

  // A generated call throws either the declared `CoreError` or UniFFI's own
  // `UniffiInternalError`, which is fileprivate to the bindings and carries the
  // panic; anything that is not the former is therefore the latter (#1946).
  private func call(_ work: () throws -> Data) throws -> Data {
    do {
      return try work()
    } catch let error as CoreError {
      throw error
    } catch {
      throw CorePanic(underlying: error)
    }
  }
}
