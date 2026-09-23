import IntradaCoreFFI
import SharedTypes
import XCTest

@testable import Intrada

@MainActor
final class StoreEffectLoopTests: XCTestCase {

  // ── Effect dispatch ────────────────────────────────────────────────────

  func testInitRendersInitialViewModel() {
    let bridge = FakeBridge()
    let store = Store(bridge: bridge)
    XCTAssertNotNil(store.viewModel, "init should seed the ViewModel from the bridge")
    XCTAssertEqual(bridge.viewCallCount, 1)
  }

  func testRenderEffectRefreshesViewModel() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 1, effect: .render(RenderOperation()))] }
    let store = Store(bridge: bridge)

    bridge.nextViewModel = {
      var vm = try emptyViewModel()
      vm.error = "refreshed"
      return vm
    }
    store.send(.setQuery(nil))

    XCTAssertEqual(store.viewModel?.error, "refreshed", "render effect should re-read view()")
  }

  func testSaveSessionInProgressEffectWritesBlobAndClearRemovesIt() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "sip-\(UUID().uuidString)"))
    let active = ActiveSession(
      id: "s-crash", entries: [], currentIndex: 0,
      currentItemStartedAt: "2026-07-14T10:00:00Z", sessionStartedAt: "2026-07-14T10:00:00Z")
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 1, effect: .app(.saveSessionInProgress(active)))]
    }
    let store = Store(bridge: bridge, sortDefaults: defaults)

    store.send(.setQuery(nil))

    let pending = try XCTUnwrap(
      store.pendingSessionInProgress(), "the save effect must persist a recoverable blob")
    XCTAssertEqual(pending.id, "s-crash")

    bridge.updateHandler = { _ in [Request(id: 2, effect: .app(.clearSessionInProgress))] }
    store.send(.setQuery(nil))
    XCTAssertNil(
      store.pendingSessionInProgress(), "the clear effect must remove the recoverable blob")
  }

  func testDiscardSessionInProgressRemovesBlobWithoutCoreEvent() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "sip-\(UUID().uuidString)"))
    let active = ActiveSession(
      id: "s-stale", entries: [], currentIndex: 0,
      currentItemStartedAt: "2026-07-14T10:00:00Z", sessionStartedAt: "2026-07-14T10:00:00Z")
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 1, effect: .app(.saveSessionInProgress(active)))]
    }
    let store = Store(bridge: bridge, sortDefaults: defaults)
    store.send(.setQuery(nil))
    store.recoverableSession = store.pendingSessionInProgress()
    XCTAssertNotNil(store.recoverableSession)

    store.discardSessionInProgress()

    XCTAssertNil(store.pendingSessionInProgress())
    XCTAssertNil(store.recoverableSession)
  }

  func testAppEffectIsNotResolved() {
    // Why never resolve: testRealBridgeAppEffectIsNeverResolved (#882).
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 7, effect: .app(.clearSessionInProgress))] }
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))

    XCTAssertTrue(bridge.emptyResolved.isEmpty, "app effect must not be resolved")
  }

  func testPersistenceLoadResolvesFromStore() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 8, effect: .persistence(.loadItems))] }
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))

    XCTAssertEqual(bridge.persistenceResolved.first?.id, 8)
    guard case .items(let items) = bridge.persistenceResolved.first?.output else {
      return XCTFail(
        "expected .items, got \(String(describing: bridge.persistenceResolved.first?.output))")
    }
    XCTAssertTrue(items.isEmpty, "fresh in-memory store has no rows")
  }

  /// The shell's whole job on this effect: run Vision, hand the core back a
  /// `RecognitionOutput`, decide nothing. An id with no bytes behind it is
  /// `Failed`, never a page of no lines (which the core would read as a blank).
  func testRecognitionResolvesFailedForAPhotoThatIsNotOnDisk() async {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 11, effect: .recognition(.readPage(photoId: Ulid.generate())))]
    }
    let store = Store(bridge: bridge)

    await whenResolved(bridge) { store.send(.setQuery(nil)) }

    XCTAssertEqual(bridge.recognitionResolved.first?.id, 11)
    XCTAssertEqual(bridge.recognitionResolved.first?.output, .failed)
  }

  func testPersistenceWriteFailureResolvesFailed() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 9, effect: .persistence(.saveItem(LibraryItemFixture.record())))]
    }
    let store = Store(bridge: bridge, store: FailingStore())

    store.send(.setQuery(nil))

    XCTAssertEqual(
      bridge.persistenceResolved.first?.output, .failed,
      "a failing local store must resolve .failed, not a phantom .ack")
  }

  func testBatchProcessesEveryRequest() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [
        Request(id: 1, effect: .app(.clearSessionInProgress)),
        Request(id: 2, effect: .render(RenderOperation())),
      ]
    }
    let store = Store(bridge: bridge)

    bridge.nextViewModel = {
      var vm = try emptyViewModel()
      vm.error = "batched"
      return vm
    }
    store.send(.setQuery(nil))

    XCTAssertTrue(bridge.emptyResolved.isEmpty, "app effect must not be resolved")
    XCTAssertEqual(store.viewModel?.error, "batched", "render after the app effect still runs")
  }

  // ── Library sort persistence ───────────────────────────────────────────

  func testSaveLibrarySortEffectWritesToDefaults() throws {
    let defaults = UserDefaults(suiteName: "sort-test-\(UUID().uuidString)")!
    let sort = LibrarySort(field: .title, direction: .ascending)
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 5, effect: .app(.saveLibrarySort(sort)))] }
    let store = Store(bridge: bridge, sortDefaults: defaults)

    store.send(.setQuery(nil))

    let data = try XCTUnwrap(defaults.data(forKey: Store.sortDefaultsKey))
    let restored = try LibrarySort.bincodeDeserialize(input: [UInt8](data))
    XCTAssertEqual(restored, sort, "save effect persists the chosen sort")
    XCTAssertTrue(bridge.emptyResolved.isEmpty, "the app effect must not be resolved (#882)")
  }

  func testRestorePersistedSortReplaysSetSort() throws {
    let defaults = UserDefaults(suiteName: "sort-test-\(UUID().uuidString)")!
    let sort = LibrarySort(field: .lastPracticed, direction: .ascending)
    defaults.set(Data(try sort.bincodeSerialize()), forKey: Store.sortDefaultsKey)

    let bridge = FakeBridge()
    var sentEvents: [Event] = []
    bridge.updateHandler = { event in
      sentEvents.append(event)
      return []
    }
    let store = Store(bridge: bridge, sortDefaults: defaults)

    store.restorePersistedSort()

    XCTAssertEqual(
      sentEvents, [.setSort(sort)], "restore re-dispatches SetSort with the stored order")
  }

  func testRestorePersistedSortNoopWhenAbsent() {
    let defaults = UserDefaults(suiteName: "sort-test-\(UUID().uuidString)")!
    let bridge = FakeBridge()
    var sentEvents: [Event] = []
    bridge.updateHandler = { event in
      sentEvents.append(event)
      return []
    }
    let store = Store(bridge: bridge, sortDefaults: defaults)

    store.restorePersistedSort()

    XCTAssertTrue(sentEvents.isEmpty, "no stored sort → no event")
  }

  // ── Profile persistence ────────────────────────────────────────────────

  func testSaveProfileEffectWritesToDefaultsUnderVersionedKey() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "profile-\(UUID().uuidString)"))
    let profile = Profile(name: "Jon", instrument: "Piano", iconChoice: .harp, colour: .sky)
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 7, effect: .app(.saveProfile(profile)))] }
    let store = Store(bridge: bridge, sortDefaults: defaults)

    store.send(.setQuery(nil))

    let data = try XCTUnwrap(defaults.data(forKey: Store.profileDefaultsKey))
    let restored = try Profile.bincodeDeserialize(input: [UInt8](data))
    XCTAssertEqual(restored, profile, "save effect persists the profile")
    XCTAssertTrue(bridge.emptyResolved.isEmpty, "the app effect must not be resolved (#882)")
  }

  func testRestorePersistedProfileReplaysLoaded() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "profile-\(UUID().uuidString)"))
    let profile = Profile(name: "Jon", instrument: "Cello", iconChoice: nil, colour: .butter)
    defaults.set(Data(try profile.bincodeSerialize()), forKey: Store.profileDefaultsKey)

    let bridge = FakeBridge()
    var sentEvents: [Event] = []
    bridge.updateHandler = { event in
      sentEvents.append(event)
      return []
    }
    let store = Store(bridge: bridge, sortDefaults: defaults)

    store.restorePersistedProfile()

    XCTAssertEqual(
      sentEvents, [.profile(.loaded(profile))],
      "restore replays Loaded, never Save, so nothing is re-validated or re-written")
  }

  func testForgetPersistedProfileRemovesTheBlob() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "profile-\(UUID().uuidString)"))
    let profile = Profile(name: "Jon", instrument: "Cello", iconChoice: nil, colour: .butter)
    defaults.set(Data(try profile.bincodeSerialize()), forKey: Store.profileDefaultsKey)
    let store = Store(bridge: FakeBridge(), sortDefaults: defaults)

    store.forgetPersistedProfile()

    XCTAssertNil(defaults.data(forKey: Store.profileDefaultsKey), "the reset flag's job")
  }

  func testRestorePersistedProfileNoopWhenAbsent() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "profile-\(UUID().uuidString)"))
    let bridge = FakeBridge()
    var sentEvents: [Event] = []
    bridge.updateHandler = { event in
      sentEvents.append(event)
      return []
    }
    let store = Store(bridge: bridge, sortDefaults: defaults)

    store.restorePersistedProfile()

    XCTAssertTrue(sentEvents.isEmpty, "no stored profile, no event")
  }

  // ── Failure-soft (guarded) ─────────────────────────────────────────────

  func testUpdateThrowIsSwallowedWithoutCrashing() {
    let bridge = FakeBridge()
    bridge.throwOnUpdate = TestError()
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))

    XCTAssertTrue(bridge.persistenceResolved.isEmpty)
    XCTAssertTrue(bridge.emptyResolved.isEmpty)
    XCTAssertNotNil(store.viewModel, "a thrown update should fail soft, not wipe the ViewModel")
    XCTAssertFalse(store.halted, "one plain throw is soft, not the end of the launch")
  }

  // ── Halt on a core panic (#1946) ───────────────────────────────────────

  func testCorePanicHaltsTheStoreOnTheFirstSend() {
    let bridge = FakeBridge()
    bridge.throwOnUpdate = CorePanic(underlying: TestError())
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))

    XCTAssertTrue(store.halted, "a panic poisons the core's model lock; nothing after it can work")
    XCTAssertNotNil(store.viewModel, "the last screen stays up under the banner")
  }

  func testHaltedStoreNeverReachesTheBridgeAgain() {
    let bridge = FakeBridge()
    bridge.throwOnUpdate = CorePanic(underlying: TestError())
    let store = Store(bridge: bridge)
    store.send(.setQuery(nil))
    let callsAtHalt = bridge.events.count + bridge.viewCallCount

    store.send(.setQuery(nil))

    XCTAssertEqual(
      bridge.events.count + bridge.viewCallCount, callsAtHalt,
      "a halted store refuses the send itself: one report, not one per tap")
  }

  func testTwoConsecutiveBridgeFailuresHaltTheStore() {
    let bridge = FakeBridge()
    bridge.throwOnUpdate = TestError()
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))
    XCTAssertFalse(store.halted)
    store.send(.setQuery(nil))

    XCTAssertTrue(store.halted, "a second failure in a row is a broken bridge, not a blip")
  }

  func testASuccessBetweenTwoFailuresKeepsTheStoreAlive() {
    let bridge = FakeBridge()
    bridge.throwOnUpdate = TestError()
    let store = Store(bridge: bridge)

    store.send(.setQuery(nil))
    bridge.throwOnUpdate = nil
    store.send(.setQuery(nil))
    bridge.throwOnUpdate = TestError()
    store.send(.setQuery(nil))

    XCTAssertFalse(store.halted, "the run resets on a send the bridge accepts")
  }

  func testHaltedStoreRefusesEverySend() {
    let bridge = FakeBridge()
    bridge.throwOnUpdate = CorePanic(underlying: TestError())
    let store = Store(bridge: bridge)
    store.send(.setQuery(nil))
    bridge.throwOnUpdate = nil

    XCTAssertFalse(
      store.sendAccepted(.setQuery(nil)),
      "a send that never reached the core must not fire a success haptic (#1937)")
  }

  func testCorePanicAtInitHaltsTheStore() {
    let bridge = FakeBridge()
    bridge.throwOnView = CorePanic(underlying: TestError())
    let store = Store(bridge: bridge)

    XCTAssertTrue(store.halted)
    XCTAssertNil(store.viewModel)
  }

  func testViewThrowAtInitLeavesViewModelNil() {
    let bridge = FakeBridge()
    bridge.throwOnView = TestError()
    let store = Store(bridge: bridge)

    XCTAssertNil(store.viewModel, "a thrown view() should leave nil (loading state), not crash")
  }

  func testResolveChainedRenderRefreshesView() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 4, effect: .persistence(.loadItems))] }
    bridge.resolveHandler = { _ in [Request(id: 2, effect: .render(RenderOperation()))] }
    let store = Store(bridge: bridge)

    bridge.nextViewModel = {
      var vm = try emptyViewModel()
      vm.error = "post-resolve"
      return vm
    }
    store.send(.setQuery(nil))

    XCTAssertEqual(
      store.viewModel?.error, "post-resolve", "render from a resolve should refresh view")
  }

  // ── Real bridge (app-wide events) ──────────────────────────────────────

  /// Real-bridge profile round-trip (#846): the Swift-encoded `Profile` must
  /// decode in the core, and the derived view must ride back.
  func testRealBridgeProfileSaveProjectsTheView() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)

    let requests = try bridge.update(
      .profile(
        .save(Profile(name: "  Jon ", instrument: "Double bass", iconChoice: nil, colour: .coral))))
    let saved = requests.compactMap { request -> Profile? in
      if case .app(.saveProfile(let profile)) = request.effect { return profile }
      return nil
    }
    XCTAssertEqual(saved.map(\.name), ["Jon"], "the save effect carries the trimmed profile")

    let view = try bridge.view().profile
    XCTAssertEqual(view.name, "Jon")
    XCTAssertEqual(view.instrument, "Double bass")
    XCTAssertEqual(view.suggestedIcon, .cello)
    XCTAssertEqual(view.icon, .cello)
    XCTAssertEqual(view.colour, .coral)
    XCTAssertTrue(
      ["Morning, Jon", "Afternoon, Jon", "Evening, Jon"].contains(view.greeting),
      "the greeting follows the device clock: \(view.greeting)")

    _ = try bridge.update(
      .profile(
        .loaded(Profile(name: "", instrument: "", iconChoice: .harp, colour: .butter))))
    let reloaded = try bridge.view().profile
    XCTAssertEqual(reloaded.icon, .harp, "the pick wins over the suggestion")
    XCTAssertEqual(reloaded.suggestedIcon, .other)
    XCTAssertEqual(reloaded.greeting, "", "no name means no greeting")
  }

  /// Real-bridge wire pin for `SetUtcOffset` (#1330): the Swift serializer and
  /// the Rust deserializer must agree on the new Event variant. Semantics are
  /// pinned by core tests; a wire break here surfaces as a throw or an error
  /// in the next view read.
  func testRealBridgeSetUtcOffsetCrossesTheWire() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp)

    _ = try bridge.update(.setUtcOffset(minutes: -300))

    XCTAssertNil(try bridge.view().error, "offset report should be a silent success")
  }

  /// App effects come from `notify_shell` — fire-and-forget notifications the
  /// live bridge rejects resolving, so the Store must not resolve `.app`. The
  /// stub bridge can't enforce this; pinned here against the real bridge (#882).
  func testRealBridgeAppEffectIsNeverResolved() throws {
    let bridge = LiveBridge()
    let requests = try bridge.update(
      .setSort(LibrarySort(field: .title, direction: .ascending)))
    let appRequest = try XCTUnwrap(
      requests.first { if case .app = $0.effect { return true } else { return false } },
      "setSort should emit an App (SaveLibrarySort) effect")

    XCTAssertThrowsError(try bridge.resolveEmpty(appRequest.id))
  }

  // ── Helpers ────────────────────────────────────────────────────────────

  /// Resume on the bridge's `resolve` callback, not a wall-clock `fulfillment`:
  /// a loaded CI runner starves the detached recognition Task and a tight
  /// ceiling flakes (#956, #861). The 30s net is a backstop, not the happy path.
  private func whenResolved(
    _ bridge: FakeBridge, _ action: () -> Void,
    file: StaticString = #filePath, line: UInt = #line
  ) async {
    let gate = ResolveGate()
    await withCheckedContinuation { (continuation: CheckedContinuation<Void, Never>) in
      let timeout = Task {
        try? await Task.sleep(for: .seconds(30))
        if gate.claim() {
          XCTFail("bridge never resolved within the 30s safety ceiling", file: file, line: line)
          continuation.resume()
        }
      }
      bridge.onResolve = {
        if gate.claim() {
          timeout.cancel()
          continuation.resume()
        }
      }
      action()
    }
  }
}

func emptyViewModel() throws -> ViewModel {
  try ViewModel.bincodeDeserialize(input: [UInt8](CoreFfi().view()))
}

final class FakeBridge: CoreBridge {
  var updateHandler: (Event) -> [Request] = { _ in [] }
  var resolveHandler: (UInt32) -> [Request] = { _ in [] }
  var nextViewModel: (() throws -> ViewModel)?
  var onResolve: (() -> Void)?
  /// When set, the corresponding bridge call throws — drives the Store's
  /// `guarded` failure-soft path.
  var throwOnUpdate: Error?
  var throwOnView: Error?

  private(set) var events: [Event] = []
  private(set) var persistenceResolved: [(id: UInt32, output: PersistenceOutput)] = []
  private(set) var recognitionResolved: [(id: UInt32, output: RecognitionOutput)] = []
  private(set) var emptyResolved: [UInt32] = []
  private(set) var viewCallCount = 0

  func update(_ event: Event) throws -> [Request] {
    events.append(event)
    if let throwOnUpdate { throw throwOnUpdate }
    return updateHandler(event)
  }

  func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request] {
    persistenceResolved.append((id, persistenceOutput))
    onResolve?()
    return resolveHandler(id)
  }

  func resolve(_ id: UInt32, recognitionOutput: RecognitionOutput) throws -> [Request] {
    recognitionResolved.append((id, recognitionOutput))
    onResolve?()
    return []
  }

  func resolveEmpty(_ id: UInt32) throws -> [Request] {
    emptyResolved.append(id)
    return []
  }

  func view() throws -> ViewModel {
    viewCallCount += 1
    if let throwOnView { throw throwOnView }
    if let nextViewModel { return try nextViewModel() }
    return try emptyViewModel()
  }
}

/// One-shot: only the first of {resolve, timeout} resumes (double-resume traps).
private final class ResolveGate {
  private let lock = NSLock()
  private var claimed = false
  func claim() -> Bool {
    lock.lock()
    defer { lock.unlock() }
    if claimed { return false }
    claimed = true
    return true
  }
}

private struct TestError: Error {}

/// An `ItemStore` that always throws — drives the failure path (#816).
private struct FailingStore: ItemStore {
  func loadItems() throws -> [Item] { throw TestError() }
  func save(_ item: Item) throws { throw TestError() }
  func save(_ items: [Item]) throws { throw TestError() }
  func delete(id: String, deletedAt: String) throws { throw TestError() }
  func loadSessions() throws -> [PracticeSession] { throw TestError() }
  func saveSession(_ session: PracticeSession) throws { throw TestError() }
}
