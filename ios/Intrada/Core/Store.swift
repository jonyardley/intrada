import Foundation
import IntradaCoreFFI
import SharedTypes

/// Holds the `ViewModel`, sends `Event`s to the core, and runs the effect loop.
/// Owns zero domain logic — a pump between SwiftUI and the core (CLAUDE.md).
@MainActor
@Observable
final class Store {
  private(set) var viewModel: ViewModel?
  /// Sent by the core only when a row changes, so a tap mid-practice does not
  /// replace them (#1801).
  private(set) var libraryRows: [LibraryItemView] = []
  private(set) var sessionHistory: [PracticeSessionView] = []
  private(set) var practiceWeeks: [PracticeWeekView] = []

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

  static let sortDefaultsKey = "intrada.library-sort.v\(librarySortBlobVersion())"
  /// Builds before #2089 saved the v1 shape unversioned. It moves to the v1
  /// key, never the current one, so a later bump cannot decode it as its own.
  static let legacySortDefaultsKey = "intrada.library-sort"
  static let legacySortMovesToKey = "intrada.library-sort.v1"
  /// Positional bincode: any change to `ActiveSession`'s graph takes a new key,
  /// named by the core's `ActiveSession::BLOB_VERSION` so the bump never lives here (#1345, #1116).
  static let sessionInProgressKey = "intrada.session-in-progress.v\(sessionBlobVersion())"
  /// Positional bincode too: a field added to `Profile` takes a new key,
  /// named by the core's `Profile::BLOB_VERSION` so the bump never lives here (#2026).
  static let profileDefaultsKey = "intrada.profile.v\(profileBlobVersion())"
  /// Its own blob, never fields on `Profile`, so no profile is lost to a
  /// failed decode (#1915; pinned by `practice_defaults_blob_wire_is_pinned`).
  static let practiceDefaultsKey = "intrada.practice-defaults.v2"  // gitleaks:allow
  static let firstRunKey = "intrada.first-run.v\(firstRunBlobVersion())"
  private let bridge: CoreBridge
  private let store: (any ItemStore)?
  private let sortSlot: DefaultsSlot
  private let legacySortSlot: DefaultsSlot
  private let sessionSlot: DefaultsSlot
  private let profileSlot: DefaultsSlot
  private let practiceDefaultsSlot: DefaultsSlot
  private let firstRunSlot: DefaultsSlot
  private var diskTail: Task<Void, Never>?

  init(
    bridge: CoreBridge = LiveBridge(),
    store: (any ItemStore)? = nil, sortDefaults: UserDefaults = .standard,
    degraded: Bool = false
  ) {
    self.bridge = bridge
    self.degraded = degraded
    // Default to in-memory so tests/previews never touch disk; the real app
    // passes an on-disk store. Not `guarded`: `self` isn't initialized yet.
    if let store {
      self.store = store
    } else {
      do {
        self.store = try SharedItemStore.inMemory()
      } catch {
        report(error, "in-memory store")
        self.store = nil
      }
    }
    sortSlot = DefaultsSlot(key: Self.sortDefaultsKey, defaults: sortDefaults)
    legacySortSlot = DefaultsSlot(key: Self.legacySortDefaultsKey, defaults: sortDefaults)
    sessionSlot = DefaultsSlot(key: Self.sessionInProgressKey, defaults: sortDefaults)
    profileSlot = DefaultsSlot(key: Self.profileDefaultsKey, defaults: sortDefaults)
    practiceDefaultsSlot = DefaultsSlot(key: Self.practiceDefaultsKey, defaults: sortDefaults)
    firstRunSlot = DefaultsSlot(key: Self.firstRunKey, defaults: sortDefaults)
    // Initial render comes straight from the core; nil only if the bridge
    // itself fails, in which case the view shows a loading state.
    self.viewModel = bridged { try bridge.view() }
  }

  func send(_ event: Event) {
    let requests = bridgeSignposter.withIntervalSignpost("bridge.update") {
      bridged { try bridge.update(event) } ?? []
    }
    process(requests)
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
      case .sync:
        // FIXME(#2355): the sync engine carries these to iCloud.
        break
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
    let next = bridgeSignposter.withIntervalSignpost("bridge.view") {
      bridged { try bridge.view() }
    }
    if let next {
      bridgeSignposter.withIntervalSignpost("viewModel.assign") { viewModel = next }
    }
  }

  private func handleAppEffect(_ effect: AppEffect) {
    switch effect {
    case .saveLibrarySort(let sort):
      if let bytes = guarded({ try sort.bincodeSerialize() }) {
        sortSlot.write(bytes)
      }
    case .saveSessionInProgress(let active):
      if let bytes = guarded({ try active.bincodeSerialize() }) {
        sessionSlot.write(bytes)
      }
    case .clearSessionInProgress:
      sessionSlot.clear()
      recoverableSession = nil
    case .saveProfile(let profile):
      if let bytes = guarded({ try profile.bincodeSerialize() }) {
        profileSlot.write(bytes)
      }
    case .savePracticeDefaults(let defaults):
      if let bytes = guarded({ try defaults.bincodeSerialize() }) {
        practiceDefaultsSlot.write(bytes)
      }
    case .saveFirstRun(let firstRun):
      if let bytes = guarded({ try firstRun.bincodeSerialize() }) {
        firstRunSlot.write(bytes)
      }
    case .libraryChanged(let rows):
      libraryRows = rows
    case .historyChanged(let sessions):
      sessionHistory = sessions
    case .weeksChanged(let weeks):
      practiceWeeks = weeks
    }
  }

  /// The rows the Library filter leaves showing, in sort order (#1998).
  var visibleItems: [LibraryItemView] {
    libraryRows.rows(withIds: viewModel?.visibleIds ?? [])
  }

  var recentlyPractisedItems: [LibraryItemView] {
    libraryRows.rows(withIds: viewModel?.recentlyPractisedIds ?? [])
  }

  /// Crash-recovery blob found at launch; non-nil drives the Practice tab's
  /// Resume / Discard prompt (#962).
  var recoverableSession: ActiveSession?

  func pendingSessionInProgress() -> ActiveSession? {
    guard let bytes = sessionSlot.read() else { return nil }
    return guarded { try ActiveSession.bincodeDeserialize(input: bytes) }
  }

  func loadRecoverableSession() {
    guard viewModel?.offersRecovery == true else { return }
    clearRetiredSessionsInProgress()
    recoverableSession = pendingSessionInProgress()
  }

  /// A practice saved by an older build has a shape this one cannot read, so
  /// it is never half restored: its key is deleted and the core says so (#2246).
  private func clearRetiredSessionsInProgress() {
    let retired = (1..<sessionBlobVersion())
      .map {
        DefaultsSlot(key: "intrada.session-in-progress.v\($0)", defaults: sessionSlot.defaults)
      }
      .filter { $0.read() != nil }
    guard !retired.isEmpty else { return }
    for slot in retired { slot.clear() }
    send(.session(.retiredSessionFound))
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
    sessionSlot.clear()
    recoverableSession = nil
  }

  /// Tell the core which way the device's clock is offset from UTC so
  /// analytics turn the day over at the user's midnight (#1330).
  func reportUtcOffset(_ timeZone: TimeZone = .current) {
    send(.setUtcOffset(minutes: Int32(timeZone.secondsFromGMT() / 60)))
  }

  func restorePersistedSort() {
    if let legacy = legacySortSlot.read() {
      let v1Slot = DefaultsSlot(key: Self.legacySortMovesToKey, defaults: legacySortSlot.defaults)
      if v1Slot.read() == nil { v1Slot.write(legacy) }
      legacySortSlot.clear()
    }
    guard let bytes = sortSlot.read(),
      let sort = guarded({ try LibrarySort.bincodeDeserialize(input: bytes) })
    else { return }
    send(.setSort(sort))
  }

  func forgetPersistedProfileDefaultsAndFirstRun() {
    profileSlot.clear()
    practiceDefaultsSlot.clear()
    firstRunSlot.clear()
  }

  func restorePersistedProfile() {
    guard let bytes = profileSlot.read(),
      let profile = guarded({ try Profile.bincodeDeserialize(input: bytes) })
    else { return }
    send(.profile(.loaded(profile)))
  }

  func restorePersistedPracticeDefaults() {
    guard let bytes = practiceDefaultsSlot.read(),
      let defaults = guarded({ try PracticeDefaults.bincodeDeserialize(input: bytes) })
    else { return }
    send(.practiceDefaults(.loaded(defaults)))
  }

  func restorePersistedFirstRun() {
    guard let bytes = firstRunSlot.read(),
      let firstRun = guarded({ try FirstRun.bincodeDeserialize(input: bytes) })
    else { return }
    send(.firstRun(.loaded(firstRun)))
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
      return Outcome(output: try store.handle(operation), error: nil)
    } catch {
      return Outcome(output: .failed, error: error)
    }
  }
}

#if DEBUG
  extension Store {
    /// Previews and snapshots hand over the rows the core would send (#1801).
    func receive(_ effects: [AppEffect]) {
      effects.forEach(handleAppEffect)
    }
  }
#endif
