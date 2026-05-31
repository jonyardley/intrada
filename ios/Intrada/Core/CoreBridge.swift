import SharedTypes

/// The single abstraction the app uses to talk to the Rust core. Swift values
/// in, Swift values out — the concrete `LiveBridge` is the *only* place that
/// knows about bincode or the UniFFI `CoreFFI` object (see CLAUDE.md "Native
/// iOS Shell"). A fake conforming type lets views/tests run without the FFI.
protocol CoreBridge {
  func update(_ event: Event) throws -> [Request]
  func resolve(_ id: UInt32, httpResult: HttpResult) throws -> [Request]
  func resolveEmpty(_ id: UInt32) throws -> [Request]
  func view() throws -> ViewModel
}
