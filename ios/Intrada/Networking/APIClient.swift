// APIClient.swift
//
// HTTP client for the Intrada REST API.
// Port of crates/intrada-web/src/api_client.rs (432 lines).
//
// All requests include `Authorization: Bearer {jwt}` from Clerk.
// Built-in 401 retry: get fresh token, retry once.

import ClerkKit
import Foundation

/// Actor-isolated HTTP client for the Intrada REST API.
///
/// Mirrors the web shell's `api_client.rs` with 14 endpoints for items,
/// sessions, routines, and goals. All requests are authenticated via Clerk JWT.
actor APIClient {

    private let baseURL: String
    private let session: URLSession

    init(baseURL: String = Config.apiBaseURL) {
        self.baseURL = baseURL
        self.session = URLSession.shared
    }

    // MARK: - Items

    func getItems() async throws -> [Item] {
        try await getJSON("/api/items")
    }

    func createItem(_ item: Item) async throws -> Item {
        try await postJSON("/api/items", body: item)
    }

    func updateItem(_ item: Item) async throws -> Item {
        try await putJSON("/api/items/\(item.id)", body: item)
    }

    func deleteItem(id: String) async throws {
        try await delete("/api/items/\(id)")
    }

    // MARK: - Sessions

    func getSessions() async throws -> [PracticeSession] {
        try await getJSON("/api/sessions")
    }

    func createSession(_ session: PracticeSession) async throws -> PracticeSession {
        try await postJSON("/api/sessions", body: session)
    }

    func deleteSession(id: String) async throws {
        try await delete("/api/sessions/\(id)")
    }

    // MARK: - Routines

    func getRoutines() async throws -> [Routine] {
        try await getJSON("/api/routines")
    }

    func createRoutine(_ routine: Routine) async throws -> Routine {
        let request = CreateRoutineAPIRequest(routine: routine)
        return try await postJSON("/api/routines", body: request)
    }

    func updateRoutine(_ routine: Routine) async throws -> Routine {
        let request = UpdateRoutineAPIRequest(routine: routine)
        return try await putJSON("/api/routines/\(routine.id)", body: request)
    }

    func deleteRoutine(id: String) async throws {
        try await delete("/api/routines/\(id)")
    }

    // MARK: - Goals

    func getGoals() async throws -> [Goal] {
        try await getJSON("/api/goals")
    }

    func createGoal(_ goal: Goal) async throws -> Goal {
        try await postJSON("/api/goals", body: goal)
    }

    func updateGoal(_ goal: Goal) async throws -> Goal {
        let request = UpdateGoalAPIRequest(goal: goal)
        return try await putJSON("/api/goals/\(goal.id)", body: request)
    }

    func deleteGoal(id: String) async throws {
        try await delete("/api/goals/\(id)")
    }

    // MARK: - Generic Helpers with 401 Retry

    private func getJSON<T: Decodable>(_ path: String) async throws -> T {
        try await requestWithRetry(path: path, method: "GET", body: nil as Empty?)
    }

    private func postJSON<B: Encodable, T: Decodable>(_ path: String, body: B) async throws -> T {
        try await requestWithRetry(path: path, method: "POST", body: body)
    }

    private func putJSON<B: Encodable, T: Decodable>(_ path: String, body: B) async throws -> T {
        try await requestWithRetry(path: path, method: "PUT", body: body)
    }

    private func delete(_ path: String) async throws {
        let _: Empty = try await requestWithRetry(path: path, method: "DELETE", body: nil as Empty?)
    }

    /// Execute an HTTP request with automatic 401 retry.
    ///
    /// On 401, fetches a fresh Clerk JWT and retries once.
    private func requestWithRetry<B: Encodable, T: Decodable>(
        path: String,
        method: String,
        body: B?
    ) async throws -> T {
        do {
            return try await executeRequest(path: path, method: method, body: body, freshToken: false)
        } catch APIError.unauthorized {
            // Retry once with a fresh token
            return try await executeRequest(path: path, method: method, body: body, freshToken: true)
        }
    }

    private func executeRequest<B: Encodable, T: Decodable>(
        path: String,
        method: String,
        body: B?,
        freshToken: Bool
    ) async throws -> T {
        guard let url = URL(string: baseURL + path) else {
            throw APIError.invalidURL(path)
        }

        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        // Get auth token from Clerk
        do {
            if let token = try await getAuthToken(forceRefresh: freshToken) {
                request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
            } else {
                print("[APIClient] ⚠️ No auth token for \(method) \(path)")
            }
        } catch {
            print("[APIClient] ⚠️ Failed to get auth token: \(error)")
        }

        if let body {
            request.httpBody = try JSONEncoder.intrada.encode(body)
        }

        let (data, response) = try await session.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }

        switch httpResponse.statusCode {
        case 200...299:
            if T.self == Empty.self {
                // swiftlint:disable:next force_cast
                return Empty() as! T
            }
            return try JSONDecoder.intrada.decode(T.self, from: data)
        case 401:
            throw APIError.unauthorized
        default:
            let body = String(data: data, encoding: .utf8) ?? "<no body>"
            throw APIError.httpError(statusCode: httpResponse.statusCode, body: body)
        }
    }

    /// Get a JWT from Clerk for API authorization.
    @MainActor
    private func getAuthToken(forceRefresh: Bool = false) async throws -> String? {
        guard let session = Clerk.shared.session else {
            print("[APIClient] ⚠️ No Clerk session available — requests will be unauthenticated")
            return nil
        }
        let token = try await session.getToken(.init(skipCache: forceRefresh))
        if token == nil {
            print("[APIClient] ⚠️ Clerk session exists but getToken returned nil")
        }
        return token
    }
}

// MARK: - API Request Types

/// Matches `CreateRoutineApiRequest` in the Rust web shell.
private struct CreateRoutineAPIRequest: Encodable {
    let name: String
    let entries: [CreateRoutineEntryRequest]

    init(routine: Routine) {
        self.name = routine.name
        self.entries = routine.entries.map { entry in
            CreateRoutineEntryRequest(
                itemId: entry.itemId,
                itemTitle: entry.itemTitle,
                itemType: entry.itemType,
                position: entry.position
            )
        }
    }
}

private struct CreateRoutineEntryRequest: Encodable {
    let itemId: String
    let itemTitle: String
    let itemType: String
    let position: UInt

    enum CodingKeys: String, CodingKey {
        case itemId = "item_id"
        case itemTitle = "item_title"
        case itemType = "item_type"
        case position
    }
}

/// Matches `UpdateRoutineApiRequest` in the Rust web shell.
private struct UpdateRoutineAPIRequest: Encodable {
    let name: String
    let entries: [CreateRoutineEntryRequest]

    init(routine: Routine) {
        self.name = routine.name
        self.entries = routine.entries.map { entry in
            CreateRoutineEntryRequest(
                itemId: entry.itemId,
                itemTitle: entry.itemTitle,
                itemType: entry.itemType,
                position: entry.position
            )
        }
    }
}

/// Matches `UpdateGoalApiRequest` in the Rust web shell.
private struct UpdateGoalAPIRequest: Encodable {
    let title: String
    let kind: GoalKind
    let status: String
    let deadline: Date?

    enum CodingKeys: String, CodingKey {
        case title, kind, status, deadline
    }

    init(goal: Goal) {
        self.title = goal.title
        self.kind = goal.kind
        self.status = goal.status.rawValue
        self.deadline = goal.deadline
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(title, forKey: .title)
        try container.encode(kind, forKey: .kind)
        try container.encode(status, forKey: .status)
        try container.encodeIfPresent(deadline, forKey: .deadline)
    }
}

// MARK: - Error Types

enum APIError: Error, LocalizedError {
    case invalidURL(String)
    case invalidResponse
    case unauthorized
    case httpError(statusCode: Int, body: String)

    var errorDescription: String? {
        switch self {
        case .invalidURL(let path):
            return "Invalid URL: \(path)"
        case .invalidResponse:
            return "Invalid server response"
        case .unauthorized:
            return "Authentication failed"
        case .httpError(let code, let body):
            return "HTTP \(code): \(body)"
        }
    }
}

/// Empty type for DELETE responses and nil request bodies.
private struct Empty: Codable {}
