#if DEBUG
  import IntradaCoreFFI
  import SharedTypes

  /// Offline bridge for Xcode previews: serves the core's initial (empty)
  /// ViewModel — optionally seeded with library items — and emits no effects,
  /// so store-backed screens render in the canvas without FFI networking.
  final class PreviewBridge: CoreBridge {
    private let core = CoreFfi()
    private let items: [LibraryItemView]

    init(items: [LibraryItemView] = []) { self.items = items }

    func update(_ event: Event) throws -> [Request] { [] }
    func resolve(_ id: UInt32, httpResult: HttpResult) throws -> [Request] { [] }
    func resolveEmpty(_ id: UInt32) throws -> [Request] { [] }
    func view() throws -> ViewModel {
      var viewModel = try ViewModel.bincodeDeserialize(input: [UInt8](core.view()))
      viewModel.items = items
      return viewModel
    }
  }

  extension Store {
    /// A deterministic, offline store for `#Preview` blocks.
    static var preview: Store { Store(bridge: PreviewBridge()) }

    /// An offline store seeded with sample library items.
    static var previewLibrary: Store {
      Store(bridge: PreviewBridge(items: [.previewPiece, .previewExercise, .previewMinimal]))
    }
  }

  extension LibraryItemView {
    static var previewPiece: LibraryItemView {
      LibraryItemView(
        id: "piece-1", itemType: .piece, title: "Clair de Lune", subtitle: "Claude Debussy",
        key: "D♭ major", tempo: "Andante (72 BPM)", tempoMarking: "Andante", tempoBpm: 72,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        latestAchievedTempo: nil, priority: false)
    }

    static var previewExercise: LibraryItemView {
      LibraryItemView(
        id: "exercise-1", itemType: .exercise, title: "Hanon No. 1",
        subtitle: "Charles-Louis Hanon",
        key: "C major", tempo: "108 BPM", tempoMarking: nil, tempoBpm: 108,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        latestAchievedTempo: nil, priority: false)
    }

    static var previewMinimal: LibraryItemView {
      LibraryItemView(
        id: "piece-2", itemType: .piece, title: "Prelude in C", subtitle: "",
        key: nil, tempo: nil, tempoMarking: nil, tempoBpm: nil,
        notes: nil, tags: [], createdAt: "", updatedAt: "", practice: nil,
        latestAchievedTempo: nil, priority: false)
    }
  }
#endif
