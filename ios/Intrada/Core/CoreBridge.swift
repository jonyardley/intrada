import SharedTypes

/// The single abstraction the app uses to talk to the Rust core. Swift values
/// in, Swift values out — the concrete `LiveBridge` is the *only* place that
/// knows about bincode or the UniFFI `CoreFFI` object (see CLAUDE.md "Native
/// iOS Shell"). A fake conforming type lets views/tests run without the FFI.
protocol CoreBridge {
  /// Dispatch an event; returns the effect requests the core emitted.
  func update(_ event: Event) throws -> [Request]
  /// Resolve an HTTP effect with its result; returns follow-up requests.
  func resolve(_ id: UInt32, httpResult: HttpResult) throws -> [Request]
  /// Resolve a non-output effect (e.g. localStorage); returns follow-ups.
  func resolveEmpty(_ id: UInt32) throws -> [Request]
  /// The current serialized `ViewModel`.
  func view() throws -> ViewModel
}
