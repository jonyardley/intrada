import Foundation
import SharedTypes

/// The app's single source of UI truth. Holds the `ViewModel`, sends `Event`s
/// to the core, and runs the effect loop (Render → view; Http → URLSession;
/// App → ack). Owns zero domain logic — it's a pump between SwiftUI and the core.
@MainActor
@Observable
final class Store {
  private(set) var viewModel: ViewModel?

  private let bridge: CoreBridge
  private let session: URLSession
  private let store: LibraryStore?

  init(
    bridge: CoreBridge = LiveBridge(), session: URLSession = .shared, store: LibraryStore? = nil
  ) {
    self.bridge = bridge
    self.session = session
    // Default to in-memory so tests/previews never touch disk; the real app
    // passes an on-disk store. `try?` (not `guarded`) because `self` isn't fully
    // initialized yet here; in-memory creation effectively never fails.
    self.store = store ?? (try? LibraryStore.inMemory())
    // Initial render comes straight from the core; nil only if the bridge
    // itself fails, in which case the view shows a loading state.
    self.viewModel = guarded { try bridge.view() }
  }

  func send(_ event: Event) {
    process(guarded { try bridge.update(event) } ?? [])
  }

  private func process(_ requests: [Request]) {
    for request in requests {
      switch request.effect {
      case .render:
        refreshView()
      case .http(let httpRequest):
        Task { await self.handleHttp(httpRequest, id: request.id) }
      case .app:
        // Foundation: localStorage effects are no-ops for now — ack so
        // the core can continue its command chain.
        process(guarded { try bridge.resolveEmpty(request.id) } ?? [])
      case .persistence(let operation):
        let output = persistenceOutput(for: operation)
        process(guarded { try bridge.resolve(request.id, persistenceOutput: output) } ?? [])
      }
    }
  }

  private func refreshView() {
    if let next = guarded({ try bridge.view() }) {
      viewModel = next
    }
  }

  private func handleHttp(_ request: HttpRequest, id: UInt32) async {
    let result = await Self.execute(request, session: session)
    process(guarded { try bridge.resolve(id, httpResult: result) } ?? [])
  }

  /// Run a persistence op against the local store. A store failure is reported
  /// and answered with the empty/ack shape so the core's command chain still
  /// completes (offline-first assumes local writes succeed; failures are rare
  /// disk/corruption cases worth surfacing, not silently dropping the effect).
  private func persistenceOutput(for operation: PersistenceOperation) -> PersistenceOutput {
    guard let store else { return Self.emptyOutput(for: operation) }
    do {
      switch operation {
      case .loadItems: return .items(try store.loadItems())
      case .saveItem(let item):
        try store.save(item)
        return .ack
      case .deleteItem(let id):
        try store.delete(id: id)
        return .ack
      }
    } catch {
      report(error)
      return Self.emptyOutput(for: operation)
    }
  }

  private static func emptyOutput(for operation: PersistenceOperation) -> PersistenceOutput {
    switch operation {
    case .loadItems: .items([])
    case .saveItem, .deleteItem: .ack
    }
  }

  // A bridge failure means a serialization/protocol break (e.g. stale bindings
  // vs a regenerated core) — unrecoverable at runtime, so report it rather than
  // swallow it silently, and fail soft.
  private func guarded<T>(_ work: () throws -> T) -> T? {
    do { return try work() } catch {
      report(error)
      return nil
    }
  }

  /// Execute a core-built HTTP request via URLSession and map the raw response
  /// back into the core's `HttpResult`. No auth yet (foundation scope).
  private static func execute(_ request: HttpRequest, session: URLSession) async -> HttpResult {
    guard let url = URL(string: request.url) else {
      return .err(.url("Invalid URL: \(request.url)"))
    }

    var urlRequest = URLRequest(url: url)
    urlRequest.httpMethod = request.method
    for header in request.headers {
      urlRequest.setValue(header.value, forHTTPHeaderField: header.name)
    }
    if !request.body.isEmpty {
      urlRequest.httpBody = Data(request.body)
    }

    do {
      let (data, response) = try await session.data(for: urlRequest)
      guard let http = response as? HTTPURLResponse else {
        return .err(.io("Invalid response type"))
      }
      let headers: [HttpHeader] = http.allHeaderFields.compactMap { key, value in
        guard let name = key as? String, let val = value as? String else { return nil }
        return HttpHeader(name: name, value: val)
      }
      return .ok(
        HttpResponse(
          status: UInt16(http.statusCode),
          headers: headers,
          body: [UInt8](data)
        ))
    } catch let error as URLError where error.code == .timedOut {
      return .err(.timeout)
    } catch {
      return .err(.io(error.localizedDescription))
    }
  }
}
