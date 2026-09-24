import Foundation
import IntradaCoreFFI
import SharedTypes

/// Holds the `ViewModel`, sends `Event`s to the core, and runs the effect loop.
/// Owns zero domain logic — a pump between SwiftUI and the core (CLAUDE.md).
@MainActor
@Observable
final class Store {
  private(set) var viewModel: ViewModel?

  /// Bumped on every throw from the core bridge: a throw yields no render and no `errorSeq`
  /// move, so confirmations need their own signal for it (#1937).
  private(set) var bridgeFailureSeq = 0

  /// The core panicked, or the bridge failed twice running: nothing after that
  /// can work, so sends are refused without reaching the bridge and the shell
  /// shows a standing banner (#1946). The in-progress blob is untouched.
  private(set) var halted = false
  private var consecutiveBridgeFailures = 0
  static let haltedMessage = "The app has stopped responding · close and reopen it to carry on."

  /// On-disk store couldn't open → fell back to in-memory, so writes won't
  /// persist. The shell shows a standing warning. False for tests/previews.
  let degraded: Bool

  // FIXME(#2089): unversioned key, no wire pin.
  static let sortDefaultsKey = "intrada.library-sort"
  /// Positional bincode: any change to `ActiveSession`'s graph takes a new key,
  /// named by the core's `ActiveSession::BLOB_VERSION` so the bump never lives here (#1345, #1116).
  static let sessionInProgressKey = "intrada.session-in-progress.v\(sessionBlobVersion())"
  /// Positional bincode too: a field added to `Profile` takes a new key
  /// (`specs/profile.md`; pinned by the core's `profile_blob_wire_is_pinned`).
  static let profileDefaultsKey = "intrada.profile.v1"
  private let bridge: CoreBridge
  private let store: (any ItemStore)?
  private let sortDefaults: UserDefaults
  private var diskTail: Task<Void, Never>?

  init(
    bridge: CoreBridge = LiveBridge(),
    store: (any ItemStore)? = nil, sortDefaults: UserDefaults = .standard,
    degraded: Bool = false
  ) {
    self.bridge = bridge
    self.degraded = degraded
    // Default to in-memory so tests/previews never touch disk; the real app
    // passes an on-disk store. `try?` (not `guarded`) because `self` isn't fully
    // initialized yet here; in-memory creation effectively never fails.
    self.store = store ?? (try? LibraryStore.inMemory())
    self.sortDefaults = sortDefaults
    // Initial render comes straight from the core; nil only if the bridge
    // itself fails, in which case the view shows a loading state.
    self.viewModel = bridged { try bridge.view() }
  }

  func send(_ event: Event) {
    process(bridged { try bridge.update(event) } ?? [])
  }

  private func process(_ requests: [Request]) {
    for request in requests {
      switch request.effect {
      case .render:
        refreshView()
      case .app(let appEffect):
        // notify_shell effect: fire-and-forget, must not be resolved (#882).
        handleAppEffect(appEffect)
      case .persistence(let operation):
        enqueueDiskJob(operation, id: request.id)
      case .recognition(let operation):
        Task { await self.handleRecognition(operation, id: request.id) }
      }
    }
  }

  private func handleRecognition(_ operation: RecognitionOperation, id: UInt32) async {
    let output: RecognitionOutput
    switch operation {
    case .readPage(let photoId):
      output = await PageReader.read(photoId: photoId)
    }
    process(bridged { try bridge.resolve(id, recognitionOutput: output) } ?? [])
  }

  /// One job at a time, in the order the core asked: a load sent after a save
  /// must see the saved row.
  private func enqueueDiskJob(_ operation: PersistenceOperation, id: UInt32) {
    let previous = diskTail
    let job = DiskJob(store: store, operation: operation)
    diskTail = Task {
      await previous?.value
      let result = await Task.detached(priority: .userInitiated) { job.run() }.value
      if let error = result.error { report(error, "persistence") }
      process(bridged { try bridge.resolve(id, persistenceOutput: result.output) } ?? [])
    }
  }

  /// Waits until no disk job is queued, including jobs the core chains from a
  /// resolve while this waits.
  func settle() async {
    while let tail = diskTail {
      await tail.value
      if diskTail == tail { diskTail = nil }
    }
  }

  private func refreshView() {
    if let next = bridged({ try bridge.view() }) {
      viewModel = next
    }
  }

  private func handleAppEffect(_ effect: AppEffect) {
    switch effect {
    case .saveLibrarySort(let sort):
      if let bytes = guarded({ try sort.bincodeSerialize() }) {
        sortDefaults.set(Data(bytes), forKey: Self.sortDefaultsKey)
      }
    case .saveSessionInProgress(let active):
      if let bytes = guarded({ try active.bincodeSerialize() }) {
        sortDefaults.set(Data(bytes), forKey: Self.sessionInProgressKey)
      }
    case .clearSessionInProgress:
      sortDefaults.removeObject(forKey: Self.sessionInProgressKey)
      recoverableSession = nil
    case .saveProfile(let profile):
      if let bytes = guarded({ try profile.bincodeSerialize() }) {
        sortDefaults.set(Data(bytes), forKey: Self.profileDefaultsKey)
      }
    }
  }

  /// Crash-recovery blob found at launch; non-nil drives the Practice tab's
  /// Resume / Discard prompt (#962).
  var recoverableSession: ActiveSession?

  func pendingSessionInProgress() -> ActiveSession? {
    guard let data = sortDefaults.data(forKey: Self.sessionInProgressKey) else { return nil }
    return guarded { try ActiveSession.bincodeDeserialize(input: [UInt8](data)) }
  }

  func loadRecoverableSession() {
    guard viewModel?.activeSession == nil, viewModel?.summary == nil else { return }
    recoverableSession = pendingSessionInProgress()
  }

  func resumeRecoverableSession() {
    guard let session = recoverableSession else { return }
    send(.session(.recoverSession(session: session, now: SessionClock.nowRFC3339())))
    if viewModel?.activeSession != nil {
      recoverableSession = nil
    }
  }

  /// Pre-recovery discard is pure KV cleanup: the model is Idle, and the
  /// core's own clearing path (SaveSession or DiscardSession emitting
  /// ClearSessionInProgress) needs a session to be running (#962).
  func discardSessionInProgress() {
    sortDefaults.removeObject(forKey: Self.sessionInProgressKey)
    recoverableSession = nil
  }

  /// Tell the core which way the device's clock is offset from UTC so
  /// analytics turn the day over at the user's midnight (#1330).
  func reportUtcOffset(_ timeZone: TimeZone = .current) {
    send(.setUtcOffset(minutes: Int32(timeZone.secondsFromGMT() / 60)))
  }

  func restorePersistedSort() {
    guard let data = sortDefaults.data(forKey: Self.sortDefaultsKey),
      let sort = guarded({ try LibrarySort.bincodeDeserialize(input: [UInt8](data)) })
    else { return }
    send(.setSort(sort))
  }

  func forgetPersistedProfile() {
    sortDefaults.removeObject(forKey: Self.profileDefaultsKey)
  }

  func restorePersistedProfile() {
    guard let data = sortDefaults.data(forKey: Self.profileDefaultsKey),
      let profile = guarded({ try Profile.bincodeDeserialize(input: [UInt8](data)) })
    else { return }
    send(.profile(.loaded(profile)))
  }

  // A bridge failure means a serialization/protocol break (e.g. stale bindings
  // vs a regenerated core) — unrecoverable at runtime, so report it rather than
  // swallow it silently, and fail soft.
  private func guarded<T>(_ work: () throws -> T) -> T? {
    do { return try work() } catch {
      report(error, "bridge")
      return nil
    }
  }

  private func bridged<T>(_ work: () throws -> T) -> T? {
    guard !halted else {
      bridgeFailureSeq += 1
      return nil
    }
    do {
      let result = try work()
      consecutiveBridgeFailures = 0
      return result
    } catch {
      bridgeFailureSeq += 1
      consecutiveBridgeFailures += 1
      let panicked = error is CorePanic
      report(error, panicked ? "core-panic" : "bridge")
      if panicked || consecutiveBridgeFailures >= 2 { halted = true }
      return nil
    }
  }
}

/// Generated bridge types are value types without a `Sendable` conformance.
private struct DiskJob: @unchecked Sendable {
  let store: (any ItemStore)?
  let operation: PersistenceOperation

  struct Outcome: @unchecked Sendable {
    let output: PersistenceOutput
    let error: Error?
  }

  /// Failure (or no store) → `.failed` so the core surfaces it, not a phantom ack (#816).
  func run() -> Outcome {
    guard let store else { return Outcome(output: .failed, error: nil) }
    do {
      switch operation {
      case .loadItems: return Outcome(output: .items(try store.loadItems()), error: nil)
      case .saveItem(let item): try store.save(item)
      case .saveItems(let items): try store.save(items)
      case .deleteItem(let id, let deletedAt): try store.delete(id: id, deletedAt: deletedAt)
      case .loadSessions: return Outcome(output: .sessions(try store.loadSessions()), error: nil)
      case .saveSession(let session): try store.saveSession(session)
      }
      return Outcome(output: .ack, error: nil)
    } catch {
      return Outcome(output: .failed, error: error)
    }
  }
}
