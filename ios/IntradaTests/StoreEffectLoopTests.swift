import IntradaCoreFFI
import SharedTypes
import XCTest

@testable import Intrada

@MainActor
final class StoreEffectLoopTests: XCTestCase {

  override func tearDown() {
    MockURLProtocol.handler = nil
    MockURLProtocol.lastRequest = nil
    super.tearDown()
  }

  // ── Effect dispatch ────────────────────────────────────────────────────

  func testInitRendersInitialViewModel() {
    let bridge = FakeBridge()
    let store = Store(bridge: bridge, session: mockSession())
    XCTAssertNotNil(store.viewModel, "init should seed the ViewModel from the bridge")
    XCTAssertEqual(bridge.viewCallCount, 1)
  }

  func testRenderEffectRefreshesViewModel() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 1, effect: .render(RenderOperation()))] }
    let store = Store(bridge: bridge, session: mockSession())

    bridge.nextViewModel = {
      var vm = try emptyViewModel()
      vm.error = "refreshed"
      return vm
    }
    store.send(.setQuery(nil))

    XCTAssertEqual(store.viewModel?.error, "refreshed", "render effect should re-read view()")
  }

  func testAppEffectIsNotResolved() {
    // Why never resolve: testRealBridgeAppEffectIsNeverResolved (#882).
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 7, effect: .app(.clearSessionInProgress))] }
    let store = Store(bridge: bridge, session: mockSession())

    store.send(.setQuery(nil))

    XCTAssertTrue(bridge.emptyResolved.isEmpty, "app effect must not be resolved")
  }

  func testPersistenceLoadResolvesFromStore() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 8, effect: .persistence(.loadItems))] }
    let store = Store(bridge: bridge, session: mockSession())

    store.send(.setQuery(nil))

    XCTAssertEqual(bridge.persistenceResolved.first?.id, 8)
    guard case .items(let items) = bridge.persistenceResolved.first?.output else {
      return XCTFail(
        "expected .items, got \(String(describing: bridge.persistenceResolved.first?.output))")
    }
    XCTAssertTrue(items.isEmpty, "fresh in-memory store has no rows")
  }

  func testPersistenceWriteFailureResolvesFailed() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 9, effect: .persistence(.saveItem(Self.sampleItem)))]
    }
    let store = Store(bridge: bridge, session: mockSession(), store: FailingStore())

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
    let store = Store(bridge: bridge, session: mockSession())

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
    let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)

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
    let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)

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
    let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)

    store.restorePersistedSort()

    XCTAssertTrue(sentEvents.isEmpty, "no stored sort → no event")
  }

  // ── Failure-soft (guarded) ─────────────────────────────────────────────

  func testUpdateThrowIsSwallowedWithoutCrashing() {
    let bridge = FakeBridge()
    bridge.throwOnUpdate = TestError()
    let store = Store(bridge: bridge, session: mockSession())

    store.send(.setQuery(nil))

    XCTAssertTrue(bridge.resolved.isEmpty)
    XCTAssertTrue(bridge.emptyResolved.isEmpty)
    XCTAssertNotNil(store.viewModel, "a thrown update should fail soft, not wipe the ViewModel")
  }

  func testViewThrowAtInitLeavesViewModelNil() {
    let bridge = FakeBridge()
    bridge.throwOnView = TestError()
    let store = Store(bridge: bridge, session: mockSession())

    XCTAssertNil(store.viewModel, "a thrown view() should leave nil (loading state), not crash")
  }

  // ── HTTP execution + result mapping ────────────────────────────────────

  func testHttpEffectMapsOkResponse() async {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [
        Request(
          id: 3,
          effect: .http(
            HttpRequest(method: "GET", url: "https://x.test/items", headers: [], body: [])))
      ]
    }
    MockURLProtocol.handler = { request in
      let response = HTTPURLResponse(
        url: request.url!, statusCode: 201,
        httpVersion: nil, headerFields: ["X-Test": "yes"])!
      return (response, Data("hello".utf8))
    }
    let store = Store(bridge: bridge, session: mockSession())

    await whenResolved(bridge) { store.send(.setQuery(nil)) }

    guard case .ok(let response) = bridge.resolved.first?.result else {
      return XCTFail("expected .ok, got \(String(describing: bridge.resolved.first?.result))")
    }
    XCTAssertEqual(bridge.resolved.first?.id, 3)
    XCTAssertEqual(response.status, 201)
    XCTAssertEqual(response.body, [UInt8]("hello".utf8))
    XCTAssertTrue(
      response.headers.contains { $0.name == "X-Test" && $0.value == "yes" },
      "server headers should map into HttpResponse")
  }

  func testHttpEffectMapsNetworkErrorToIo() async {
    let bridge = httpBridge()
    MockURLProtocol.handler = { _ in throw URLError(.notConnectedToInternet) }
    let store = Store(bridge: bridge, session: mockSession())

    await whenResolved(bridge) { store.send(.setQuery(nil)) }

    guard case .err(.io) = bridge.resolved.first?.result else {
      return XCTFail("expected .err(.io), got \(String(describing: bridge.resolved.first?.result))")
    }
  }

  func testHttpEffectMapsTimeout() async {
    let bridge = httpBridge()
    MockURLProtocol.handler = { _ in throw URLError(.timedOut) }
    let store = Store(bridge: bridge, session: mockSession())

    await whenResolved(bridge) { store.send(.setQuery(nil)) }

    XCTAssertEqual(bridge.resolved.first?.result, .err(.timeout))
  }

  func testHttpEffectMapsInvalidUrl() async {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [
        Request(
          id: 9,
          effect: .http(HttpRequest(method: "GET", url: "h ttp://nope", headers: [], body: [])))
      ]
    }
    let store = Store(bridge: bridge, session: mockSession())

    await whenResolved(bridge) { store.send(.setQuery(nil)) }

    guard case .err(.url) = bridge.resolved.first?.result else {
      return XCTFail(
        "expected .err(.url), got \(String(describing: bridge.resolved.first?.result))")
    }
  }

  func testHttpRequestMapsMethodAndHeaders() async {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [
        Request(
          id: 5,
          effect: .http(
            HttpRequest(
              method: "POST", url: "https://x.test/items",
              headers: [HttpHeader(name: "Content-Type", value: "application/json")],
              body: [1, 2, 3])))
      ]
    }
    MockURLProtocol.handler = { request in
      MockURLProtocol.lastRequest = request
      return (
        HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!,
        Data()
      )
    }
    let store = Store(bridge: bridge, session: mockSession())

    await whenResolved(bridge) { store.send(.setQuery(nil)) }

    XCTAssertEqual(MockURLProtocol.lastRequest?.httpMethod, "POST")
    XCTAssertEqual(
      MockURLProtocol.lastRequest?.value(forHTTPHeaderField: "Content-Type"), "application/json")
  }

  func testResolveChainedRenderRefreshesView() async {
    let bridge = httpBridge()
    bridge.resolveHandler = { _, _ in [Request(id: 2, effect: .render(RenderOperation()))] }
    MockURLProtocol.handler = { request in
      (
        HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!,
        Data()
      )
    }
    let store = Store(bridge: bridge, session: mockSession())

    bridge.nextViewModel = {
      var vm = try emptyViewModel()
      vm.error = "post-resolve"
      return vm
    }
    await whenResolved(bridge) { store.send(.setQuery(nil)) }

    XCTAssertEqual(
      store.viewModel?.error, "post-resolve", "render from a resolve should refresh view")
  }

  // ── Real bridge (Swift↔Rust bincode round-trip) ────────────────────────

  /// Real-bridge bincode round-trip (#846): calls LiveBridge directly (not via
  /// Store) so a serialization throw surfaces instead of being swallowed by
  /// Store.send's `guarded`.
  func testRealBridgeEditAppliesToViewModel() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Original", kind: .piece, composer: "Bach", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: []))))

    let afterAdd = try bridge.view()
    XCTAssertEqual(
      afterAdd.items.count, 1,
      "add should land: count=\(afterAdd.items.count) err=\(afterAdd.error ?? "nil")")
    let id = try XCTUnwrap(afterAdd.items.first?.id)

    // Mirrors ItemFormModel.updateInput(): every PATCH field set, type flipped.
    _ = try bridge.update(
      .item(
        .update(
          id: id,
          input: UpdateItem(
            title: "Renamed", kind: .exercise, composer: .some("Bach"), key: .some(nil),
            modality: .some(nil), tempo: .some(nil), notes: .some(nil), tags: nil, priority: nil))))

    let afterEdit = try bridge.view()
    XCTAssertEqual(
      afterEdit.items.first?.title, "Renamed",
      "edited title should apply (err=\(afterEdit.error ?? "nil"))")
    XCTAssertEqual(afterEdit.items.first?.itemType, .exercise, "edited type should apply")
  }

  /// Real-bridge priority toggle (#763): the star sends an UpdateItem with every
  /// optional field "no change" (outer nil) and only `priority` set — a different
  /// bincode shape than the full edit, so round-trip it through the live bridge to
  /// catch an absent-vs-present wire break (#846).
  func testRealBridgePriorityToggleAppliesToViewModel() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Etude", kind: .piece, composer: "Chopin", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: []))))
    let item = try XCTUnwrap(try bridge.view().items.first)
    XCTAssertFalse(item.priority, "new items start non-priority")

    func toggle(_ on: Bool) -> Event {
      .item(
        .update(
          id: item.id,
          input: UpdateItem(
            title: item.title, kind: item.itemType,
            composer: nil, key: nil, modality: nil, tempo: nil, notes: nil,
            tags: nil, priority: on)))
    }

    _ = try bridge.update(toggle(true))
    let on = try bridge.view().items.first
    XCTAssertEqual(on?.priority, true, "star should flip priority on")
    XCTAssertEqual(on?.subtitle, "Chopin", "a priority-only update must not clobber other fields")

    _ = try bridge.update(toggle(false))
    XCTAssertEqual(try bridge.view().items.first?.priority, false, "star should flip priority off")
  }

  /// "Practise this" (#1034): StartBuildingWith is a new bridge-crossing
  /// write — round-trip it through the real bincode bridge (#846).
  func testRealBridgePractiseThisSeedsBuilder() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Hanon No. 1", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: []))))
    let itemId = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.session(.startBuildingWith(itemId: itemId)))
    let vm = try bridge.view()
    XCTAssertNotNil(vm.buildingSetlist, "startBuildingWith should open a seeded setlist")
    XCTAssertEqual(vm.buildingSetlist?.entries.count, 1)
    XCTAssertEqual(vm.buildingSetlist?.entries.first?.itemId, itemId)
    XCTAssertNil(vm.error)
  }

  /// Real-bridge build→play→save lifecycle (#932): drives the actual bincode
  /// bridge through Building → Active → Summary → Idle, mirroring the
  /// SessionBuilder → FocusPlayer → Summary screens. A wire break surfaces here
  /// as a failed transition instead of the silent no-op the stub bridge would
  /// hide (#846).
  func testRealBridgeSessionFlowBuildPlaySave() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Etude", kind: .piece, composer: "Chopin", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: []))))
    let itemId = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.session(.startBuilding))
    _ = try bridge.update(.session(.addToSetlist(itemId: itemId)))
    let building = try bridge.view()
    XCTAssertNotNil(building.buildingSetlist, "startBuilding + add should open a setlist")
    XCTAssertEqual(building.buildingSetlist?.entries.count, 1)
    XCTAssertNil(building.activeSession)

    _ = try bridge.update(.session(.startSession(now: "2026-06-16T10:00:00Z")))
    let active = try bridge.view()
    XCTAssertNotNil(active.activeSession, "startSession should enter the player")
    XCTAssertNil(active.buildingSetlist, "the builder should close on start")
    XCTAssertNil(active.summary)

    // The FocusPlayer reaches the summary by advancing past the last item (its
    // Done/Finish path), not finishSession — round-trip the event the screen
    // actually sends.
    _ = try bridge.update(.session(.nextItem(now: "2026-06-16T10:20:00Z")))
    let summary = try bridge.view()
    XCTAssertNotNil(summary.summary, "advancing past the last item should reach the summary")
    XCTAssertNil(summary.activeSession)

    // Optional-payload events crossing bincode (the absent-vs-present wire
    // hazard, #846): set then clear a score and the session notes.
    let entryId = try XCTUnwrap(summary.summary?.entries.first?.id)
    _ = try bridge.update(.session(.updateEntryScore(entryId: entryId, score: 4)))
    XCTAssertEqual(try bridge.view().summary?.entries.first?.score, 4, "score should round-trip")
    _ = try bridge.update(.session(.updateEntryScore(entryId: entryId, score: nil)))
    XCTAssertNil(try bridge.view().summary?.entries.first?.score, "clearing a score round-trips")
    // Per-entry notes — the hand-off reflection sheet's write, never previously
    // sent from Swift (#846). Round-trip set + clear through the live bridge.
    _ = try bridge.update(
      .session(.updateEntryNotes(entryId: entryId, notes: "RH evenness better at 96")))
    XCTAssertEqual(
      try bridge.view().summary?.entries.first?.notes, "RH evenness better at 96",
      "the reflection sheet's per-entry note should round-trip")
    _ = try bridge.update(.session(.updateEntryNotes(entryId: entryId, notes: nil)))
    XCTAssertNil(
      try bridge.view().summary?.entries.first?.notes, "clearing an entry note round-trips")
    _ = try bridge.update(.session(.updateSessionNotes(notes: "Felt good")))
    XCTAssertEqual(try bridge.view().summary?.notes, "Felt good", "notes should round-trip")
    _ = try bridge.update(.session(.updateSessionNotes(notes: nil)))
    XCTAssertNil(try bridge.view().summary?.notes, "clearing notes round-trips")

    _ = try bridge.update(.session(.saveSession(now: "2026-06-16T10:20:30Z")))
    let saved = try bridge.view()
    XCTAssertNil(saved.summary, "saveSession clears the summary (session persisted)")
    XCTAssertNil(saved.activeSession)
    XCTAssertNil(saved.error, "a clean save surfaces no error")
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

  private func httpBridge() -> FakeBridge {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [
        Request(
          id: 4,
          effect: .http(
            HttpRequest(method: "GET", url: "https://x.test/items", headers: [], body: [])))
      ]
    }
    return bridge
  }

  /// Resume on the bridge's `resolve` callback, not a wall-clock `fulfillment`
  /// — a loaded CI runner starves the detached HTTP Task; a tight ceiling flakes
  /// (#956, #861). The 30s net is a fail-bounded backstop, not the happy path.
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

  static let sampleItem = Item(
    id: "p1", title: "Etude", kind: .piece, composer: "Chopin", key: nil, modality: nil,
    tempo: nil,
    notes: nil, tags: [], linkedExerciseIds: [], createdAt: "2026-01-01T00:00:00Z",
    updatedAt: "2026-01-01T00:00:00Z", priority: false)

  private func mockSession() -> URLSession {
    let config = URLSessionConfiguration.ephemeral
    config.protocolClasses = [MockURLProtocol.self]
    return URLSession(configuration: config)
  }
}

private func emptyViewModel() throws -> ViewModel {
  try ViewModel.bincodeDeserialize(input: [UInt8](CoreFfi().view()))
}

private final class FakeBridge: CoreBridge {
  var updateHandler: (Event) -> [Request] = { _ in [] }
  var resolveHandler: (UInt32, HttpResult) -> [Request] = { _, _ in [] }
  var nextViewModel: (() throws -> ViewModel)?
  var onResolve: (() -> Void)?
  /// When set, the corresponding bridge call throws — drives the Store's
  /// `guarded` failure-soft path.
  var throwOnUpdate: Error?
  var throwOnView: Error?

  private(set) var events: [Event] = []
  private(set) var resolved: [(id: UInt32, result: HttpResult)] = []
  private(set) var persistenceResolved: [(id: UInt32, output: PersistenceOutput)] = []
  private(set) var emptyResolved: [UInt32] = []
  private(set) var viewCallCount = 0

  func update(_ event: Event) throws -> [Request] {
    events.append(event)
    if let throwOnUpdate { throw throwOnUpdate }
    return updateHandler(event)
  }

  func resolve(_ id: UInt32, httpResult: HttpResult) throws -> [Request] {
    resolved.append((id, httpResult))
    onResolve?()
    return resolveHandler(id, httpResult)
  }

  func resolve(_ id: UInt32, persistenceOutput: PersistenceOutput) throws -> [Request] {
    persistenceResolved.append((id, persistenceOutput))
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
  func delete(id: String, deletedAt: String) throws { throw TestError() }
  func loadSessions() throws -> [PracticeSession] { throw TestError() }
  func saveSession(_ session: PracticeSession) throws { throw TestError() }
}

final class MockURLProtocol: URLProtocol {
  nonisolated(unsafe) static var handler: ((URLRequest) throws -> (HTTPURLResponse, Data))?
  nonisolated(unsafe) static var lastRequest: URLRequest?

  override class func canInit(with request: URLRequest) -> Bool { true }
  override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

  override func startLoading() {
    guard let handler = Self.handler else {
      client?.urlProtocol(self, didFailWithError: URLError(.unknown))
      return
    }
    do {
      let (response, data) = try handler(request)
      client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
      client?.urlProtocol(self, didLoad: data)
      client?.urlProtocolDidFinishLoading(self)
    } catch {
      client?.urlProtocol(self, didFailWithError: error)
    }
  }

  override func stopLoading() {}
}
