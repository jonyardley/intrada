import SharedTypes

/// The single abstraction the app uses to talk to the Rust core. Swift values
/// in, Swift values out — the concrete `LiveBridge` is the *only* place that
/// knows about bincode or the UniFFI `CoreFFI` object (see CLAUDE.md "Native
/// iOS Shell"). A fake conforming type lets views/tests run without the FFI.
protocol CoreBridge {
  func update(_ event: Event) throws -> [Request]
  func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request]
  func resolve(_ id: UInt32, recognitionOutput: RecognitionOutput) throws -> [Request]
  func resolveEmpty(_ id: UInt32) throws -> [Request]
  func view() throws -> ViewModel
}

/// The Rust side panicked, or the UniFFI call itself broke: nothing after this
/// call can work (#1946).
struct CorePanic: Error {
  let underlying: Error
}
