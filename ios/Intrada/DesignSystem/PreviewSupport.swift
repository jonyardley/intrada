#if DEBUG
  import IntradaCoreFFI
  import SharedTypes

  /// Offline bridge for Xcode previews: serves the core's initial (empty)
  /// ViewModel and emits no effects, so store-backed screens render in the
  /// canvas without FFI networking.
  final class PreviewBridge: CoreBridge {
    private let core = CoreFfi()
    func update(_ event: Event) throws -> [Request] { [] }
    func resolve(_ id: UInt32, httpResult: HttpResult) throws -> [Request] { [] }
    func resolveEmpty(_ id: UInt32) throws -> [Request] { [] }
    func view() throws -> ViewModel {
      try ViewModel.bincodeDeserialize(input: [UInt8](core.view()))
    }
  }

  extension Store {
    /// A deterministic, offline store for `#Preview` blocks.
    static var preview: Store { Store(bridge: PreviewBridge()) }
  }
#endif
