import IntradaCoreFFI
import SharedTypes
import XCTest

@testable import Intrada

/// Drives `Store`'s effect loop with a scripted bridge and a mocked URLSession,
/// so the Render → view / Http → URLSession → resolve / App → ack wiring and the
/// URLSession → `HttpResult` mapping are covered without the real core or network.
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

  func testAppEffectResolvesEmpty() {
    let bridge = FakeBridge()
    bridge.updateHandler = { _ in [Request(id: 7, effect: .app(.clearSessionInProgress))] }
    let store = Store(bridge: bridge, session: mockSession())

    store.send(.setQuery(nil))

    XCTAssertEqual(bridge.emptyResolved, [7], "app effect should ack via resolveEmpty")
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

    XCTAssertEqual(bridge.emptyResolved, [1], "every request in the batch should run")
    XCTAssertEqual(store.viewModel?.error, "batched")
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
    // A bare space can't form a URL, so `URL(string:)` returns nil.
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

  /// Run `action`, then await the bridge's first `resolve` — the HTTP leg hops
  /// through a detached `Task`, so the assertion must wait for it.
  private func whenResolved(_ bridge: FakeBridge, _ action: () -> Void) async {
    let resolved = expectation(description: "bridge resolved")
    bridge.onResolve = { resolved.fulfill() }
    action()
    await fulfillment(of: [resolved], timeout: 2)
  }

  static let sampleItem = Item(
    id: "p1", title: "Etude", kind: .piece, composer: "Chopin", key: nil, modality: nil,
    tempo: nil,
    notes: nil, tags: [], createdAt: "2026-01-01T00:00:00Z", updatedAt: "2026-01-01T00:00:00Z",
    priority: false)

  private func mockSession() -> URLSession {
    let config = URLSessionConfiguration.ephemeral
    config.protocolClasses = [MockURLProtocol.self]
    return URLSession(configuration: config)
  }
}

private func emptyViewModel() throws -> ViewModel {
  try ViewModel.bincodeDeserialize(input: [UInt8](CoreFfi().view()))
}

/// A scriptable `CoreBridge`: returns canned `[Request]` for update/resolve and
/// records what the `Store` fed back, so tests assert on the effect loop.
private final class FakeBridge: CoreBridge {
  var updateHandler: (Event) -> [Request] = { _ in [] }
  var resolveHandler: (UInt32, HttpResult) -> [Request] = { _, _ in [] }
  /// Overrides the next `view()` result, letting a test prove a render refreshed.
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

private struct TestError: Error {}

/// An `ItemStore` that always throws — drives the failure path (#816).
private struct FailingStore: ItemStore {
  func loadItems() throws -> [Item] { throw TestError() }
  func save(_ item: Item) throws { throw TestError() }
  func delete(id: String) throws { throw TestError() }
}

/// Intercepts URLSession traffic so `Store.execute` runs against canned
/// responses/errors. `handler` is set per test; `lastRequest` captures the
/// mapped `URLRequest` for request-shape assertions.
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
