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

  func testSaveSessionInProgressEffectWritesBlobAndClearRemovesIt() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "sip-\(UUID().uuidString)"))
    let active = ActiveSession(
      id: "s-crash", entries: [], currentIndex: 0,
      currentItemStartedAt: "2026-07-14T10:00:00Z", sessionStartedAt: "2026-07-14T10:00:00Z",
      sessionIntention: "even RH at 96")
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 1, effect: .app(.saveSessionInProgress(active)))]
    }
    let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)

    store.send(.setQuery(nil))

    let pending = try XCTUnwrap(
      store.pendingSessionInProgress(), "the save effect must persist a recoverable blob")
    XCTAssertEqual(pending.id, "s-crash")
    XCTAssertEqual(pending.sessionIntention, "even RH at 96")

    bridge.updateHandler = { _ in [Request(id: 2, effect: .app(.clearSessionInProgress))] }
    store.send(.setQuery(nil))
    XCTAssertNil(
      store.pendingSessionInProgress(), "the clear effect must remove the recoverable blob")
  }

  func testDiscardSessionInProgressRemovesBlobWithoutCoreEvent() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "sip-\(UUID().uuidString)"))
    let active = ActiveSession(
      id: "s-stale", entries: [], currentIndex: 0,
      currentItemStartedAt: "2026-07-14T10:00:00Z", sessionStartedAt: "2026-07-14T10:00:00Z",
      sessionIntention: nil)
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 1, effect: .app(.saveSessionInProgress(active)))]
    }
    let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)
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

  private static let coachBlock = BlockRecord(
    id: "b1", node: "rootless-a-b", drill: "shell-voicings", gate: "rootless-under-melody",
    level: ParameterLevel(tempoBpm: 92, clickLevel: .twoAndFour), circle: .hands, mode: .keys,
    startedAt: "2026-08-04T10:00:00Z", endedAt: "2026-08-04T10:00:30Z",
    attempts: [
      AttemptSummary(
        at: "2026-08-04T10:00:09Z", verdict: .clean, source: .tapVerdict, cold: true,
        selfPredicted: nil, level: ParameterLevel(tempoBpm: 92, clickLevel: .twoAndFour))
    ],
    attemptsToPass: 3, gateOpenedAtAttempt: 3, repsAfterGate: 0, activeMs: 30_000,
    escalationFired: [], exit: .gatePassed, origin: .authored)

  private static func saveCoachRecords(id: UInt32) -> Request {
    Request(
      id: id,
      effect: .persistence(
        .saveCoachRecords(
          blocks: [coachBlock], wanders: [], playThroughs: [], updatedAt: "2026-08-04T10:00:30Z")))
  }

  func testCoachRecordsWriteResolvesAck() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Self.saveCoachRecords(id: 20)] }
    let store = Store(bridge: bridge, session: mockSession())

    store.send(.setQuery(nil))

    XCTAssertEqual(bridge.persistenceResolved.first?.id, 20)
    XCTAssertEqual(
      bridge.persistenceResolved.first?.output, .ack,
      "a successful evidence write resolves .ack")
  }

  func testCoachRecordsWriteFailureResolvesFailed() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Self.saveCoachRecords(id: 21)] }
    let store = Store(bridge: bridge, session: mockSession(), store: FailingStore())

    store.send(.setQuery(nil))

    XCTAssertEqual(
      bridge.persistenceResolved.first?.output, .failed,
      "a lost block must surface as .failed, never a phantom .ack (#816)")
  }

  func testCoachRecordsReadResolvesWhatWasWritten() throws {
    let libraryStore = try LibraryStore.inMemory()
    try libraryStore.saveCoachRecords(
      blocks: [Self.coachBlock], wanders: [], playThroughs: [], updatedAt: "2026-08-04T10:00:30Z")
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 22, effect: .persistence(.loadCoachRecords))]
    }
    let store = Store(bridge: bridge, session: mockSession(), store: libraryStore)

    store.send(.setQuery(nil))

    XCTAssertEqual(
      bridge.persistenceResolved.first?.output, .coachRecords([Self.coachBlock]),
      "the launch read hands the core back the evidence it wrote (#1214)")
  }

  func testCoachRecordsReadFailureResolvesFailed() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 23, effect: .persistence(.loadCoachRecords))] }
    let store = Store(bridge: bridge, session: mockSession(), store: FailingStore())

    store.send(.setQuery(nil))

    XCTAssertEqual(
      bridge.persistenceResolved.first?.output, .failed,
      "a failed read surfaces, so the core never rebuilds mastery from a lie (#816)")
  }

  /// The session the core just snapshotted for crash recovery — also the only
  /// route to its `EngineConfig`, which `CoachView` does not carry.
  private func coachSession(in requests: [Request]) throws -> EngineSession {
    let session = requests.lazy.compactMap { request -> EngineSession? in
      guard case .app(.saveCoachSessionInProgress(let session)) = request.effect else { return nil }
      return session
    }.first
    return try XCTUnwrap(
      session,
      "no SaveCoachSessionInProgress effect: the core stopped snapshotting the session")
  }

  /// The in-progress session as the engine actually builds it, rather than a
  /// hand-made one: recovery depends on this exact shape surviving the wire.
  private func runningCoachSession() throws -> EngineSession {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    return try coachSession(
      in: try bridge.update(.coach(.startPlannedSession(now: "2026-08-04T10:00:00Z"))))
  }

  func testSaveCoachSessionEffectWritesBlobAndClearRemovesIt() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "coach-\(UUID().uuidString)"))
    let session = try runningCoachSession()
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 1, effect: .app(.saveCoachSessionInProgress(session)))]
    }
    let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)

    store.send(.setQuery(nil))

    let pending = try XCTUnwrap(
      store.pendingCoachSession(), "the save effect must persist a recoverable blob")
    XCTAssertEqual(
      pending, session,
      "the blob must round-trip the whole session, plan and block state included")

    bridge.updateHandler = { _ in [Request(id: 2, effect: .app(.clearCoachSessionInProgress))] }
    store.send(.setQuery(nil))
    XCTAssertNil(store.pendingCoachSession(), "a closed session's blob would recover nothing")
  }

  /// Real-bridge recovery round trip (#846 class, #1181): the blob leaves the
  /// core as bincode, sits in UserDefaults, and goes back in as a `CoachEvent`
  /// payload. A stub bridge cannot catch a mismatch on that wire — the symptom
  /// would be a recovery that silently no-ops and discards the evidence.
  func testRealBridgeRecoversAnInProgressCoachSessionFromItsStoredBlob() throws {
    let crashed = try runningCoachSession()
    let blob = Data(try crashed.bincodeSerialize())
    let restored = try EngineSession.bincodeDeserialize(input: [UInt8](blob))

    // A fresh core, as after a relaunch: it knows nothing until the blob lands.
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    XCTAssertNil(try bridge.view().coach.drill, "a fresh core has no drill running")

    _ = try bridge.update(
      .coach(.recoverSession(session: restored, now: "2026-08-04T11:00:00Z")))

    let drill = try XCTUnwrap(
      try bridge.view().coach.drill, "the recovered session must put the drill back on screen")
    XCTAssertEqual(drill.blockIndex, 0)
    XCTAssertFalse(drill.drillTitle.isEmpty, "the recovered block keeps its identity")
  }

  func testPendingCoachSessionIsNilWithNoStoredBlob() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "coach-\(UUID().uuidString)"))
    let store = Store(bridge: FakeBridge(), session: mockSession(), sortDefaults: defaults)
    XCTAssertNil(store.pendingCoachSession())
  }

  /// An `EngineSession` field change makes every blob written by the previous
  /// build undecodable. That must lose the recovery, not the launch: bincode is
  /// positional, so there is no forward compatibility to lean on here.
  func testPendingCoachSessionIsNilForAnUndecodableBlob() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "coach-\(UUID().uuidString)"))
    defaults.set(Data([0xff, 0x00, 0x2a]), forKey: Store.coachSessionInProgressKey)
    let store = Store(bridge: FakeBridge(), session: mockSession(), sortDefaults: defaults)
    XCTAssertNil(store.pendingCoachSession(), "a stale blob degrades to a fresh start")
  }

  /// #1223 renumbered `Phase` and added a byte mid-`BlockState`, so a previous
  /// build's blob cannot decode. The key bump makes that a designed absence
  /// rather than a logged decode failure.
  func testAPreviousVersionsCoachBlobIsNeverOffered() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "coach-\(UUID().uuidString)"))
    let stale = Data(try runningCoachSession().bincodeSerialize())
    let retired = try XCTUnwrap(Store.retiredCoachSessionKeys.first)
    defaults.set(stale, forKey: retired)

    let store = Store(bridge: FakeBridge(), session: mockSession(), sortDefaults: defaults)
    XCTAssertNil(
      store.pendingCoachSession(), "a blob under a retired key is not a recovery candidate")
  }

  func testWritingTheCurrentCoachBlobDropsRetiredOnes() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "coach-\(UUID().uuidString)"))
    let retired = try XCTUnwrap(Store.retiredCoachSessionKeys.first)
    defaults.set(Data([0xff, 0x00, 0x2a]), forKey: retired)

    let session = try runningCoachSession()
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in
      [Request(id: 1, effect: .app(.saveCoachSessionInProgress(session)))]
    }
    let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)
    store.send(.setQuery(nil))

    XCTAssertNil(defaults.data(forKey: retired), "the dead blob is gone, not merely ignored")
    XCTAssertNotNil(store.pendingCoachSession(), "and the current one is there")
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

  /// Real-bridge step round-trip (#846, #1083): `AddVariant` pushes a `Variant`
  /// onto the exercise, which then rides the whole `Item` and the derived
  /// `steps`/`currentVariantId` back across the bincode wire — a shape the stub
  /// bridge can't exercise. A wire break would drop the ladder silently.
  func testRealBridgeAddVariantSurfacesStepsInViewModel() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.item(.addVariant(itemId: id, label: "F major")))
    _ = try bridge.update(.item(.addVariant(itemId: id, label: "Bb major")))

    let view = try bridge.view()
    let ex = try XCTUnwrap(view.items.first { $0.id == id })
    XCTAssertEqual(
      ex.variants.map(\.label), ["F major", "Bb major"],
      "steps round-trip the live bridge in ladder order (err=\(view.error ?? "nil"))")
    XCTAssertFalse(ex.variants.contains { $0.isSolid }, "unpractised steps aren't solid")
    XCTAssertEqual(
      ex.variants.first?.isCurrent, true,
      "current step is the first not-yet-solid step")
  }

  /// Real-bridge built-session write (#846, #1256): `CreateUserDrill` crosses
  /// the wire, the core mints the entity, and the matching `SaveUserDrill`
  /// operation comes back across the same wire. A stub bridge can't catch a
  /// bincode break on either leg.
  func testRealBridgeCreateUserDrillEmitsMatchingSaveOp() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    let requests = try bridge.update(
      .builtSession(
        .createUserDrill(
          CreateUserDrill(
            name: "Descending run", criterion: "Three clean passes at 72",
            tempoBpm: 72, keys: ["F"], passesToOpen: 3, serves: .node("alice-bridge")))))

    let drill = try XCTUnwrap(
      requests.compactMap { request -> UserDrill? in
        if case .persistence(.saveUserDrill(let drill)) = request.effect { return drill }
        return nil
      }.first, "CreateUserDrill must emit a SaveUserDrill persistence op")
    XCTAssertEqual(drill.id.count, 26, "core-minted ulid")
    XCTAssertEqual(drill.criterion, "Three clean passes at 72")
    XCTAssertEqual(drill.tempoBpm, 72)
    XCTAssertEqual(drill.serves, .node("alice-bridge"))
    XCTAssertNil(drill.deletedAt)
  }

  /// Real-bridge built-session composition (#846, #1256): every `BuiltTarget`
  /// kind rides one `BuiltSession` across the wire and back out in the
  /// `SaveBuiltSession` op — the enum-variant shape a stub can't exercise.
  func testRealBridgeSaveBuiltSessionRoundTripsEveryTargetKind() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    let blocks = [
      BuiltBlock(id: "b1", target: .node(node: "shell-voicings"), minutes: 5),
      BuiltBlock(id: "b2", target: .userDrill(drillId: "d1"), minutes: nil),
      BuiltBlock(id: "b3", target: .journal(journalId: "j1"), minutes: 8),
      BuiltBlock(id: "b4", target: .piece(itemId: "p1"), minutes: 10),
    ]
    let requests = try bridge.update(
      .builtSession(
        .saveBuiltSession(
          session: BuiltSession(
            id: "01BUILT0000000000000000001", source: "From Friday's lesson", blocks: blocks,
            createdAt: "2026-08-07T10:00:00Z", updatedAt: "2026-08-07T10:00:00Z",
            deletedAt: nil))))

    let saved = try XCTUnwrap(
      requests.compactMap { request -> BuiltSession? in
        if case .persistence(.saveBuiltSession(let session)) = request.effect { return session }
        return nil
      }.first, "SaveBuiltSession must emit its persistence op")
    XCTAssertEqual(saved.blocks, blocks, "all four target kinds survive the wire")
    XCTAssertEqual(saved.source, "From Friday's lesson")
  }

  /// Real-bridge steer sheet (#846, #1256, Phase B): the whole of Journey A
  /// over the *real* wire — a name in, a question back through the ViewModel, a
  /// drill created, a session built and started. Every one of those events and
  /// views is a new bincode shape, and a stub bridge cannot break on any of them.
  func testRealBridgeComposesResolvesAndStartsABuiltSession() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(.builtSession(.openCompose))
    _ = try bridge.update(
      .builtSession(.addComposeEntry(text: "Zzz stride pattern", pickedItemId: nil)))

    let asking = try bridge.view().built.compose
    let question = try XCTUnwrap(asking?.questions.first, "an unknown name owes one question")
    guard case .userDrill(let criterion, _, _, let passes, _) = question.ask else {
      return XCTFail("expected the criterion form, got \(question.ask)")
    }
    XCTAssertEqual(criterion, "Zzz stride pattern", "the field opens on what was said")
    XCTAssertEqual(passes, 3)
    XCTAssertEqual(asking?.canBuild, false, "the price was stated, so it must be paid")

    let created = try bridge.update(
      .builtSession(
        .resolveAsUserDrill(
          entryId: question.entryId, criterion: "Three clean passes at 72",
          serves: .circle(.hands))))
    let drill = try XCTUnwrap(
      created.compactMap { request -> UserDrill? in
        if case .persistence(.saveUserDrill(let drill)) = request.effect { return drill }
        return nil
      }.first, "the answer creates the drill")
    XCTAssertEqual(drill.tempoBpm, 72, "parsed from the sentence, never asked for")

    let built = try bridge.update(.builtSession(.buildSession(source: "From Friday's lesson")))
    let session = try XCTUnwrap(
      built.compactMap { request -> BuiltSession? in
        if case .persistence(.saveBuiltSession(let session)) = request.effect { return session }
        return nil
      }.first, "building persists the composition")
    XCTAssertEqual(session.blocks.count, 1)

    let composed = try XCTUnwrap(try bridge.view().built.session, "A6 reads back over the wire")
    XCTAssertEqual(composed.source, "From Friday's lesson")
    XCTAssertEqual(composed.blocks.first?.kind, .exercise)

    _ = try bridge.update(
      .builtSession(
        .startBuiltSession(sessionId: session.id, now: "2026-08-07T10:00:00Z")))
    let drillView = try XCTUnwrap(try bridge.view().coach.drill, "the loop is running")
    XCTAssertEqual(drillView.origin, .userDrill, "the new origin field survives the wire")
    XCTAssertEqual(drillView.serves, "Adds to what your hands know")
    XCTAssertEqual(drillView.gateTarget, 3, "the sentence's passes became the gate")
  }

  /// Real-bridge reflection + feel (#846, #1256): `RecordReflection` carries
  /// three optional Strings (the absent-vs-present hazard) and `RecordFeel` a
  /// fieldless enum; both must decode on the wire and emit their ops.
  func testRealBridgeReflectionAndFeelDecodeOnWire() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    let reflectionRequests = try bridge.update(
      .builtSession(
        .recordReflection(
          kind: .sessionClose, sessionRef: nil,
          transcript: "The bridge still rushes", audioPath: nil, durationS: 24)))
    let reflection = try XCTUnwrap(
      reflectionRequests.compactMap { request -> Reflection? in
        if case .persistence(.saveReflection(let reflection)) = request.effect {
          return reflection
        }
        return nil
      }.first)
    XCTAssertEqual(reflection.transcript, "The bridge still rushes")
    XCTAssertNil(reflection.audioPath)

    let feelRequests = try bridge.update(
      .builtSession(.recordFeel(blockId: "b1", feel: .itSang)))
    let entry = try XCTUnwrap(
      feelRequests.compactMap { request -> FeelEntry? in
        if case .persistence(.saveFeelEntry(let entry)) = request.effect { return entry }
        return nil
      }.first)
    XCTAssertEqual(entry.feel, .itSang)
  }

  /// Real-bridge hydration (#846, #1256): the launch `LoadBuiltSessionData`
  /// request resolves with a fully-populated `BuiltSessionData` — the read
  /// leg of the wire, which must decode in Rust without error.
  func testRealBridgeLoadBuiltSessionDataResolvesPopulatedPayload() throws {
    let bridge = LiveBridge()
    let requests = try bridge.update(
      .startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    let load = try XCTUnwrap(
      requests.first { request in
        if case .persistence(.loadBuiltSessionData) = request.effect { return true }
        return false
      }, "local-first launch must hydrate the built-session store")

    let data = BuiltSessionData(
      userDrills: [
        UserDrill(
          id: "01USERDRILL0000000000000001", name: "Descending run",
          criterion: "Three clean passes", tempoBpm: nil, keys: [], passesToOpen: 3,
          serves: .circle(.hands), createdAt: "2026-08-07T10:00:00Z",
          updatedAt: "2026-08-07T10:00:00Z", deletedAt: nil)
      ],
      journalItems: [
        JournalItem(
          id: "01JOURNAL000000000000000001", name: "Rubato feel", notes: "Time and notes",
          linkedItemId: "p1", createdAt: "2026-08-07T10:00:00Z",
          updatedAt: "2026-08-07T10:00:00Z", deletedAt: nil)
      ],
      builtSessions: [],
      playThroughs: [
        PlayThroughRecord(
          id: "01PLAY00000000000000000001", itemId: "p1",
          startedAt: "2026-08-07T10:00:00Z", endedAt: "2026-08-07T10:04:00Z", counted: false,
          sections: [SectionVerdict(section: "The bridge", held: true, at: "2026-08-07T10:01:00Z")],
          updatedAt: "2026-08-07T10:04:00Z", deletedAt: nil)
      ],
      reflections: [],
      feelEntries: [])
    // A wire break on the read leg throws here rather than landing data.
    _ = try bridge.resolve(load.id, persistenceOutput: .builtSessionData(data))
    let view = try bridge.view()
    XCTAssertNil(view.error, "hydration must not surface an error")
  }

  /// Real-bridge journal write (#846, #1256): `CreateJournalItem` carries two
  /// optional Strings, the absent-vs-present hazard on a positional wire.
  func testRealBridgeJournalWriteDecodesOnWire() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))

    let journalRequests = try bridge.update(
      .builtSession(
        .createJournalItem(
          CreateJournalItem(name: "Rubato feel", notes: nil, linkedItemId: "p1"))))
    let journal = try XCTUnwrap(
      journalRequests.compactMap { request -> JournalItem? in
        if case .persistence(.saveJournalItem(let journal)) = request.effect { return journal }
        return nil
      }.first, "CreateJournalItem must emit a SaveJournalItem persistence op")
    XCTAssertEqual(journal.id.count, 26, "core-minted ulid")
    XCTAssertNil(journal.notes, "an absent optional stays absent across the wire")
    XCTAssertEqual(journal.linkedItemId, "p1")
  }

  /// Real-bridge run-through (#846, #1256 Phase C): the whole altitude, driven
  /// over the live bridge to the batch it writes. `SaveCoachRecords` carries a
  /// nested struct array (`playThroughs[].sections[]`) that a stub bridge could
  /// not catch a wire break in.
  func testRealBridgeRunThroughDecodesOnWire() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))

    let addRequests = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Alice in Wonderland", kind: .piece, composer: "Sammy Fain", key: nil,
            modality: nil, tempo: nil, notes: nil, tags: []))))
    let pieceId = try XCTUnwrap(
      addRequests.compactMap { request -> String? in
        if case .persistence(.saveItem(let item)) = request.effect { return item.id }
        return nil
      }.first, "the piece is written with a core-minted id")
    _ = try bridge.update(
      .item(.setChordChart(pieceId: pieceId, rawChart: "[A]\n| Cmaj7 | Am7 |\n[B]\n| Dm7 | G7 |")))

    _ = try bridge.update(
      .builtSession(
        .startPlayThrough(
          itemId: pieceId, altitude: .runThrough, now: "2026-08-07T10:00:00Z")))
    let run = try XCTUnwrap(
      try bridge.view().coach.runThrough, "the run-through screen has something to draw")
    XCTAssertEqual(run.sections, ["A", "B"], "the piece's own named sections cross the wire")
    XCTAssertEqual(try bridge.view().coach.altitude, .runThrough)

    _ = try bridge.update(.coach(.judgeSection(held: true, now: "2026-08-07T10:01:00Z")))
    _ = try bridge.update(.coach(.judgeSection(held: false, now: "2026-08-07T10:03:00Z")))
    let closing = try bridge.update(.coach(.closeSession(now: "2026-08-07T10:04:00Z")))

    let records = try XCTUnwrap(
      closing.compactMap { request -> [PlayThroughRecord]? in
        if case .persistence(.saveCoachRecords(_, _, let playThroughs, _)) = request.effect {
          return playThroughs
        }
        return nil
      }.first, "a closed run must ride the coach batch")
    let record = try XCTUnwrap(records.first)
    XCTAssertEqual(record.id.count, 26, "core-minted ulid")
    XCTAssertEqual(record.itemId, pieceId)
    XCTAssertTrue(record.counted)
    XCTAssertEqual(record.sections.map(\.section), ["A", "B"])
    XCTAssertEqual(record.sections.map(\.held), [true, false], "verdict order survives the wire")
  }

  /// Real-bridge rung tag (#846, #1083): `SetEntryVariant` carries an optional
  /// String across the bincode wire (the absent-vs-present hazard). Drive it
  /// through the live bridge and assert it decodes without error.
  func testRealBridgeTagEntryWithVariantDecodesOnWire() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Scales", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: []))))
    let itemId = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.item(.addVariant(itemId: itemId, label: "F major")))
    let stepId = try XCTUnwrap(try bridge.view().items.first?.variants.first?.id)

    let entry = SetlistEntry(
      id: "e1", itemId: itemId, itemTitle: "Scales", itemType: .exercise,
      position: 0, durationSecs: 0, status: .notAttempted,
      notes: nil, score: nil, intention: nil, repTarget: nil, repCount: nil,
      repTargetReached: nil, repHistory: nil, plannedDurationSecs: nil, achievedTempo: nil,
      groupId: nil, variantId: nil)
    let active = ActiveSession(
      id: "s1", entries: [entry], currentIndex: 0,
      currentItemStartedAt: "2026-06-16T09:00:00Z", sessionStartedAt: "2026-06-16T09:00:00Z",
      sessionIntention: nil)
    _ = try bridge.update(.session(.recoverSession(session: active, now: "2026-06-16T09:00:00Z")))
    let entryId = try XCTUnwrap(try bridge.view().activeSession?.entries.first?.id)

    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: stepId)))
    XCTAssertNil(try bridge.view().error, "tagging a rung must decode on the wire (#846)")
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: nil)))
    XCTAssertNil(try bridge.view().error, "clearing the rung round-trips")
  }

  /// Press-start with a blob already on disk (#1182). `DrillLoopHost.run()`
  /// prefers recovery over starting whenever `pendingCoachSession()` returns
  /// something, so anything planning leaves behind is read as a session to
  /// resume. This drives that exact branch through a real Store and a real
  /// bridge: plan, then take the same decision the host takes, and require a
  /// drill at the end of it. Pressing start and landing back on Practice is the
  /// #846 class, and the two earlier press-start tests could not see it because
  /// neither had a blob in play.
  ///
  /// The drill assertion is indifferent to how the core keeps the promise —
  /// whether planning stops snapshotting, or recovering a planned session starts
  /// it — because press-start has to reach a drill either way. The two blob
  /// assertions are not indifferent: what a blob *means* is settled (#1219), so
  /// they pin it from both sides rather than leaving the branch above to chance.
  func testRealBridgePressStartReachesADrillWithABlobOnDisk() throws {
    let defaults = try XCTUnwrap(UserDefaults(suiteName: "coach-\(UUID().uuidString)"))
    let store = Store(bridge: LiveBridge(), session: mockSession(), sortDefaults: defaults)
    store.send(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))

    store.send(.coach(.planSession(now: SessionClock.nowRFC3339(), availableMinutes: nil)))
    XCTAssertNotNil(store.viewModel?.coach.plan, "the hero needs a plan to press start on")
    XCTAssertNil(
      store.pendingCoachSession(),
      "a blob means a block was cut off mid-flight, so merely planning must not write one (#1219)")

    // Verbatim the branch in DrillLoopHost.run().
    let now = SessionClock.nowRFC3339()
    if let crashed = store.pendingCoachSession() {
      store.send(.coach(.recoverSession(session: crashed, now: now)))
    } else {
      store.send(.coach(.startPlannedSession(now: now)))
    }

    XCTAssertNotNil(
      store.viewModel?.coach.drill,
      "press start must open a block; with no drill the host closes and the tap does nothing")
    XCTAssertNotNil(
      store.pendingCoachSession(),
      "a running block is worth recovering, or an interruption loses its evidence (#1181)")
  }

  /// Real-bridge press-start (#1182): `PlanView` is what the Practice hero
  /// renders, so it has to survive the actual bincode wire before it reaches a
  /// screen — a stub bridge would pass while the hero showed the no-plan
  /// fallback forever (`specs/coach-2a-slice-contract.md` §3).
  ///
  /// The plan's content is authored (`content/`), so the assertions are the
  /// invariants press-start depends on rather than titles: a first block with a
  /// non-empty why line, minutes that add up to something, and the plan giving
  /// way to a drill once the session runs.
  func testRealBridgePlanSessionCrossesTheWireForPressStart() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    XCTAssertNil(try bridge.view().coach.plan, "no plan until one is asked for")

    _ = try bridge.update(
      .coach(.planSession(now: SessionClock.nowRFC3339(), availableMinutes: nil)))
    let plan = try XCTUnwrap(
      try bridge.view().coach.plan, "planning must reach the shell (#846)")
    let first = try XCTUnwrap(plan.blocks.first, "a plan press-start can run has a first block")
    XCTAssertFalse(first.drillTitle.isEmpty, "the hero's headline crossed the wire")
    XCTAssertFalse(first.why.isEmpty, "every block cites why it is here (spec §5)")
    XCTAssertGreaterThan(plan.totalMinutes, 0, "the hero promises a length")

    _ = try bridge.update(.coach(.startPlannedSession(now: SessionClock.nowRFC3339())))
    XCTAssertNotNil(try bridge.view().coach.drill, "running the plan opens its first block")
    XCTAssertNil(try bridge.view().coach.plan, "the plan gives way once the block opens")
  }

  /// Real-bridge drill loop (#846, #1176): drives a gate to completion — start,
  /// count-in, body beats, a clean tap per required pass — through the actual
  /// Swift↔Rust bincode wire, and asserts the core's own counting comes back in
  /// `CoachView`. A stub bridge can't catch a wire break here; the symptom would
  /// be a drill screen that never advances (`specs/intrada-coach-engine.md` §6).
  ///
  /// How many passes the gate wants is authored content (#1180) that Jon
  /// recalibrates, so the target is read from the view and the assertion is the
  /// invariant: the dots fill one per clean pass, and the last one opens the
  /// gate. A literal here would redden on a content tweak that broke nothing.
  func testRealBridgeDrillLoopCountsRepsInTheCore() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    XCTAssertNil(try bridge.view().coach.drill, "no drill until one is started")

    _ = try bridge.update(.coach(.startPlannedSession(now: SessionClock.nowRFC3339())))
    let card = try XCTUnwrap(try bridge.view().coach.drill)
    XCTAssertEqual(card.phase, .blockEntry, "a session opens on block 1's card (#1223)")
    XCTAssertFalse(card.pulseRunning, "the card is silent — nothing sounds until Start")
    XCTAssertFalse(card.why.isEmpty, "the card cites why this block is here")
    XCTAssertGreaterThan(card.minutes, 0, "the card promises a length")

    _ = try bridge.update(.coach(.startBlock(now: SessionClock.nowRFC3339())))
    let opening = try XCTUnwrap(try bridge.view().coach.drill)
    let openingTempo = try XCTUnwrap(opening.tempoBpm, "a clocked rung crosses with its tempo")
    XCTAssertFalse(opening.drillTitle.isEmpty, "the block crossed the wire with its title")
    XCTAssertEqual(
      opening.gateQuestion, "Clean at \(openingTempo)?",
      "the criterion names the tempo it is asked at")
    XCTAssertGreaterThanOrEqual(opening.gateTarget, 1, "a gate needs at least one clean pass")
    XCTAssertEqual(opening.gateFilled, 0)
    XCTAssertEqual(
      opening.phase, .countIn(remaining: 4),
      "Start counts the block in on the during-play page (#1184)")
    XCTAssertTrue(opening.pulseRunning, "Start is what sets the click going")
    XCTAssertFalse(
      opening.clickPattern.isEmpty, "placement crosses as a mask, not an enum (#1224)")
    XCTAssertGreaterThan(opening.phraseBeats, 0, "a pass has a length")

    _ = try bridge.update(.coach(.countInBeat(remaining: 1)))
    XCTAssertEqual(try bridge.view().coach.drill?.phase, .countIn(remaining: 1))

    _ = try bridge.update(.coach(.beat(beatIndex: 0)))
    XCTAssertEqual(try bridge.view().coach.drill?.phase, .playing, "the first pass is under way")

    for pass in 1...opening.gateTarget {
      // Beats count on across the whole pulse now; a pass ends on the boundary.
      let boundary = opening.phraseBeats * UInt32(pass)
      _ = try bridge.update(.coach(.beat(beatIndex: boundary)))
      XCTAssertEqual(
        try bridge.view().coach.drill?.phase, .awaitingVerdict,
        "the phrase ended on pass \(pass), so the core asks the question")

      _ = try bridge.update(.coach(.tap(clean: true, now: SessionClock.nowRFC3339())))
      let after = try XCTUnwrap(try bridge.view().coach.drill)
      XCTAssertEqual(after.gateFilled, pass, "the core fills the dots, not the shell")
      XCTAssertEqual(
        after.pulseSeq, opening.pulseSeq,
        "a tap never restarts the click — the pulse runs the whole block (#1223)")
      XCTAssertTrue(after.pulseRunning)

      if pass == opening.gateTarget {
        XCTAssertEqual(after.phase, .gateOpen, "the last clean pass opens the gate")
      } else {
        XCTAssertEqual(after.phase, .acknowledged(clean: true))
        _ = try bridge.update(.coach(.beat(beatIndex: boundary + 1)))
        let playing = try XCTUnwrap(try bridge.view().coach.drill)
        XCTAssertEqual(playing.phase, .playing, "the next beat clears the glance (T10)")
        XCTAssertEqual(playing.pulseSeq, opening.pulseSeq, "and still leaves the click alone")
      }
    }
    XCTAssertNil(try bridge.view().error, "a whole gate must decode on the wire (#846)")
  }

  /// Two phrase boundaries go by and only the tapped one banks anything, so not
  /// tapping costs the pass rather than freezing the loop. The closing tap is
  /// the control: without it, "still 0" would pass on a core that had never
  /// counted at all.
  func testRealBridgeOnlyATappedPassBanks() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(.coach(.startPlannedSession(now: SessionClock.nowRFC3339())))
    _ = try bridge.update(.coach(.startBlock(now: SessionClock.nowRFC3339())))
    let opening = try XCTUnwrap(try bridge.view().coach.drill)
    let phrase = opening.phraseBeats

    _ = try bridge.update(.coach(.beat(beatIndex: 0)))
    _ = try bridge.update(.coach(.beat(beatIndex: phrase)))
    XCTAssertEqual(try bridge.view().coach.drill?.phase, .awaitingVerdict)

    _ = try bridge.update(.coach(.beat(beatIndex: phrase + 1)))
    XCTAssertEqual(
      try bridge.view().coach.drill?.phase, .awaitingVerdict,
      "an ordinary beat does not close the window — the hands are mid-pass")

    // Leave the first pass unjudged and play through the next boundary.
    _ = try bridge.update(.coach(.beat(beatIndex: phrase * 2)))
    let second = try XCTUnwrap(try bridge.view().coach.drill)
    XCTAssertEqual(second.gateFilled, 0, "the pass nobody judged banked nothing")
    XCTAssertEqual(second.pulseSeq, opening.pulseSeq, "and the click never paused for it")

    _ = try bridge.update(.coach(.tap(clean: true, now: SessionClock.nowRFC3339())))
    XCTAssertEqual(
      try bridge.view().coach.drill?.gateFilled, 1,
      "one tap, one dot — two boundaries went by and only the judged pass counted")
    XCTAssertNil(try bridge.view().error)
  }

  /// A wire break here would look like a button that quietly logs a fail.
  func testRealBridgeDiscardRecordsNothing() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(.coach(.startPlannedSession(now: SessionClock.nowRFC3339())))
    _ = try bridge.update(.coach(.startBlock(now: SessionClock.nowRFC3339())))
    let opening = try XCTUnwrap(try bridge.view().coach.drill)
    _ = try bridge.update(.coach(.beat(beatIndex: 0)))
    _ = try bridge.update(.coach(.beat(beatIndex: opening.phraseBeats)))
    XCTAssertEqual(try bridge.view().coach.drill?.phase, .awaitingVerdict)

    _ = try bridge.update(.coach(.discardAttempt(now: SessionClock.nowRFC3339())))
    let after = try XCTUnwrap(try bridge.view().coach.drill)
    XCTAssertEqual(after.phase, .playing, "a discard closes the window and plays on")
    XCTAssertEqual(after.gateFilled, 0, "a discarded pass banks nothing")
    XCTAssertEqual(after.pulseSeq, opening.pulseSeq, "and never restarts the click")

    // The control for the assertion above: a dot the discard did not fill is
    // only evidence if the very next tap can still fill one.
    _ = try bridge.update(.coach(.beat(beatIndex: opening.phraseBeats * 2)))
    _ = try bridge.update(.coach(.tap(clean: true, now: SessionClock.nowRFC3339())))
    XCTAssertEqual(
      try bridge.view().coach.drill?.gateFilled, 1,
      "the gate was fillable all along — the discard chose not to")
    XCTAssertNil(try bridge.view().error)
  }

  /// `Exit::Skipped` was specced in the engine and unreachable from Swift.
  func testRealBridgeSkipBlockAdvancesToTheNextCard() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(.coach(.startPlannedSession(now: SessionClock.nowRFC3339())))
    let card = try XCTUnwrap(try bridge.view().coach.drill)
    XCTAssertEqual(card.phase, .blockEntry)

    _ = try bridge.update(.coach(.skipBlock(now: SessionClock.nowRFC3339())))
    let next = try XCTUnwrap(
      try bridge.view().coach.drill, "skipping block 1 opens block 2, not nothing")
    XCTAssertEqual(next.blockIndex, card.blockIndex + 1, "the session moved on one block")
    XCTAssertEqual(next.phase, .blockEntry, "and lands on its card, not mid-count-in")
    XCTAssertNil(try bridge.view().error)
  }

  /// Real-bridge evidence read-back (#846, #1214): a wire break would leave the
  /// mastery track silently unrebuilt, so drive the resolve through LiveBridge
  /// and read the outcome off the plan, the only ViewModel surface it reaches.
  func testRealBridgeCoachRecordsRebuildMasteryOnTheWire() throws {
    let bridge = LiveBridge()
    let launch = try bridge.update(
      .startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    let id = try XCTUnwrap(
      launch.first { request in
        if case .persistence(.loadCoachRecords) = request.effect { return true }
        return false
      }?.id, "a local-first launch must ask for the coach records")

    _ = try bridge.resolve(id, persistenceOutput: .coachRecords(Self.overdueEvidence))
    _ = try bridge.update(
      .coach(.planSession(now: "2026-09-20T10:00:00Z", availableMinutes: 30)))

    let plan = try XCTUnwrap(try bridge.view().coach.plan, "a planned session")
    XCTAssertNil(try bridge.view().error, "a clean read leaves nothing to surface")
    // "is due back" needs `overdue_pct >= 100`, unreachable without a replayed
    // `last_attempt_at`: its appearance proves the records crossed the wire.
    XCTAssertTrue(
      plan.blocks.contains { $0.why.contains("is due back") },
      "the plan must say what fell due while the app was closed: "
        + "\(plan.blocks.map(\.why))")
  }

  /// A fortnight of clean passes on the last node of the authored route, far
  /// enough back to be overdue by the time the plan is asked for.
  private static var overdueEvidence: [BlockRecord] {
    (0..<12).map { day in
      BlockRecord(
        id: "b\(day)", node: "phrase-transposition", drill: "phrase-home-key",
        gate: "phrase-home-key",
        level: ParameterLevel(tempoBpm: 120, clickLevel: .twoAndFour),
        circle: .head, mode: .keys,
        startedAt: "2026-08-\(String(format: "%02d", day + 1))T10:00:00Z",
        endedAt: "2026-08-\(String(format: "%02d", day + 1))T10:00:30Z",
        attempts: [
          AttemptSummary(
            at: "2026-08-\(String(format: "%02d", day + 1))T10:00:09Z", verdict: .clean,
            source: .tapVerdict, cold: false, selfPredicted: nil,
            level: ParameterLevel(tempoBpm: 120, clickLevel: .twoAndFour))
        ],
        attemptsToPass: nil, gateOpenedAtAttempt: nil, repsAfterGate: 0, activeMs: 30_000,
        escalationFired: [], exit: .ceilingHit, origin: .authored)
    }
  }

  /// Real-bridge escalation (#846, #1176): "I'm stuck" is the core's decision —
  /// the ladder drops the tempo and the criterion follows it. The old Swift
  /// harness did this arithmetic itself.
  ///
  /// The drop is the configured percentage, read off the core's own
  /// `EngineConfig` rather than written as a literal, so retuning
  /// `escalation.tempo_down_pct` in the content moves the expectation with it.
  func testRealBridgeStuckDropsTheTempoInTheCore() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    let started = try bridge.update(.coach(.startPlannedSession(now: SessionClock.nowRFC3339())))
    let config = try coachSession(in: started).config
    _ = try bridge.update(.coach(.startBlock(now: SessionClock.nowRFC3339())))
    let opening = try XCTUnwrap(try bridge.view().coach.drill)
    _ = try bridge.update(.coach(.beat(beatIndex: 0)))

    _ = try bridge.update(.coach(.stuck(now: SessionClock.nowRFC3339())))
    let after = try XCTUnwrap(try bridge.view().coach.drill)

    let openingTempo = try XCTUnwrap(opening.tempoBpm, "a clocked rung has a tempo to drop")
    let afterTempo = try XCTUnwrap(after.tempoBpm, "and still has one after the drop")
    let opened = UInt32(openingTempo)
    let dropped = opened - opened * UInt32(min(config.tempoDownPct, 100)) / 100
    XCTAssertEqual(
      afterTempo, UInt16(max(dropped, UInt32(config.tempoFloorBpm))),
      "the first rung takes the configured percentage off, floored")
    XCTAssertLessThan(
      afterTempo, openingTempo, "a rung that moves nothing is not an escalation")
    XCTAssertEqual(
      after.gateQuestion, "Clean at \(afterTempo)?",
      "the criterion follows the tempo down")
    XCTAssertEqual(after.phase, .playing, "escalation acts rather than narrates")
    XCTAssertGreaterThan(
      after.pulseSeq, opening.pulseSeq,
      "a rung that moved a click parameter is the one thing that does restart it (#1223)")
  }

  /// Real-bridge chord-chart round-trip (#846): `SetChordChart` carries a String
  /// but returns a nested `ChordChart` + `ScaffoldPreviewView` across the bincode
  /// wire — a shape a stub bridge can't exercise. A bad chart must surface an
  /// error and never store a partial.
  func testRealBridgeSetChordChartDerivesScaffoldPreview() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma", key: "G",
            modality: .minor, tempo: nil, notes: nil, tags: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 | F7 | Bbmaj7 |")))
    let ok = try bridge.view()
    let piece = try XCTUnwrap(ok.items.first { $0.id == id })
    let preview = try XCTUnwrap(
      piece.scaffoldPreview, "charted piece derives a preview (err=\(ok.error ?? "nil"))")
    XCTAssertEqual(preview.key, "G")
    XCTAssertEqual(preview.specs.count, 5, "five generators")
    XCTAssertEqual(piece.chordChart?.sections.first?.bars.count, 3, "three bars round-trip")

    // A bad token surfaces an error and leaves the prior chart intact.
    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 | Hxyz |")))
    let bad = try bridge.view()
    XCTAssertNotNil(bad.error, "a parse error must surface, not vanish (#846)")
    XCTAssertEqual(
      bad.items.first { $0.id == id }?.chordChart?.sections.first?.bars.count, 3,
      "a failed parse never overwrites the stored chart")
  }

  /// Real-bridge commit (#1106): `CommitScaffold` carries a `Vec<ScaffoldKind>`
  /// — round-trip it through the live bincode bridge so the write payload can't
  /// silently misalign (#846), and assert the core materialises + links the
  /// selected exercises.
  func testRealBridgeCommitScaffoldLinksExercises() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Autumn Leaves", kind: .piece, composer: "Joseph Kosma", key: "G",
            modality: .minor, tempo: nil, notes: nil, tags: []))))
    let id = try XCTUnwrap(try bridge.view().items.first?.id)
    _ = try bridge.update(
      .item(.setChordChart(pieceId: id, rawChart: "| Cm7 | F7 | Bbmaj7 |")))

    _ = try bridge.update(
      .item(.commitScaffold(pieceId: id, kinds: [.shells, .guideToneLines])))

    let after = try bridge.view()
    XCTAssertNil(after.error, "commit surfaces no error (err=\(after.error ?? "nil"))")
    let piece = try XCTUnwrap(after.items.first { $0.id == id })
    XCTAssertEqual(piece.linkedExercises.count, 2, "two selected kinds become linked exercises")
    let titles = Swift.Set(piece.linkedExercises.map(\.title))
    XCTAssertTrue(titles.contains("Shells") && titles.contains("Guide-tone lines"))

    // Re-committing the same kinds dedups — no duplicate exercises.
    _ = try bridge.update(.item(.commitScaffold(pieceId: id, kinds: [.shells])))
    let reran = try bridge.view()
    let shells = reran.items.filter { $0.title == "Shells" }
    XCTAssertEqual(shells.count, 1, "re-commit adds no duplicate (#1106 dedup)")
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

  /// Real-bridge play→save lifecycle (#932): drives the actual bincode
  /// bridge through Active → Summary → Idle, mirroring the FocusPlayer →
  /// Summary screens. A wire break surfaces here as a failed transition
  /// instead of the silent no-op the stub bridge would hide (#846).
  func testRealBridgeSessionFlowPlaySave() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Etude", kind: .piece, composer: "Chopin", key: nil, modality: nil,
            tempo: nil, notes: nil, tags: []))))
    let itemId = try XCTUnwrap(try bridge.view().items.first?.id)

    let entry = SetlistEntry(
      id: "e1", itemId: itemId, itemTitle: "Etude", itemType: .piece,
      position: 0, durationSecs: 0, status: .notAttempted,
      notes: nil, score: nil, intention: nil, repTarget: nil, repCount: nil,
      repTargetReached: nil, repHistory: nil, plannedDurationSecs: nil, achievedTempo: nil,
      groupId: nil, variantId: nil)
    let building = ActiveSession(
      id: "s1", entries: [entry], currentIndex: 0,
      currentItemStartedAt: "2026-06-16T10:00:00Z", sessionStartedAt: "2026-06-16T10:00:00Z",
      sessionIntention: nil)
    _ = try bridge.update(.session(.recoverSession(session: building, now: "2026-06-16T10:00:00Z")))
    let active = try bridge.view()
    XCTAssertNotNil(active.activeSession, "recoverSession should enter the player")
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
    // Achieved tempo — the hand-off sheet's TempoStepper write, never previously
    // sent from Swift (#846). Round-trip set + clear through the live bridge.
    _ = try bridge.update(.session(.updateEntryTempo(entryId: entryId, tempo: 96)))
    XCTAssertEqual(
      try bridge.view().summary?.entries.first?.achievedTempo, 96,
      "the tempo stepper's achieved tempo should round-trip")
    _ = try bridge.update(.session(.updateEntryTempo(entryId: entryId, tempo: nil)))
    XCTAssertNil(
      try bridge.view().summary?.entries.first?.achievedTempo,
      "clearing an achieved tempo round-trips")
    _ = try bridge.update(.session(.updateSessionNotes(notes: "Felt good")))
    XCTAssertEqual(try bridge.view().summary?.notes, "Felt good", "notes should round-trip")
    _ = try bridge.update(.session(.updateSessionNotes(notes: nil)))
    XCTAssertNil(try bridge.view().summary?.notes, "clearing notes round-trips")
    // Structured reflection — a brand-new wire surface (ReflectionField's
    // variant indices) never sent from Swift before (#846). Set + clear each.
    _ = try bridge.update(
      .session(.updateSessionReflection(field: .improved, text: "thumb-unders even")))
    XCTAssertEqual(
      try bridge.view().summary?.reflectionImproved, "thumb-unders even",
      "reflection 'improved' should round-trip the live bridge")
    _ = try bridge.update(
      .session(.updateSessionReflection(field: .stillRough, text: "bridge rushes")))
    XCTAssertEqual(try bridge.view().summary?.reflectionStillRough, "bridge rushes")
    _ = try bridge.update(
      .session(.updateSessionReflection(field: .nextTarget, text: "bridge at 80")))
    XCTAssertEqual(try bridge.view().summary?.reflectionNextTarget, "bridge at 80")
    _ = try bridge.update(.session(.updateSessionReflection(field: .improved, text: nil)))
    XCTAssertNil(
      try bridge.view().summary?.reflectionImproved, "clearing a reflection round-trips")

    _ = try bridge.update(.session(.saveSession(now: "2026-06-16T10:20:30Z")))
    let saved = try bridge.view()
    XCTAssertNil(saved.summary, "saveSession clears the summary (session persisted)")
    XCTAssertNil(saved.activeSession)
    XCTAssertNil(saved.error, "a clean save surfaces no error")

    // Crash recovery — RecoverSession (with its new `now` re-anchor field) has
    // never crossed the live bridge from Swift before (#846, #962). The stale
    // anchor must come back re-anchored to the `now` we send.
    let blobEntry = SetlistEntry(
      id: "re1", itemId: "i1", itemTitle: "Recovered Scales", itemType: .exercise,
      position: 0, durationSecs: 0, status: .notAttempted,
      notes: nil, score: nil, intention: nil, repTarget: nil, repCount: nil,
      repTargetReached: nil, repHistory: nil, plannedDurationSecs: nil, achievedTempo: nil,
      groupId: nil, variantId: nil)
    let blob = ActiveSession(
      id: "recovered", entries: [blobEntry], currentIndex: 0,
      currentItemStartedAt: "2026-06-16T08:00:00Z", sessionStartedAt: "2026-06-16T08:00:00Z",
      sessionIntention: nil)
    _ = try bridge.update(.session(.recoverSession(session: blob, now: "2026-06-16T11:00:00Z")))
    let recovered = try bridge.view()
    XCTAssertEqual(recovered.activeSession?.currentItemTitle, "Recovered Scales")
    XCTAssertEqual(
      recovered.activeSession?.currentItemStartedAt, "2026-06-16T11:00:00+00:00",
      "recovery must re-anchor the running item's timer to `now`")
    _ = try bridge.update(.session(.abandonSession))
    XCTAssertNil(try bridge.view().activeSession, "abandon after recover returns to Idle")
  }

  /// Real-bridge step-ladder lifecycle (#1083 C1): SetVariants and
  /// UpdateEntryVariant have never crossed the live bincode bridge from Swift
  /// before; a wire break here is the silent no-op class (#846). Drives
  /// ladder create → reorder-preserves-ids → per-entry attribution set/clear.
  func testRealBridgeStepLadderAndEntryAttribution() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))
    _ = try bridge.update(
      .item(
        .add(
          CreateItem(
            title: "Shells", kind: .exercise, composer: nil, key: nil, modality: nil,
            tempo: nil, notes: nil, tags: []))))
    let exId = try XCTUnwrap(try bridge.view().items.first?.id)

    _ = try bridge.update(.item(.setVariants(id: exId, labels: ["C", "F", "B♭"])))
    let afterSet = try bridge.view()
    let ladder = try XCTUnwrap(afterSet.items.first?.variants)
    XCTAssertEqual(
      ladder.map(\.label), ["C", "F", "B♭"],
      "the ladder should land (err=\(afterSet.error ?? "nil"))")
    XCTAssertTrue(ladder.allSatisfy { !$0.isSolid }, "an unrated ladder has no solid steps")
    XCTAssertEqual(ladder.first?.isCurrent, true, "an unrated ladder starts at step one")
    let stepId = try XCTUnwrap(ladder.first?.id)

    // Reorder must keep ids (and so score history); reconcile-by-label.
    _ = try bridge.update(.item(.setVariants(id: exId, labels: ["F", "C", "B♭"])))
    let reordered = try XCTUnwrap(try bridge.view().items.first?.variants)
    XCTAssertEqual(
      reordered.first { $0.label == "C" }?.id, stepId, "reordering keeps each step's id")

    // Practise it to the summary, then attribute the entry to a step + clear.
    let entry = SetlistEntry(
      id: "e1", itemId: exId, itemTitle: "Shells", itemType: .exercise,
      position: 0, durationSecs: 0, status: .notAttempted,
      notes: nil, score: nil, intention: nil, repTarget: nil, repCount: nil,
      repTargetReached: nil, repHistory: nil, plannedDurationSecs: nil, achievedTempo: nil,
      groupId: nil, variantId: nil)
    let active = ActiveSession(
      id: "s1", entries: [entry], currentIndex: 0,
      currentItemStartedAt: "2026-07-17T10:00:00Z", sessionStartedAt: "2026-07-17T10:00:00Z",
      sessionIntention: nil)
    _ = try bridge.update(.session(.recoverSession(session: active, now: "2026-07-17T10:00:00Z")))
    _ = try bridge.update(.session(.nextItem(now: "2026-07-17T10:10:00Z")))
    let summary = try bridge.view()
    let entryId = try XCTUnwrap(summary.summary?.entries.first?.id)

    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: stepId)))
    let attributed = try bridge.view()
    XCTAssertEqual(
      attributed.summary?.entries.first?.variantId, stepId,
      "the step attribution should round-trip (err=\(attributed.error ?? "nil"))")
    _ = try bridge.update(.session(.setEntryVariant(entryId: entryId, variantId: nil)))
    XCTAssertNil(
      try bridge.view().summary?.entries.first?.variantId,
      "clearing the attribution round-trips")
  }

  /// A stub bridge can't catch a regression here (#1272): the message is
  /// assembled in the core, so only the real wire carries a 4xx body in and the
  /// finished string back out.
  func testRealBridgeSurfacesTheMessageTheServerSentWithARejection() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))

    let requests = try bridge.update(.account(.loadPreferences))
    let http = try XCTUnwrap(
      requests.first { if case .http = $0.effect { return true } else { return false } },
      "preferences are server-backed, so loading them emits an Http effect")

    let body = Array(#"{"error":"Your session has expired. Sign in again."}"#.utf8)
    _ = try bridge.resolve(
      http.id, httpResult: .ok(HttpResponse(status: 401, headers: [], body: body)))

    XCTAssertEqual(
      try bridge.view().error,
      "Failed to load preferences: Your session has expired. Sign in again.",
      "the API's sentence must reach the banner, not the bare status")
  }

  /// Guards the banner going blank rather than showing the status.
  func testRealBridgeFallsBackToTheStatusWhenARejectionIsNotOurEnvelope() throws {
    let bridge = LiveBridge()
    _ = try bridge.update(.startApp(apiBaseUrl: "http://localhost:3001", localFirst: true))

    let requests = try bridge.update(.account(.loadPreferences))
    let http = try XCTUnwrap(
      requests.first { if case .http = $0.effect { return true } else { return false } })

    _ = try bridge.resolve(
      http.id,
      httpResult: .ok(
        HttpResponse(status: 502, headers: [], body: Array("<html>Bad Gateway</html>".utf8))))

    let error = try XCTUnwrap(try bridge.view().error, "a rejection must surface something")
    XCTAssertTrue(error.contains("502"), "got \(error)")
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
    updatedAt: "2026-01-01T00:00:00Z", priority: false, chordChart: nil, variants: [])

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
  func save(_ items: [Item]) throws { throw TestError() }
  func delete(id: String, deletedAt: String) throws { throw TestError() }
  func loadSessions() throws -> [PracticeSession] { throw TestError() }
  func saveSession(_ session: PracticeSession) throws { throw TestError() }
  func saveCoachRecords(
    blocks: [BlockRecord], wanders: [WanderRecord], playThroughs: [PlayThroughRecord],
    updatedAt: String
  ) throws {
    throw TestError()
  }
  func loadCoachRecords() throws -> [BlockRecord] { throw TestError() }
  func saveUserDrill(_ drill: UserDrill) throws { throw TestError() }
  func saveJournalItem(_ journal: JournalItem) throws { throw TestError() }
  func saveBuiltSession(_ session: BuiltSession) throws { throw TestError() }
  func saveReflection(_ reflection: Reflection) throws { throw TestError() }
  func saveFeelEntry(_ entry: FeelEntry) throws { throw TestError() }
  func loadBuiltSessionData() throws -> BuiltSessionData { throw TestError() }
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
