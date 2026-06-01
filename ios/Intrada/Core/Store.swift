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

  init(bridge: CoreBridge = LiveBridge(), session: URLSession = .shared) {
    self.bridge = bridge
    self.session = session
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
        // B1 stub — no GRDB yet (B2); answer with the empty/ack shape the core
        // expects so the contract + typed-Output round-trip are exercised.
        let output: PersistenceOutput =
          switch operation {
          case .loadItems: .items([])
          case .saveItem, .deleteItem: .ack
          }
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
