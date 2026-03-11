// IntradaCore.swift
//
// Swift shell effect processor for Intrada.
// Port of crates/intrada-web/src/core_bridge.rs.
//
// Wraps CoreJson (Rust FFI via JSON) and dispatches all AppEffect variants
// to the API client, session storage, and view model updates.
//
// Effect handling is added incrementally as features are implemented.

import Foundation
import Observation

/// The main effect processor for the Intrada iOS shell.
///
/// Holds the Rust `CoreJson` instance and publishes the `ViewModel` for SwiftUI.
/// All state changes flow through: UI -> `update(Event)` -> Core -> effects -> API -> re-render.
@Observable
@MainActor
final class IntradaCore {

    // MARK: - Published State

    /// The current view model, updated after every Render effect.
    private(set) var viewModel = ViewModel(
        items: [],
        sessions: [],
        activeSession: nil,
        buildingSetlist: nil,
        summary: nil,
        sessionStatus: "idle",
        error: nil,
        analytics: nil,
        routines: [],
        goals: []
    )

    /// Whether the initial data load is in progress.
    private(set) var isLoading = true

    // MARK: - Private State

    private let core: CoreJson
    private let api: APIClient

    // MARK: - Init

    init(api: APIClient = APIClient()) {
        self.core = CoreJson()
        self.api = api
    }

    // MARK: - Public API

    /// Send an event to the Rust core and process resulting effects.
    ///
    /// This is the primary entry point for all UI interactions.
    func update(_ event: Event) {
        let eventJSON: String
        do {
            let data = try JSONEncoder.intrada.encode(event)
            guard let str = String(data: data, encoding: .utf8) else {
                reportError("Failed to encode event as UTF-8 string")
                return
            }
            eventJSON = str
        } catch {
            reportError("Failed to encode event: \(error)")
            return
        }

        let effectsJSON = core.processEvent(eventJSON)
        processEffects(effectsJSON)
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
                update(.session(.restoreSession(session: savedSession)))
            }

            isLoading = false
        }
    }

    // MARK: - Effect Processing

    private func processEffects(_ effectsJSON: String) {
        guard let data = effectsJSON.data(using: .utf8) else {
            reportError("Effects JSON is not valid UTF-8")
            return
        }

        let effects: [JsonEffect]
        do {
            effects = try JSONDecoder.intrada.decode([JsonEffect].self, from: data)
        } catch {
            reportError("Failed to decode effects: \(error)")
            return
        }

        for effect in effects {
            if effect.isRender {
                refreshViewModel()
            } else if let appEffect = effect.appEffect {
                handleAppEffect(appEffect)
            }
        }
    }

    private func refreshViewModel() {
        let viewJSON = core.view()
        guard let data = viewJSON.data(using: .utf8) else {
            reportError("View JSON is not valid UTF-8")
            return
        }
        do {
            viewModel = try JSONDecoder.intrada.decode(ViewModel.self, from: data)
        } catch {
            reportError("Failed to decode ViewModel: \(error)")
        }
    }

    // MARK: - AppEffect Dispatch

    // swiftlint:disable:next function_body_length cyclomatic_complexity
    private func handleAppEffect(_ effect: AppEffect) {
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
    private func spawnMutate<T>(_ kind: RefreshKind, action: @escaping () async throws -> T) async {
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
