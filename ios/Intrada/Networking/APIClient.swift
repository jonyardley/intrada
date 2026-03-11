// APIClient.swift
//
// HTTP client for the Intrada REST API.
// Port of crates/intrada-web/src/api_client.rs.
//
// All requests include `Authorization: Bearer {jwt}` from Clerk.
// Built-in 401 retry: get fresh token, retry once.
//
// Endpoints are added incrementally as features are implemented.

import ClerkKit
import Foundation

/// Actor-isolated HTTP client for the Intrada REST API.
///
/// All requests are authenticated via Clerk JWT with automatic 401 retry.
/// Add feature-specific endpoints here as each feature is implemented.
actor APIClient {

    private let baseURL: String
    private let session: URLSession

    init(baseURL: String = Config.apiBaseURL) {
        self.baseURL = baseURL
        self.session = URLSession.shared
    }

    // MARK: - Items (added with Library feature)

    func getItems() async throws -> [Item] {
        try await getJSON("/api/items")
    }

    // MARK: - Sessions (added with Practice feature)

    func getSessions() async throws -> [PracticeSession] {
        try await getJSON("/api/sessions")
    }

    // MARK: - Routines (added with Routines feature)

    func getRoutines() async throws -> [Routine] {
        try await getJSON("/api/routines")
    }

    // MARK: - Goals (added with Goals feature)

    func getGoals() async throws -> [Goal] {
        try await getJSON("/api/goals")
    }

    // MARK: - Generic Helpers with 401 Retry

    func getJSON<T: Decodable>(_ path: String) async throws -> T {
        try await requestWithRetry(path: path, method: "GET", body: nil as Empty?)
    }

    func postJSON<B: Encodable, T: Decodable>(_ path: String, body: B) async throws -> T {
        try await requestWithRetry(path: path, method: "POST", body: body)
    }

    func putJSON<B: Encodable, T: Decodable>(_ path: String, body: B) async throws -> T {
        try await requestWithRetry(path: path, method: "PUT", body: body)
    }

    func delete(_ path: String) async throws {
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
                print("[APIClient] Warning: No auth token for \(method) \(path)")
            }
        } catch {
            print("[APIClient] Warning: Failed to get auth token: \(error)")
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
            print("[APIClient] Warning: No Clerk session available")
            return nil
        }
        return try await session.getToken(.init(skipCache: forceRefresh))
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
struct Empty: Codable {}
