// IntradaCore.swift
//
// Swift shell effect processor for Intrada.
// Port of crates/intrada-web/src/core_bridge.rs.
//
// Wraps CoreFfi (Rust FFI via BCS/bincode) and dispatches all AppEffect variants
// to the API client, session storage, and view model updates.
//
// The CoreFfi bridge uses the Request/Resolve pattern:
//   1. update(eventBytes) → [Request] (each with id + effect)
//   2. Shell processes each effect (Render → read view, App → execute side effect)
//   3. Shell calls resolve(id, responseBytes) for App effects → more [Request]

import Foundation
import Observation

/// The main effect processor for the Intrada iOS shell.
///
/// Holds the Rust `CoreFfi` instance and publishes the `ViewModel` for SwiftUI.
/// All state changes flow through: UI -> `update(Event)` -> Core -> effects -> API -> re-render.
@Observable
@MainActor
final class IntradaCore {

    // MARK: - Published State

    /// The current view model, updated after every Render effect.
    private(set) var viewModel: ViewModel

    /// Whether the initial data load is in progress.
    private(set) var isLoading = true

    // MARK: - Private State

    private let core: CoreFfi
    private let api: APIClient

    // MARK: - Init

    init(api: APIClient = APIClient()) {
        let core = CoreFfi()
        self.core = core
        self.api = api

        // Read initial ViewModel from the core (empty lists, default values)
        let data = core.view()
        do {
            self.viewModel = try ViewModel.bincodeDeserialize(input: [UInt8](data))
        } catch {
            fatalError("[IntradaCore] Failed to deserialize initial ViewModel: \(error)")
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
            reportError("Failed to serialize event: \(error)")
            return
        }

        let responseData = core.update(Data(eventBytes))
        processRequests(responseData)
    }

    /// Load initial data from the API.
    ///
    /// Called once after sign-in. Fetches items, sessions, routines, and goals
    /// in parallel, then sends the loaded data to the core.
    func fetchInitialData() {
        isLoading = true

        Task {
            await withTaskGroup(of: Void.self) { group in
                group.addTask { await self.fetchItems() }
                group.addTask { await self.fetchSessions() }
                group.addTask { await self.fetchRoutines() }
                group.addTask { await self.fetchGoals() }
            }

            // Restore any in-progress session from crash recovery
            if let savedSession = SessionStorage.load() {
                update(.session(.recoverSession(session: savedSession)))
            }

            isLoading = false
        }
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
            reportError("Failed to deserialize requests: \(error)")
            return
        }

        for request in requests {
            switch request.effect {
            case .render:
                refreshViewModel()
            case .app(let appEffect):
                handleAppEffect(appEffect, requestId: request.id)
            }
        }
    }

    /// Read the current ViewModel from the core via BCS.
    private func refreshViewModel() {
        let data = core.view()
        do {
            viewModel = try ViewModel.bincodeDeserialize(input: [UInt8](data))
        } catch {
            reportError("Failed to deserialize ViewModel: \(error)")
        }
    }

    /// Tell the core an App effect has been resolved, then process any new requests.
    private func resolveEffect(_ requestId: UInt32) {
        let moreData = core.resolve(requestId, Data())
        processRequests(moreData)
    }

    // MARK: - AppEffect Dispatch

    // swiftlint:disable:next function_body_length cyclomatic_complexity
    private func handleAppEffect(_ effect: AppEffect, requestId: UInt32) {
        Task {
            switch effect {
            case .loadAll:
                await fetchAllData()

            case .saveItem(let item):
                await spawnMutate(.library) {
                    try await self.api.postJSON("/api/items", body: item) as Item
                }

            case .updateItem(let item):
                await spawnMutate(.library) {
                    try await self.api.putJSON("/api/items/\(item.id)", body: item) as Item
                }

            case .deleteItem(let id):
                await spawnMutate(.library) {
                    try await self.api.delete("/api/items/\(id)")
                }

            case .loadSessions:
                await fetchSessions()

            case .savePracticeSession(let session):
                await spawnMutate(.sessions) {
                    try await self.api.postJSON("/api/sessions", body: session) as PracticeSession
                }

            case .deletePracticeSession(let id):
                await spawnMutate(.sessions) {
                    try await self.api.delete("/api/sessions/\(id)")
                }

            case .saveSessionInProgress(let session):
                SessionStorage.save(session)

            case .clearSessionInProgress:
                SessionStorage.clear()

            case .saveRoutine(let routine):
                await spawnMutate(.routines) {
                    try await self.api.postJSON("/api/routines", body: routine) as Routine
                }

            case .updateRoutine(let routine):
                await spawnMutate(.routines) {
                    try await self.api.putJSON("/api/routines/\(routine.id)", body: routine) as Routine
                }

            case .deleteRoutine(let id):
                await spawnMutate(.routines) {
                    try await self.api.delete("/api/routines/\(id)")
                }

            case .saveGoal(let goal):
                await spawnMutate(.goals) {
                    try await self.api.postJSON("/api/goals", body: goal) as Goal
                }

            case .updateGoal(let goal):
                await spawnMutate(.goals) {
                    try await self.api.putJSON("/api/goals/\(goal.id)", body: goal) as Goal
                }

            case .deleteGoal(let id):
                await spawnMutate(.goals) {
                    try await self.api.delete("/api/goals/\(id)")
                }

            case .loadGoals:
                await fetchGoals()
            }

            // Resolve the effect to notify the core it's complete.
            // This may produce further requests (e.g., additional Render effects).
            resolveEffect(requestId)
        }
    }

    // MARK: - Refresh-After-Mutate

    /// Which data to refresh after a mutation.
    private enum RefreshKind {
        case library
        case sessions
        case routines
        case goals
    }

    /// Execute a mutation, then refresh the relevant data from the API.
    ///
    /// Mirrors the `spawn_mutate()` pattern from `core_bridge.rs`.
    private func spawnMutate<T: Sendable>(_ kind: RefreshKind, action: @escaping @Sendable () async throws -> T) async {
        do {
            _ = try await action()
        } catch {
            reportError("Mutation failed: \(error.localizedDescription)")
            return
        }

        switch kind {
        case .library:
            await fetchItems()
        case .sessions:
            await fetchSessions()
        case .routines:
            await fetchRoutines()
        case .goals:
            await fetchGoals()
        }
    }

    // MARK: - Data Fetching

    private func fetchAllData() async {
        await withTaskGroup(of: Void.self) { group in
            group.addTask { await self.fetchItems() }
            group.addTask { await self.fetchSessions() }
            group.addTask { await self.fetchRoutines() }
            group.addTask { await self.fetchGoals() }
        }
    }

    private func fetchItems() async {
        do {
            let items = try await api.getItems()
            update(.dataLoaded(items: items))
        } catch {
            reportError("Failed to load items: \(error.localizedDescription)")
        }
    }

    private func fetchSessions() async {
        do {
            let sessions = try await api.getSessions()
            update(.sessionsLoaded(sessions: sessions))
        } catch {
            reportError("Failed to load sessions: \(error.localizedDescription)")
        }
    }

    private func fetchRoutines() async {
        do {
            let routines = try await api.getRoutines()
            update(.routinesLoaded(routines: routines))
        } catch {
            reportError("Failed to load routines: \(error.localizedDescription)")
        }
    }

    private func fetchGoals() async {
        do {
            let goals = try await api.getGoals()
            update(.goalsLoaded(goals: goals))
        } catch {
            reportError("Failed to load goals: \(error.localizedDescription)")
        }
    }

    // MARK: - Error Reporting

    private func reportError(_ message: String) {
        print("[IntradaCore] \(message)")
        update(.loadFailed(message))
    }
}
