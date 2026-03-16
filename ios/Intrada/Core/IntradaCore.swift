// IntradaCore.swift
//
// Swift shell effect processor for Intrada.
// Port of crates/intrada-web/src/core_bridge.rs.
//
// Wraps CoreFfi (Rust FFI via BCS/bincode) and dispatches effects:
//   - Render → read ViewModel from core
//   - Http   → execute HTTP request via URLSession, resolve with HttpResult
//   - App    → session crash recovery (save/clear in-progress session)
//
// The CoreFfi bridge uses the Request/Resolve pattern:
//   1. update(eventBytes) → [Request] (each with id + effect)
//   2. Shell processes each effect
//   3. Shell calls resolve(id, responseBytes) → more [Request]
//
// HTTP-through-core: The Rust core owns all HTTP logic (URLs, bodies, parsing).
// This shell is a dumb I/O pipe — it executes the HttpRequest the core provides
// and sends back the raw HttpResponse. Auth headers and 401 retry are handled
// here since they require platform-specific Clerk integration.

import ClerkKit
import Foundation
import Observation

/// The main effect processor for the Intrada iOS shell.
///
/// Holds the Rust `CoreFfi` instance and publishes the `ViewModel` for SwiftUI.
/// All state changes flow through: UI -> `update(Event)` -> Core -> effects -> shell I/O -> resolve -> re-render.
@Observable
@MainActor
final class IntradaCore {

    // MARK: - Published State

    /// The current view model, updated after every Render effect.
    private(set) var viewModel: ViewModel

    /// Whether the initial data load is in progress.
    /// True until the first Render effect after startApp(), then stays false
    /// so background refreshes don't flash the skeleton.
    private(set) var isLoading = true

    /// Guard to prevent duplicate startApp() calls (e.g. from onAppear firing multiple times).
    private var hasStarted = false

    /// Non-nil if the initial ViewModel deserialization failed.
    private(set) var initializationError: String?

    // MARK: - Private State

    private let core: CoreFfi
    private let session: URLSession

    /// The Clerk instance for auth token retrieval.
    /// Set via `setClerk(_:)` before calling `startApp()` to avoid using `Clerk.shared`.
    private var clerk: Clerk?

    // MARK: - Init

    init() {
        let core = CoreFfi()
        self.core = core
        self.session = URLSession.shared

        // Read initial ViewModel from the core (empty lists, default values)
        let data = core.view()
        do {
            self.viewModel = try ViewModel.bincodeDeserialize(input: [UInt8](data))
        } catch {
            let message = "Failed to deserialize initial ViewModel: \(error)"
            print("[IntradaCore] \(message)")
            self.initializationError = message
            // Provide a default empty ViewModel so the app can show an error state
            self.viewModel = ViewModel(
                items: [],
                sessions: [],
                activeSession: nil,
                buildingSetlist: nil,
                summary: nil,
                sessionStatus: .idle,
                error: message,
                analytics: nil,
                routines: []
            )
        }
    }

    // MARK: - Public API

    /// Send an event to the Rust core and process resulting effects.
    ///
    /// This is the primary entry point for all UI interactions.
    func update(_ event: Event) {
        let eventBytes: [UInt8]
        do {
            eventBytes = try event.bincodeSerialize()
        } catch {
            print("[IntradaCore] Failed to serialize event: \(error)")
            return
        }

        let responseData = core.update(Data(eventBytes))
        processRequests(responseData)
    }

    /// Provide the Clerk instance for auth token retrieval.
    ///
    /// Must be called before `startApp()`. This avoids using `Clerk.shared`
    /// which crashes if `Clerk.configure()` failed.
    func setClerk(_ clerk: Clerk) {
        self.clerk = clerk
    }

    /// Initialise the core with the API base URL and start loading data.
    ///
    /// Called once after sign-in. Sends `StartApp` to the core which triggers
    /// HTTP effects for fetching items, sessions, and routines.
    /// Guarded to prevent duplicate calls from `onAppear` re-firing.
    func startApp() {
        guard !hasStarted else { return }
        hasStarted = true
        isLoading = true
        update(.startApp(apiBaseUrl: Config.apiBaseURL))

        // Restore any in-progress session from crash recovery
        if let savedSession = SessionStorage.load() {
            update(.session(.recoverSession(session: savedSession)))
        }
        // isLoading is cleared by the first Render effect (see refreshViewModel)
    }

    // MARK: - Request Processing (BCS Bridge)

    /// Deserialise a BCS response from the core into [Request] and process each one.
    private func processRequests(_ data: Data) {
        let bytes = [UInt8](data)
        guard !bytes.isEmpty else { return }

        let requests: [Request]
        do {
            requests = try [Request].bincodeDeserialize(input: bytes)
        } catch {
            print("[IntradaCore] Failed to deserialize requests: \(error)")
            return
        }

        for request in requests {
            switch request.effect {
            case .render:
                refreshViewModel()
            case .http(let httpRequest):
                handleHttpEffect(httpRequest, requestId: request.id)
            case .app(let appEffect):
                handleAppEffect(appEffect, requestId: request.id)
            }
        }
    }

    /// Read the current ViewModel from the core via BCS.
    ///
    /// The first render after `startApp()` clears `isLoading` so the skeleton
    /// is replaced by real content. Subsequent renders (background refreshes)
    /// never re-set `isLoading`, preventing skeleton flashes.
    private func refreshViewModel() {
        let data = core.view()
        do {
            viewModel = try ViewModel.bincodeDeserialize(input: [UInt8](data))
            if isLoading {
                isLoading = false
            }
        } catch {
            print("[IntradaCore] Failed to deserialize ViewModel: \(error)")
        }
    }

    /// Tell the core an effect has been resolved, then process any new requests.
    private func resolveEffect(_ requestId: UInt32, data: Data = Data()) {
        let moreData = core.resolve(requestId, data)
        processRequests(moreData)
    }

    // MARK: - HTTP Effect Handler

    /// Execute an HTTP request from the core, adding auth and 401 retry.
    ///
    /// The core provides the full URL, method, headers, and body.
    /// This shell adds the Clerk auth header and retries once on 401.
    /// The raw response (status + headers + body) is serialised as `HttpResult`
    /// and resolved back to the core.
    private func handleHttpEffect(_ httpRequest: HttpRequest, requestId: UInt32) {
        Task {
            let result = await executeHttpWithRetry(httpRequest)
            do {
                let resultBytes = try result.bincodeSerialize()
                resolveEffect(requestId, data: Data(resultBytes))
            } catch {
                print("[IntradaCore] Failed to serialize HttpResult: \(error)")
            }
        }
    }

    /// Execute an HTTP request with automatic 401 retry.
    ///
    /// On 401, fetches a fresh Clerk JWT and retries once.
    private func executeHttpWithRetry(_ httpRequest: HttpRequest) async -> HttpResult {
        let firstResult = await executeHttp(httpRequest, freshToken: false)

        // If we got a 401, retry once with a fresh token
        if case .ok(let response) = firstResult, response.status == 401 {
            return await executeHttp(httpRequest, freshToken: true)
        }

        return firstResult
    }

    /// Execute a single HTTP request via URLSession.
    private func executeHttp(_ httpRequest: HttpRequest, freshToken: Bool) async -> HttpResult {
        guard let url = URL(string: httpRequest.url) else {
            return .err(.url("Invalid URL: \(httpRequest.url)"))
        }

        var request = URLRequest(url: url)
        request.httpMethod = httpRequest.method

        // Copy headers from the core's request
        for header in httpRequest.headers {
            request.setValue(header.value, forHTTPHeaderField: header.name)
        }

        // Add auth header from Clerk
        if let authHeader = await getAuthHeader(forceRefresh: freshToken) {
            request.setValue(authHeader, forHTTPHeaderField: "Authorization")
        }

        // Set body if non-empty
        if !httpRequest.body.isEmpty {
            request.httpBody = Data(httpRequest.body)
        }

        do {
            let (data, response) = try await session.data(for: request)

            guard let httpResponse = response as? HTTPURLResponse else {
                return .err(.io("Invalid response type"))
            }

            // Convert response headers to HttpHeader array
            let responseHeaders: [HttpHeader] = httpResponse.allHeaderFields.compactMap { key, value in
                guard let name = key as? String, let val = value as? String else { return nil }
                return HttpHeader(name: name, value: val)
            }

            return .ok(HttpResponse(
                status: UInt16(httpResponse.statusCode),
                headers: responseHeaders,
                body: [UInt8](data)
            ))
        } catch let error as URLError where error.code == .timedOut {
            return .err(.timeout)
        } catch {
            return .err(.io(error.localizedDescription))
        }
    }

    /// Get a Bearer token from Clerk for API authorization.
    ///
    /// Uses the injected `clerk` instance (set via `setClerk(_:)`) instead of
    /// `Clerk.shared`, which can crash if `Clerk.configure()` failed.
    @MainActor
    private func getAuthHeader(forceRefresh: Bool) async -> String? {
        guard let clerkSession = clerk?.session else {
            print("[IntradaCore] Warning: No Clerk session available (clerk instance: \(clerk == nil ? "nil" : "set, no session"))")
            return nil
        }

        do {
            let token = try await clerkSession.getToken(.init(skipCache: forceRefresh))
            return token.map { "Bearer \($0)" }
        } catch {
            print("[IntradaCore] Warning: Failed to get auth token: \(error)")
            return nil
        }
    }

    // MARK: - AppEffect Dispatch (Session Crash Recovery Only)

    private func handleAppEffect(_ effect: AppEffect, requestId: UInt32) {
        switch effect {
        case .saveSessionInProgress(let session):
            SessionStorage.save(session)
        case .clearSessionInProgress:
            SessionStorage.clear()
        }

        // Resolve the effect to notify the core it's complete.
        resolveEffect(requestId)
    }
}
