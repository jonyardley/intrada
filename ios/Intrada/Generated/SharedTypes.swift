// SharedTypes.swift
//
// Swift type definitions matching the Rust types that cross the Crux FFI boundary.
// These types are serialised/deserialised using JSON via CoreJson (serde_json).
//
// IMPORTANT: Rust serde uses "externally tagged" enum format by default:
//   Unit variant:    "VariantName"
//   Newtype variant: {"VariantName": value}
//   Struct variant:  {"VariantName": {"field1": v1, ...}}
//
// Swift Codable's default enum format differs, so custom encode/decode
// implementations are provided for all enums that cross the FFI boundary.
//
// Generated manually from crates/intrada-core/src/ — keep in sync with Rust types.
// TODO: Replace with automated typegen when serde-reflection GoalKind issue is resolved.

import Foundation

// MARK: - JSON Date Strategy

/// ISO 8601 date formatter matching chrono's default serde format (RFC3339).
/// Chrono serialises `DateTime<Utc>` as `"2026-03-04T15:30:45.123456Z"`.
extension JSONDecoder {
    static let intrada: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .custom { decoder in
            let container = try decoder.singleValueContainer()
            let string = try container.decode(String.self)
            // Try full fractional seconds first, then without
            let formatter = ISO8601DateFormatter()
            formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            if let date = formatter.date(from: string) { return date }
            formatter.formatOptions = [.withInternetDateTime]
            if let date = formatter.date(from: string) { return date }
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Cannot parse date: \(string)")
        }
        return decoder
    }()
}

extension JSONEncoder {
    static let intrada: JSONEncoder = {
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .custom { date, encoder in
            var container = encoder.singleValueContainer()
            let formatter = ISO8601DateFormatter()
            formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            try container.encode(formatter.string(from: date))
        }
        return encoder
    }()
}

// MARK: - Serde External Tagging Helpers

/// Helpers for encoding/decoding Rust serde's externally-tagged enum format.
///
/// Unit variant:    `"VariantName"` (a bare JSON string)
/// Newtype variant: `{"VariantName": value}`
/// Struct variant:  `{"VariantName": {"field1": v1, ...}}`
private enum SerdeExternalTag {

    /// Decode a single-key object `{"Key": Value}` and return (key, valueDecoder).
    static func decodeTag(from decoder: Decoder) throws -> (String, KeyedDecodingContainer<DynamicCodingKey>) {
        let container = try decoder.container(keyedBy: DynamicCodingKey.self)
        guard let key = container.allKeys.first else {
            throw DecodingError.dataCorrupted(
                .init(codingPath: decoder.codingPath, debugDescription: "Expected object with one key")
            )
        }
        return (key.stringValue, container)
    }

    /// Try to decode as a bare string first (unit variant), returns nil if not a string.
    static func decodeUnitVariant(from decoder: Decoder) -> String? {
        guard let container = try? decoder.singleValueContainer(),
              let value = try? container.decode(String.self) else {
            return nil
        }
        return value
    }
}

private struct DynamicCodingKey: CodingKey {
    var stringValue: String
    var intValue: Int?

    init(stringValue: String) {
        self.stringValue = stringValue
        self.intValue = nil
    }

    init?(intValue: Int) {
        self.stringValue = String(intValue)
        self.intValue = intValue
    }
}

// MARK: - Event (sent from Swift shell → Rust core)
// Rust: externally tagged, no special serde attrs

enum Event: Codable {
    case item(ItemEvent)
    case session(SessionEvent)
    case routine(RoutineEvent)
    case goal(GoalEvent)
    case dataLoaded(items: [Item])
    case sessionsLoaded(sessions: [PracticeSession])
    case routinesLoaded(routines: [Routine])
    case goalsLoaded(goals: [Goal])
    case loadFailed(String)
    case clearError
    case setQuery(ListQuery?)

    func encode(to encoder: Encoder) throws {
        switch self {
        case .item(let event):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(event, forKey: DynamicCodingKey(stringValue: "Item"))
        case .session(let event):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(event, forKey: DynamicCodingKey(stringValue: "Session"))
        case .routine(let event):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(event, forKey: DynamicCodingKey(stringValue: "Routine"))
        case .goal(let event):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(event, forKey: DynamicCodingKey(stringValue: "Goal"))
        case .dataLoaded(let items):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            let payload = DataLoadedPayload(items: items)
            try container.encode(payload, forKey: DynamicCodingKey(stringValue: "DataLoaded"))
        case .sessionsLoaded(let sessions):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            let payload = SessionsLoadedPayload(sessions: sessions)
            try container.encode(payload, forKey: DynamicCodingKey(stringValue: "SessionsLoaded"))
        case .routinesLoaded(let routines):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            let payload = RoutinesLoadedPayload(routines: routines)
            try container.encode(payload, forKey: DynamicCodingKey(stringValue: "RoutinesLoaded"))
        case .goalsLoaded(let goals):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            let payload = GoalsLoadedPayload(goals: goals)
            try container.encode(payload, forKey: DynamicCodingKey(stringValue: "GoalsLoaded"))
        case .loadFailed(let msg):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(msg, forKey: DynamicCodingKey(stringValue: "LoadFailed"))
        case .clearError:
            var container = encoder.singleValueContainer()
            try container.encode("ClearError")
        case .setQuery(let query):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(query, forKey: DynamicCodingKey(stringValue: "SetQuery"))
        }
    }

    init(from decoder: Decoder) throws {
        // Unit variant check
        if let unit = SerdeExternalTag.decodeUnitVariant(from: decoder) {
            switch unit {
            case "ClearError": self = .clearError
            default:
                throw DecodingError.dataCorrupted(
                    .init(codingPath: decoder.codingPath, debugDescription: "Unknown unit Event variant: \(unit)")
                )
            }
            return
        }

        let (tag, container) = try SerdeExternalTag.decodeTag(from: decoder)
        let key = DynamicCodingKey(stringValue: tag)

        switch tag {
        case "Item":
            self = .item(try container.decode(ItemEvent.self, forKey: key))
        case "Session":
            self = .session(try container.decode(SessionEvent.self, forKey: key))
        case "Routine":
            self = .routine(try container.decode(RoutineEvent.self, forKey: key))
        case "Goal":
            self = .goal(try container.decode(GoalEvent.self, forKey: key))
        case "DataLoaded":
            let payload = try container.decode(DataLoadedPayload.self, forKey: key)
            self = .dataLoaded(items: payload.items)
        case "SessionsLoaded":
            let payload = try container.decode(SessionsLoadedPayload.self, forKey: key)
            self = .sessionsLoaded(sessions: payload.sessions)
        case "RoutinesLoaded":
            let payload = try container.decode(RoutinesLoadedPayload.self, forKey: key)
            self = .routinesLoaded(routines: payload.routines)
        case "GoalsLoaded":
            let payload = try container.decode(GoalsLoadedPayload.self, forKey: key)
            self = .goalsLoaded(goals: payload.goals)
        case "LoadFailed":
            self = .loadFailed(try container.decode(String.self, forKey: key))
        case "SetQuery":
            self = .setQuery(try container.decodeIfPresent(ListQuery.self, forKey: key))
        default:
            throw DecodingError.dataCorrupted(
                .init(codingPath: decoder.codingPath, debugDescription: "Unknown Event variant: \(tag)")
            )
        }
    }
}

// Helper payloads for struct-variant Event encoding
private struct DataLoadedPayload: Codable { let items: [Item] }
private struct SessionsLoadedPayload: Codable { let sessions: [PracticeSession] }
private struct RoutinesLoadedPayload: Codable { let routines: [Routine] }
private struct GoalsLoadedPayload: Codable { let goals: [Goal] }

// MARK: - ViewModel (returned from Rust core → Swift shell)

struct ViewModel: Codable {
    var items: [LibraryItemView]
    var sessions: [PracticeSessionView]
    var activeSession: ActiveSessionData?
    var buildingSetlist: BuildingSetlistView?
    var summary: SummaryView?
    var sessionStatus: String
    var error: String?
    var analytics: AnalyticsData?
    var routines: [RoutineView]
    var goals: [GoalView]

    enum CodingKeys: String, CodingKey {
        case items, sessions
        case activeSession = "active_session"
        case buildingSetlist = "building_setlist"
        case summary
        case sessionStatus = "session_status"
        case error, analytics, routines, goals
    }
}

// MARK: - JsonEffect (decoded from CoreJson.process_event output)
// Rust: #[serde(tag = "type", content = "data")] — adjacently tagged

struct JsonEffect: Codable {
    let type: String
    let data: AppEffectPayload?

    var isRender: Bool { type == "Render" }
    var appEffect: AppEffect? { data?.effect }
}

/// Wrapper to decode the `data` field of a JsonEffect, which contains
/// an externally-tagged AppEffect value.
struct AppEffectPayload: Codable {
    let effect: AppEffect

    init(from decoder: Decoder) throws {
        effect = try AppEffect(from: decoder)
    }

    func encode(to encoder: Encoder) throws {
        try effect.encode(to: encoder)
    }
}

// MARK: - AppEffect (emitted from Rust core for shell to process)
// Rust: externally tagged, no special serde attrs

enum AppEffect: Codable {
    case loadAll
    case saveItem(Item)
    case updateItem(Item)
    case deleteItem(id: String)
    case loadSessions
    case savePracticeSession(PracticeSession)
    case deletePracticeSession(id: String)
    case saveSessionInProgress(ActiveSession)
    case clearSessionInProgress
    case saveRoutine(Routine)
    case updateRoutine(Routine)
    case deleteRoutine(id: String)
    case saveGoal(Goal)
    case updateGoal(Goal)
    case deleteGoal(id: String)
    case loadGoals

    init(from decoder: Decoder) throws {
        // Unit variants first
        if let unit = SerdeExternalTag.decodeUnitVariant(from: decoder) {
            switch unit {
            case "LoadAll": self = .loadAll
            case "LoadSessions": self = .loadSessions
            case "ClearSessionInProgress": self = .clearSessionInProgress
            case "LoadGoals": self = .loadGoals
            default:
                throw DecodingError.dataCorrupted(
                    .init(codingPath: decoder.codingPath, debugDescription: "Unknown unit AppEffect: \(unit)")
                )
            }
            return
        }

        let (tag, container) = try SerdeExternalTag.decodeTag(from: decoder)
        let key = DynamicCodingKey(stringValue: tag)

        switch tag {
        case "SaveItem":
            self = .saveItem(try container.decode(Item.self, forKey: key))
        case "UpdateItem":
            self = .updateItem(try container.decode(Item.self, forKey: key))
        case "DeleteItem":
            let payload = try container.decode(DeletePayload.self, forKey: key)
            self = .deleteItem(id: payload.id)
        case "SavePracticeSession":
            self = .savePracticeSession(try container.decode(PracticeSession.self, forKey: key))
        case "DeletePracticeSession":
            let payload = try container.decode(DeletePayload.self, forKey: key)
            self = .deletePracticeSession(id: payload.id)
        case "SaveSessionInProgress":
            self = .saveSessionInProgress(try container.decode(ActiveSession.self, forKey: key))
        case "SaveRoutine":
            self = .saveRoutine(try container.decode(Routine.self, forKey: key))
        case "UpdateRoutine":
            self = .updateRoutine(try container.decode(Routine.self, forKey: key))
        case "DeleteRoutine":
            let payload = try container.decode(DeletePayload.self, forKey: key)
            self = .deleteRoutine(id: payload.id)
        case "SaveGoal":
            self = .saveGoal(try container.decode(Goal.self, forKey: key))
        case "UpdateGoal":
            self = .updateGoal(try container.decode(Goal.self, forKey: key))
        case "DeleteGoal":
            let payload = try container.decode(DeletePayload.self, forKey: key)
            self = .deleteGoal(id: payload.id)
        default:
            throw DecodingError.dataCorrupted(
                .init(codingPath: decoder.codingPath, debugDescription: "Unknown AppEffect: \(tag)")
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        switch self {
        case .loadAll:
            var c = encoder.singleValueContainer()
            try c.encode("LoadAll")
        case .saveItem(let item):
            try container.encode(item, forKey: DynamicCodingKey(stringValue: "SaveItem"))
        case .updateItem(let item):
            try container.encode(item, forKey: DynamicCodingKey(stringValue: "UpdateItem"))
        case .deleteItem(let id):
            try container.encode(DeletePayload(id: id), forKey: DynamicCodingKey(stringValue: "DeleteItem"))
        case .loadSessions:
            var c = encoder.singleValueContainer()
            try c.encode("LoadSessions")
        case .savePracticeSession(let session):
            try container.encode(session, forKey: DynamicCodingKey(stringValue: "SavePracticeSession"))
        case .deletePracticeSession(let id):
            try container.encode(DeletePayload(id: id), forKey: DynamicCodingKey(stringValue: "DeletePracticeSession"))
        case .saveSessionInProgress(let session):
            try container.encode(session, forKey: DynamicCodingKey(stringValue: "SaveSessionInProgress"))
        case .clearSessionInProgress:
            var c = encoder.singleValueContainer()
            try c.encode("ClearSessionInProgress")
        case .saveRoutine(let routine):
            try container.encode(routine, forKey: DynamicCodingKey(stringValue: "SaveRoutine"))
        case .updateRoutine(let routine):
            try container.encode(routine, forKey: DynamicCodingKey(stringValue: "UpdateRoutine"))
        case .deleteRoutine(let id):
            try container.encode(DeletePayload(id: id), forKey: DynamicCodingKey(stringValue: "DeleteRoutine"))
        case .saveGoal(let goal):
            try container.encode(goal, forKey: DynamicCodingKey(stringValue: "SaveGoal"))
        case .updateGoal(let goal):
            try container.encode(goal, forKey: DynamicCodingKey(stringValue: "UpdateGoal"))
        case .deleteGoal(let id):
            try container.encode(DeletePayload(id: id), forKey: DynamicCodingKey(stringValue: "DeleteGoal"))
        case .loadGoals:
            var c = encoder.singleValueContainer()
            try c.encode("LoadGoals")
        }
    }
}

/// Helper for decoding `{"DeleteItem": {"id": "..."}}` etc.
private struct DeletePayload: Codable {
    let id: String
}

// MARK: - Domain Types: Items

enum ItemKind: String, Codable {
    case piece
    case exercise
}

struct Tempo: Codable {
    var marking: String?
    var bpm: UInt16?
}

struct Item: Codable {
    var id: String
    var title: String
    var kind: ItemKind
    var composer: String?
    var category: String?
    var key: String?
    var tempo: Tempo?
    var notes: String?
    var tags: [String]
    var createdAt: Date
    var updatedAt: Date

    enum CodingKeys: String, CodingKey {
        case id, title, kind, composer, category, key, tempo, notes, tags
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

struct CreateItem: Codable {
    var title: String
    var kind: ItemKind
    var composer: String?
    var category: String?
    var key: String?
    var tempo: Tempo?
    var notes: String?
    var tags: [String]
}

// Rust: externally tagged
enum ItemEvent: Codable {
    case add(CreateItem)
    case update(id: String, input: UpdateItem)
    case delete(id: String)
    case addTags(id: String, tags: [String])
    case removeTags(id: String, tags: [String])

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        switch self {
        case .add(let item):
            try container.encode(item, forKey: DynamicCodingKey(stringValue: "Add"))
        case .update(let id, let input):
            try container.encode(UpdatePayload(id: id, input: input), forKey: DynamicCodingKey(stringValue: "Update"))
        case .delete(let id):
            try container.encode(DeletePayload(id: id), forKey: DynamicCodingKey(stringValue: "Delete"))
        case .addTags(let id, let tags):
            try container.encode(TagsPayload(id: id, tags: tags), forKey: DynamicCodingKey(stringValue: "AddTags"))
        case .removeTags(let id, let tags):
            try container.encode(TagsPayload(id: id, tags: tags), forKey: DynamicCodingKey(stringValue: "RemoveTags"))
        }
    }

    init(from decoder: Decoder) throws {
        let (tag, container) = try SerdeExternalTag.decodeTag(from: decoder)
        let key = DynamicCodingKey(stringValue: tag)
        switch tag {
        case "Add":
            self = .add(try container.decode(CreateItem.self, forKey: key))
        case "Update":
            let p = try container.decode(UpdatePayload.self, forKey: key)
            self = .update(id: p.id, input: p.input)
        case "Delete":
            let p = try container.decode(DeletePayload.self, forKey: key)
            self = .delete(id: p.id)
        case "AddTags":
            let p = try container.decode(TagsPayload.self, forKey: key)
            self = .addTags(id: p.id, tags: p.tags)
        case "RemoveTags":
            let p = try container.decode(TagsPayload.self, forKey: key)
            self = .removeTags(id: p.id, tags: p.tags)
        default:
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Unknown ItemEvent: \(tag)"))
        }
    }
}

private struct UpdatePayload: Codable {
    let id: String
    let input: UpdateItem
}

private struct TagsPayload: Codable {
    let id: String
    let tags: [String]
}

struct UpdateItem: Codable {
    var title: String?
    var composer: String??
    var category: String??
    var key: String??
    var tempo: Tempo??
    var notes: String??
    var tags: [String]?
}

// MARK: - Domain Types: Sessions

enum EntryStatus: String, Codable {
    case completed = "Completed"
    case skipped = "Skipped"
    case notAttempted = "NotAttempted"
}

enum CompletionStatus: String, Codable {
    case completed = "Completed"
    case endedEarly = "EndedEarly"
}

/// Rep action delta: +1 = success, -1 = missed.
/// Serialised as i8 via serde_repr.
enum RepAction: Int8, Codable {
    case missed = -1
    case success = 1
}

struct SetlistEntry: Codable {
    var id: String
    var itemId: String
    var itemTitle: String
    var itemType: String
    var position: UInt
    var durationSecs: UInt64
    var status: EntryStatus
    var notes: String?
    var score: UInt8?
    var intention: String?
    var repTarget: UInt8?
    var repCount: UInt8?
    var repTargetReached: Bool?
    var repHistory: [RepAction]?
    var plannedDurationSecs: UInt32?
    var achievedTempo: UInt16?

    enum CodingKeys: String, CodingKey {
        case id
        case itemId = "item_id"
        case itemTitle = "item_title"
        case itemType = "item_type"
        case position
        case durationSecs = "duration_secs"
        case status, notes, score, intention
        case repTarget = "rep_target"
        case repCount = "rep_count"
        case repTargetReached = "rep_target_reached"
        case repHistory = "rep_history"
        case plannedDurationSecs = "planned_duration_secs"
        case achievedTempo = "achieved_tempo"
    }
}

struct PracticeSession: Codable {
    var id: String
    var entries: [SetlistEntry]
    var sessionNotes: String?
    var sessionIntention: String?
    var startedAt: Date
    var completedAt: Date
    var totalDurationSecs: UInt64
    var completionStatus: CompletionStatus

    enum CodingKeys: String, CodingKey {
        case id, entries
        case sessionNotes = "session_notes"
        case sessionIntention = "session_intention"
        case startedAt = "started_at"
        case completedAt = "completed_at"
        case totalDurationSecs = "total_duration_secs"
        case completionStatus = "completion_status"
    }
}

struct ActiveSession: Codable {
    var id: String
    var entries: [SetlistEntry]
    var currentIndex: UInt
    var currentItemStartedAt: Date
    var sessionStartedAt: Date
    var sessionIntention: String?

    enum CodingKeys: String, CodingKey {
        case id, entries
        case currentIndex = "current_index"
        case currentItemStartedAt = "current_item_started_at"
        case sessionStartedAt = "session_started_at"
        case sessionIntention = "session_intention"
    }
}

// Rust: externally tagged
enum SessionEvent: Codable {
    case startBuilding
    case setSessionIntention(intention: String?)
    case setEntryIntention(entryId: String, intention: String?)
    case setRepTarget(entryId: String, target: UInt8?)
    case setEntryDuration(entryId: String, durationSecs: UInt32?)
    case addToSetlist(itemId: String)
    case addNewItemToSetlist(title: String, itemType: String)
    case removeFromSetlist(entryId: String)
    case reorderSetlist(entryId: String, newPosition: UInt)
    case startSession(now: Date)
    case cancelBuilding
    case nextItem(now: Date)
    case previousItem(now: Date)
    case skipItem(now: Date)
    case finishSession(now: Date)
    case endSessionEarly(now: Date)
    case setEntryScore(entryId: String, score: UInt8)
    case setEntryNotes(entryId: String, notes: String?)
    case setSummaryNotes(notes: String?)
    case saveSession(now: Date)
    case discardSession
    case incrementRep
    case decrementRep
    case recoverSession(session: ActiveSession)
    case setAchievedTempo(entryId: String, tempo: UInt16?)

    // swiftlint:disable:next function_body_length cyclomatic_complexity
    func encode(to encoder: Encoder) throws {
        switch self {
        case .startBuilding:
            var c = encoder.singleValueContainer()
            try c.encode("StartBuilding")
        case .setSessionIntention(let intention):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(["intention": intention], forKey: DynamicCodingKey(stringValue: "SetSessionIntention"))
        case .setEntryIntention(let entryId, let intention):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                SetEntryIntentionPayload(entryId: entryId, intention: intention),
                forKey: DynamicCodingKey(stringValue: "SetEntryIntention")
            )
        case .setRepTarget(let entryId, let target):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                SetRepTargetPayload(entryId: entryId, target: target),
                forKey: DynamicCodingKey(stringValue: "SetRepTarget")
            )
        case .setEntryDuration(let entryId, let durationSecs):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                SetEntryDurationPayload(entryId: entryId, durationSecs: durationSecs),
                forKey: DynamicCodingKey(stringValue: "SetEntryDuration")
            )
        case .addToSetlist(let itemId):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                AddToSetlistPayload(itemId: itemId),
                forKey: DynamicCodingKey(stringValue: "AddToSetlist")
            )
        case .addNewItemToSetlist(let title, let itemType):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                AddNewItemPayload(title: title, itemType: itemType),
                forKey: DynamicCodingKey(stringValue: "AddNewItemToSetlist")
            )
        case .removeFromSetlist(let entryId):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                RemoveFromSetlistPayload(entryId: entryId),
                forKey: DynamicCodingKey(stringValue: "RemoveFromSetlist")
            )
        case .reorderSetlist(let entryId, let newPosition):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                ReorderPayload(entryId: entryId, newPosition: newPosition),
                forKey: DynamicCodingKey(stringValue: "ReorderSetlist")
            )
        case .startSession(let now):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(NowPayload(now: now), forKey: DynamicCodingKey(stringValue: "StartSession"))
        case .cancelBuilding:
            var c = encoder.singleValueContainer()
            try c.encode("CancelBuilding")
        case .nextItem(let now):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(NowPayload(now: now), forKey: DynamicCodingKey(stringValue: "NextItem"))
        case .previousItem(let now):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(NowPayload(now: now), forKey: DynamicCodingKey(stringValue: "PreviousItem"))
        case .skipItem(let now):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(NowPayload(now: now), forKey: DynamicCodingKey(stringValue: "SkipItem"))
        case .finishSession(let now):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(NowPayload(now: now), forKey: DynamicCodingKey(stringValue: "FinishSession"))
        case .endSessionEarly(let now):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(NowPayload(now: now), forKey: DynamicCodingKey(stringValue: "EndSessionEarly"))
        case .setEntryScore(let entryId, let score):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                SetEntryScorePayload(entryId: entryId, score: score),
                forKey: DynamicCodingKey(stringValue: "SetEntryScore")
            )
        case .setEntryNotes(let entryId, let notes):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                SetEntryNotesPayload(entryId: entryId, notes: notes),
                forKey: DynamicCodingKey(stringValue: "SetEntryNotes")
            )
        case .setSummaryNotes(let notes):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(["notes": notes], forKey: DynamicCodingKey(stringValue: "SetSummaryNotes"))
        case .saveSession(let now):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(NowPayload(now: now), forKey: DynamicCodingKey(stringValue: "SaveSession"))
        case .discardSession:
            var c = encoder.singleValueContainer()
            try c.encode("DiscardSession")
        case .incrementRep:
            var c = encoder.singleValueContainer()
            try c.encode("IncrementRep")
        case .decrementRep:
            var c = encoder.singleValueContainer()
            try c.encode("DecrementRep")
        case .restoreSession(let session):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                RestoreSessionPayload(session: session),
                forKey: DynamicCodingKey(stringValue: "RestoreSession")
            )
        case .setAchievedTempo(let entryId, let tempo):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            try container.encode(
                SetAchievedTempoPayload(entryId: entryId, tempo: tempo),
                forKey: DynamicCodingKey(stringValue: "SetAchievedTempo")
            )
        }
    }

    init(from decoder: Decoder) throws {
        if let unit = SerdeExternalTag.decodeUnitVariant(from: decoder) {
            switch unit {
            case "StartBuilding": self = .startBuilding
            case "CancelBuilding": self = .cancelBuilding
            case "DiscardSession": self = .discardSession
            case "IncrementRep": self = .incrementRep
            case "DecrementRep": self = .decrementRep
            default:
                throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Unknown unit SessionEvent: \(unit)"))
            }
            return
        }

        let (tag, container) = try SerdeExternalTag.decodeTag(from: decoder)
        let key = DynamicCodingKey(stringValue: tag)

        switch tag {
        case "SetSessionIntention":
            let p: [String: String?] = try container.decode([String: String?].self, forKey: key)
            self = .setSessionIntention(intention: p["intention"] ?? nil)
        case "SetEntryIntention":
            let p = try container.decode(SetEntryIntentionPayload.self, forKey: key)
            self = .setEntryIntention(entryId: p.entryId, intention: p.intention)
        case "SetRepTarget":
            let p = try container.decode(SetRepTargetPayload.self, forKey: key)
            self = .setRepTarget(entryId: p.entryId, target: p.target)
        case "SetEntryDuration":
            let p = try container.decode(SetEntryDurationPayload.self, forKey: key)
            self = .setEntryDuration(entryId: p.entryId, durationSecs: p.durationSecs)
        case "AddToSetlist":
            let p = try container.decode(AddToSetlistPayload.self, forKey: key)
            self = .addToSetlist(itemId: p.itemId)
        case "AddNewItemToSetlist":
            let p = try container.decode(AddNewItemPayload.self, forKey: key)
            self = .addNewItemToSetlist(title: p.title, itemType: p.itemType)
        case "RemoveFromSetlist":
            let p = try container.decode(RemoveFromSetlistPayload.self, forKey: key)
            self = .removeFromSetlist(entryId: p.entryId)
        case "ReorderSetlist":
            let p = try container.decode(ReorderPayload.self, forKey: key)
            self = .reorderSetlist(entryId: p.entryId, newPosition: p.newPosition)
        case "StartSession":
            let p = try container.decode(NowPayload.self, forKey: key)
            self = .startSession(now: p.now)
        case "NextItem":
            let p = try container.decode(NowPayload.self, forKey: key)
            self = .nextItem(now: p.now)
        case "PreviousItem":
            let p = try container.decode(NowPayload.self, forKey: key)
            self = .previousItem(now: p.now)
        case "SkipItem":
            let p = try container.decode(NowPayload.self, forKey: key)
            self = .skipItem(now: p.now)
        case "FinishSession":
            let p = try container.decode(NowPayload.self, forKey: key)
            self = .finishSession(now: p.now)
        case "EndSessionEarly":
            let p = try container.decode(NowPayload.self, forKey: key)
            self = .endSessionEarly(now: p.now)
        case "SetEntryScore":
            let p = try container.decode(SetEntryScorePayload.self, forKey: key)
            self = .setEntryScore(entryId: p.entryId, score: p.score)
        case "SetEntryNotes":
            let p = try container.decode(SetEntryNotesPayload.self, forKey: key)
            self = .setEntryNotes(entryId: p.entryId, notes: p.notes)
        case "SetSummaryNotes":
            let p: [String: String?] = try container.decode([String: String?].self, forKey: key)
            self = .setSummaryNotes(notes: p["notes"] ?? nil)
        case "SaveSession":
            let p = try container.decode(NowPayload.self, forKey: key)
            self = .saveSession(now: p.now)
        case "RestoreSession":
            let p = try container.decode(RestoreSessionPayload.self, forKey: key)
            self = .restoreSession(session: p.session)
        case "SetAchievedTempo":
            let p = try container.decode(SetAchievedTempoPayload.self, forKey: key)
            self = .setAchievedTempo(entryId: p.entryId, tempo: p.tempo)
        default:
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Unknown SessionEvent: \(tag)"))
        }
    }
}

// SessionEvent payload structs (field names match Rust snake_case via CodingKeys)
private struct SetEntryIntentionPayload: Codable {
    let entryId: String
    let intention: String?
    enum CodingKeys: String, CodingKey {
        case entryId = "entry_id"
        case intention
    }
}

private struct SetRepTargetPayload: Codable {
    let entryId: String
    let target: UInt8?
    enum CodingKeys: String, CodingKey {
        case entryId = "entry_id"
        case target
    }
}

private struct SetEntryDurationPayload: Codable {
    let entryId: String
    let durationSecs: UInt32?
    enum CodingKeys: String, CodingKey {
        case entryId = "entry_id"
        case durationSecs = "duration_secs"
    }
}

private struct AddToSetlistPayload: Codable {
    let itemId: String
    enum CodingKeys: String, CodingKey {
        case itemId = "item_id"
    }
}

private struct AddNewItemPayload: Codable {
    let title: String
    let itemType: String
    enum CodingKeys: String, CodingKey {
        case title
        case itemType = "item_type"
    }
}

private struct RemoveFromSetlistPayload: Codable {
    let entryId: String
    enum CodingKeys: String, CodingKey {
        case entryId = "entry_id"
    }
}

private struct ReorderPayload: Codable {
    let entryId: String
    let newPosition: UInt
    enum CodingKeys: String, CodingKey {
        case entryId = "entry_id"
        case newPosition = "new_position"
    }
}

private struct NowPayload: Codable {
    let now: Date
}

private struct SetEntryScorePayload: Codable {
    let entryId: String
    let score: UInt8
    enum CodingKeys: String, CodingKey {
        case entryId = "entry_id"
        case score
    }
}

private struct SetEntryNotesPayload: Codable {
    let entryId: String
    let notes: String?
    enum CodingKeys: String, CodingKey {
        case entryId = "entry_id"
        case notes
    }
}

private struct RestoreSessionPayload: Codable {
    let session: ActiveSession
}

private struct SetAchievedTempoPayload: Codable {
    let entryId: String
    let tempo: UInt16?
    enum CodingKeys: String, CodingKey {
        case entryId = "entry_id"
        case tempo
    }
}

// MARK: - Domain Types: Routines

struct RoutineEntry: Codable {
    var id: String
    var itemId: String
    var itemTitle: String
    var itemType: String
    var position: UInt

    enum CodingKeys: String, CodingKey {
        case id
        case itemId = "item_id"
        case itemTitle = "item_title"
        case itemType = "item_type"
        case position
    }
}

struct Routine: Codable {
    var id: String
    var name: String
    var entries: [RoutineEntry]
    var createdAt: Date
    var updatedAt: Date

    enum CodingKeys: String, CodingKey {
        case id, name, entries
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

// Rust: externally tagged
enum RoutineEvent: Codable {
    case saveBuildingAsRoutine(name: String)
    case saveSummaryAsRoutine(name: String)
    case loadRoutineIntoSetlist(routineId: String)
    case deleteRoutine(id: String)
    case updateRoutine(id: String, name: String, entries: [RoutineEntry])

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        switch self {
        case .saveBuildingAsRoutine(let name):
            try container.encode(
                RoutineNamePayload(name: name),
                forKey: DynamicCodingKey(stringValue: "SaveBuildingAsRoutine")
            )
        case .saveSummaryAsRoutine(let name):
            try container.encode(
                RoutineNamePayload(name: name),
                forKey: DynamicCodingKey(stringValue: "SaveSummaryAsRoutine")
            )
        case .loadRoutineIntoSetlist(let routineId):
            try container.encode(
                LoadRoutinePayload(routineId: routineId),
                forKey: DynamicCodingKey(stringValue: "LoadRoutineIntoSetlist")
            )
        case .deleteRoutine(let id):
            try container.encode(
                DeletePayload(id: id),
                forKey: DynamicCodingKey(stringValue: "DeleteRoutine")
            )
        case .updateRoutine(let id, let name, let entries):
            try container.encode(
                UpdateRoutinePayload(id: id, name: name, entries: entries),
                forKey: DynamicCodingKey(stringValue: "UpdateRoutine")
            )
        }
    }

    init(from decoder: Decoder) throws {
        let (tag, container) = try SerdeExternalTag.decodeTag(from: decoder)
        let key = DynamicCodingKey(stringValue: tag)
        switch tag {
        case "SaveBuildingAsRoutine":
            let p = try container.decode(RoutineNamePayload.self, forKey: key)
            self = .saveBuildingAsRoutine(name: p.name)
        case "SaveSummaryAsRoutine":
            let p = try container.decode(RoutineNamePayload.self, forKey: key)
            self = .saveSummaryAsRoutine(name: p.name)
        case "LoadRoutineIntoSetlist":
            let p = try container.decode(LoadRoutinePayload.self, forKey: key)
            self = .loadRoutineIntoSetlist(routineId: p.routineId)
        case "DeleteRoutine":
            let p = try container.decode(DeletePayload.self, forKey: key)
            self = .deleteRoutine(id: p.id)
        case "UpdateRoutine":
            let p = try container.decode(UpdateRoutinePayload.self, forKey: key)
            self = .updateRoutine(id: p.id, name: p.name, entries: p.entries)
        default:
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Unknown RoutineEvent: \(tag)"))
        }
    }
}

private struct RoutineNamePayload: Codable {
    let name: String
}

private struct LoadRoutinePayload: Codable {
    let routineId: String
    enum CodingKeys: String, CodingKey {
        case routineId = "routine_id"
    }
}

private struct UpdateRoutinePayload: Codable {
    let id: String
    let name: String
    let entries: [RoutineEntry]
}

// MARK: - Domain Types: Goals

// Rust: #[serde(rename_all = "snake_case")]
enum GoalStatus: String, Codable {
    case active
    case completed
    case archived
}

// Rust: #[serde(rename_all = "snake_case", tag = "type")] — internally tagged
enum GoalKind: Codable {
    case sessionFrequency(targetDaysPerWeek: UInt8)
    case practiceTime(targetMinutesPerWeek: UInt32)
    case itemMastery(itemId: String, targetScore: UInt8)
    case milestone(description: String)

    private enum TypeKey: String, CodingKey {
        case type
    }

    func encode(to encoder: Encoder) throws {
        var typeContainer = encoder.container(keyedBy: TypeKey.self)
        switch self {
        case .sessionFrequency(let targetDaysPerWeek):
            try typeContainer.encode("session_frequency", forKey: .type)
            var container = encoder.container(keyedBy: SessionFrequencyKeys.self)
            try container.encode(targetDaysPerWeek, forKey: .targetDaysPerWeek)
        case .practiceTime(let targetMinutesPerWeek):
            try typeContainer.encode("practice_time", forKey: .type)
            var container = encoder.container(keyedBy: PracticeTimeKeys.self)
            try container.encode(targetMinutesPerWeek, forKey: .targetMinutesPerWeek)
        case .itemMastery(let itemId, let targetScore):
            try typeContainer.encode("item_mastery", forKey: .type)
            var container = encoder.container(keyedBy: ItemMasteryKeys.self)
            try container.encode(itemId, forKey: .itemId)
            try container.encode(targetScore, forKey: .targetScore)
        case .milestone(let description):
            try typeContainer.encode("milestone", forKey: .type)
            var container = encoder.container(keyedBy: MilestoneKeys.self)
            try container.encode(description, forKey: .description)
        }
    }

    init(from decoder: Decoder) throws {
        let typeContainer = try decoder.container(keyedBy: TypeKey.self)
        let type = try typeContainer.decode(String.self, forKey: .type)
        switch type {
        case "session_frequency":
            let c = try decoder.container(keyedBy: SessionFrequencyKeys.self)
            self = .sessionFrequency(targetDaysPerWeek: try c.decode(UInt8.self, forKey: .targetDaysPerWeek))
        case "practice_time":
            let c = try decoder.container(keyedBy: PracticeTimeKeys.self)
            self = .practiceTime(targetMinutesPerWeek: try c.decode(UInt32.self, forKey: .targetMinutesPerWeek))
        case "item_mastery":
            let c = try decoder.container(keyedBy: ItemMasteryKeys.self)
            self = .itemMastery(
                itemId: try c.decode(String.self, forKey: .itemId),
                targetScore: try c.decode(UInt8.self, forKey: .targetScore)
            )
        case "milestone":
            let c = try decoder.container(keyedBy: MilestoneKeys.self)
            self = .milestone(description: try c.decode(String.self, forKey: .description))
        default:
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Unknown GoalKind type: \(type)"))
        }
    }

    private enum SessionFrequencyKeys: String, CodingKey {
        case targetDaysPerWeek = "target_days_per_week"
    }
    private enum PracticeTimeKeys: String, CodingKey {
        case targetMinutesPerWeek = "target_minutes_per_week"
    }
    private enum ItemMasteryKeys: String, CodingKey {
        case itemId = "item_id"
        case targetScore = "target_score"
    }
    private enum MilestoneKeys: String, CodingKey {
        case description
    }
}

struct Goal: Codable {
    var id: String
    var title: String
    var kind: GoalKind
    var status: GoalStatus
    var deadline: Date?
    var createdAt: Date
    var updatedAt: Date
    var completedAt: Date?

    enum CodingKeys: String, CodingKey {
        case id, title, kind, status, deadline
        case createdAt = "created_at"
        case updatedAt = "updated_at"
        case completedAt = "completed_at"
    }
}

struct CreateGoal: Codable {
    var title: String
    var kind: GoalKind
    var deadline: Date?
}

struct UpdateGoal: Codable {
    var title: String?
    var deadline: Date??
}

// Rust: externally tagged
enum GoalEvent: Codable {
    case add(CreateGoal)
    case update(id: String, input: UpdateGoal)
    case complete(id: String)
    case archive(id: String)
    case reactivate(id: String)
    case delete(id: String)

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        switch self {
        case .add(let goal):
            try container.encode(goal, forKey: DynamicCodingKey(stringValue: "Add"))
        case .update(let id, let input):
            try container.encode(GoalUpdatePayload(id: id, input: input), forKey: DynamicCodingKey(stringValue: "Update"))
        case .complete(let id):
            try container.encode(DeletePayload(id: id), forKey: DynamicCodingKey(stringValue: "Complete"))
        case .archive(let id):
            try container.encode(DeletePayload(id: id), forKey: DynamicCodingKey(stringValue: "Archive"))
        case .reactivate(let id):
            try container.encode(DeletePayload(id: id), forKey: DynamicCodingKey(stringValue: "Reactivate"))
        case .delete(let id):
            try container.encode(DeletePayload(id: id), forKey: DynamicCodingKey(stringValue: "Delete"))
        }
    }

    init(from decoder: Decoder) throws {
        let (tag, container) = try SerdeExternalTag.decodeTag(from: decoder)
        let key = DynamicCodingKey(stringValue: tag)
        switch tag {
        case "Add":
            self = .add(try container.decode(CreateGoal.self, forKey: key))
        case "Update":
            let p = try container.decode(GoalUpdatePayload.self, forKey: key)
            self = .update(id: p.id, input: p.input)
        case "Complete":
            let p = try container.decode(DeletePayload.self, forKey: key)
            self = .complete(id: p.id)
        case "Archive":
            let p = try container.decode(DeletePayload.self, forKey: key)
            self = .archive(id: p.id)
        case "Reactivate":
            let p = try container.decode(DeletePayload.self, forKey: key)
            self = .reactivate(id: p.id)
        case "Delete":
            let p = try container.decode(DeletePayload.self, forKey: key)
            self = .delete(id: p.id)
        default:
            throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Unknown GoalEvent: \(tag)"))
        }
    }
}

private struct GoalUpdatePayload: Codable {
    let id: String
    let input: UpdateGoal
}

// MARK: - Domain Types: Filters

struct ListQuery: Codable {
    var text: String?
    var itemType: ItemKind?
    var key: String?
    var category: String?
    var tags: [String]?

    enum CodingKeys: String, CodingKey {
        case text
        case itemType = "item_type"
        case key, category, tags
    }
}

// MARK: - View Model Types

struct LibraryItemView: Codable {
    var id: String
    var itemType: String
    var title: String
    var subtitle: String
    var category: String?
    var key: String?
    var tempo: String?
    var notes: String?
    var tags: [String]
    var createdAt: String
    var updatedAt: String
    var practice: ItemPracticeSummary?
    var latestAchievedTempo: UInt16?

    enum CodingKeys: String, CodingKey {
        case id
        case itemType = "item_type"
        case title, subtitle, category, key, tempo, notes, tags
        case createdAt = "created_at"
        case updatedAt = "updated_at"
        case practice
        case latestAchievedTempo = "latest_achieved_tempo"
    }
}

struct ItemPracticeSummary: Codable {
    var sessionCount: UInt
    var totalMinutes: UInt32
    var latestScore: UInt8?
    var scoreHistory: [ScoreHistoryEntry]
    var latestTempo: UInt16?
    var tempoHistory: [TempoHistoryEntry]

    enum CodingKeys: String, CodingKey {
        case sessionCount = "session_count"
        case totalMinutes = "total_minutes"
        case latestScore = "latest_score"
        case scoreHistory = "score_history"
        case latestTempo = "latest_tempo"
        case tempoHistory = "tempo_history"
    }
}

struct ScoreHistoryEntry: Codable {
    var sessionDate: String
    var score: UInt8
    var sessionId: String

    enum CodingKeys: String, CodingKey {
        case sessionDate = "session_date"
        case score
        case sessionId = "session_id"
    }
}

struct TempoHistoryEntry: Codable {
    var sessionDate: String
    var tempo: UInt16
    var sessionId: String

    enum CodingKeys: String, CodingKey {
        case sessionDate = "session_date"
        case tempo
        case sessionId = "session_id"
    }
}

struct PracticeSessionView: Codable {
    var id: String
    var startedAt: String
    var finishedAt: String
    var totalDurationDisplay: String
    var completionStatus: String
    var notes: String?
    var entries: [SetlistEntryView]
    var sessionIntention: String?

    enum CodingKeys: String, CodingKey {
        case id
        case startedAt = "started_at"
        case finishedAt = "finished_at"
        case totalDurationDisplay = "total_duration_display"
        case completionStatus = "completion_status"
        case notes, entries
        case sessionIntention = "session_intention"
    }
}

struct SetlistEntryView: Codable {
    var id: String
    var itemId: String
    var itemTitle: String
    var itemType: String
    var position: UInt
    var durationDisplay: String
    var status: String
    var notes: String?
    var score: UInt8?
    var intention: String?
    var repTarget: UInt8?
    var repCount: UInt8?
    var repTargetReached: Bool?
    var repHistory: [RepAction]?
    var plannedDurationSecs: UInt32?
    var plannedDurationDisplay: String?
    var achievedTempo: UInt16?

    enum CodingKeys: String, CodingKey {
        case id
        case itemId = "item_id"
        case itemTitle = "item_title"
        case itemType = "item_type"
        case position
        case durationDisplay = "duration_display"
        case status, notes, score, intention
        case repTarget = "rep_target"
        case repCount = "rep_count"
        case repTargetReached = "rep_target_reached"
        case repHistory = "rep_history"
        case plannedDurationSecs = "planned_duration_secs"
        case plannedDurationDisplay = "planned_duration_display"
        case achievedTempo = "achieved_tempo"
    }
}

struct ActiveSessionData: Codable {
    var currentItemTitle: String
    var currentItemType: String
    var currentPosition: UInt
    var totalItems: UInt
    var startedAt: String
    var entries: [SetlistEntryView]
    var sessionIntention: String?
    var currentRepTarget: UInt8?
    var currentRepCount: UInt8?
    var currentRepTargetReached: Bool?
    var currentRepHistory: [RepAction]?
    var currentPlannedDurationSecs: UInt32?
    var nextItemTitle: String?

    enum CodingKeys: String, CodingKey {
        case currentItemTitle = "current_item_title"
        case currentItemType = "current_item_type"
        case currentPosition = "current_position"
        case totalItems = "total_items"
        case startedAt = "started_at"
        case entries
        case sessionIntention = "session_intention"
        case currentRepTarget = "current_rep_target"
        case currentRepCount = "current_rep_count"
        case currentRepTargetReached = "current_rep_target_reached"
        case currentRepHistory = "current_rep_history"
        case currentPlannedDurationSecs = "current_planned_duration_secs"
        case nextItemTitle = "next_item_title"
    }
}

struct BuildingSetlistView: Codable {
    var entries: [SetlistEntryView]
    var itemCount: UInt
    var sessionIntention: String?

    enum CodingKeys: String, CodingKey {
        case entries
        case itemCount = "item_count"
        case sessionIntention = "session_intention"
    }
}

struct SummaryView: Codable {
    var totalDurationDisplay: String
    var completionStatus: String
    var notes: String?
    var entries: [SetlistEntryView]
    var sessionIntention: String?

    enum CodingKeys: String, CodingKey {
        case totalDurationDisplay = "total_duration_display"
        case completionStatus = "completion_status"
        case notes, entries
        case sessionIntention = "session_intention"
    }
}

struct RoutineView: Codable {
    var id: String
    var name: String
    var entryCount: UInt
    var entries: [RoutineEntryView]

    enum CodingKeys: String, CodingKey {
        case id, name
        case entryCount = "entry_count"
        case entries
    }
}

struct RoutineEntryView: Codable {
    var id: String
    var itemId: String
    var itemTitle: String
    var itemType: String
    var position: UInt

    enum CodingKeys: String, CodingKey {
        case id
        case itemId = "item_id"
        case itemTitle = "item_title"
        case itemType = "item_type"
        case position
    }
}

struct GoalView: Codable {
    var id: String
    var title: String
    var kindLabel: String
    var kindType: String
    var status: String
    var progress: GoalProgress?
    var deadline: String?
    var createdAt: String
    var completedAt: String?
    var itemId: String?
    var itemTitle: String?

    enum CodingKeys: String, CodingKey {
        case id, title
        case kindLabel = "kind_label"
        case kindType = "kind_type"
        case status, progress, deadline
        case createdAt = "created_at"
        case completedAt = "completed_at"
        case itemId = "item_id"
        case itemTitle = "item_title"
    }
}

struct GoalProgress: Codable {
    var currentValue: Double
    var targetValue: Double
    var percentage: Double
    var displayText: String

    enum CodingKeys: String, CodingKey {
        case currentValue = "current_value"
        case targetValue = "target_value"
        case percentage
        case displayText = "display_text"
    }
}

// MARK: - Analytics View Types

enum Direction: String, Codable {
    case up
    case down
    case same
}

struct AnalyticsData: Codable {
    var weeklySummary: WeeklySummary
    var streak: PracticeStreak
    var dailyTotals: [DailyPracticeTotal]
    var topItems: [ItemRanking]
    var scoreTrends: [ItemScoreTrend]
    var neglectedItems: [NeglectedItem]
    var scoreChanges: [ScoreChange]

    enum CodingKeys: String, CodingKey {
        case weeklySummary = "weekly_summary"
        case streak
        case dailyTotals = "daily_totals"
        case topItems = "top_items"
        case scoreTrends = "score_trends"
        case neglectedItems = "neglected_items"
        case scoreChanges = "score_changes"
    }
}

struct WeeklySummary: Codable {
    var totalMinutes: UInt32
    var sessionCount: UInt
    var itemsCovered: UInt
    var prevTotalMinutes: UInt32
    var prevSessionCount: UInt
    var prevItemsCovered: UInt
    var timeDirection: Direction
    var sessionsDirection: Direction
    var itemsDirection: Direction
    var hasPrevWeekData: Bool

    enum CodingKeys: String, CodingKey {
        case totalMinutes = "total_minutes"
        case sessionCount = "session_count"
        case itemsCovered = "items_covered"
        case prevTotalMinutes = "prev_total_minutes"
        case prevSessionCount = "prev_session_count"
        case prevItemsCovered = "prev_items_covered"
        case timeDirection = "time_direction"
        case sessionsDirection = "sessions_direction"
        case itemsDirection = "items_direction"
        case hasPrevWeekData = "has_prev_week_data"
    }
}

struct PracticeStreak: Codable {
    var currentDays: UInt32

    enum CodingKeys: String, CodingKey {
        case currentDays = "current_days"
    }
}

struct DailyPracticeTotal: Codable {
    var date: String
    var minutes: UInt32
}

struct ItemRanking: Codable {
    var itemId: String
    var itemTitle: String
    var itemType: String
    var totalMinutes: UInt32
    var sessionCount: UInt

    enum CodingKeys: String, CodingKey {
        case itemId = "item_id"
        case itemTitle = "item_title"
        case itemType = "item_type"
        case totalMinutes = "total_minutes"
        case sessionCount = "session_count"
    }
}

struct ItemScoreTrend: Codable {
    var itemId: String
    var itemTitle: String
    var scores: [ScorePoint]
    var latestScore: UInt8

    enum CodingKeys: String, CodingKey {
        case itemId = "item_id"
        case itemTitle = "item_title"
        case scores
        case latestScore = "latest_score"
    }
}

struct ScorePoint: Codable {
    var date: String
    var score: UInt8
}

struct NeglectedItem: Codable {
    var itemId: String
    var itemTitle: String
    var daysSincePractice: UInt32?

    enum CodingKeys: String, CodingKey {
        case itemId = "item_id"
        case itemTitle = "item_title"
        case daysSincePractice = "days_since_practice"
    }
}

struct ScoreChange: Codable {
    var itemId: String
    var itemTitle: String
    var previousScore: UInt8?
    var currentScore: UInt8
    var delta: Int8
    var isNew: Bool

    enum CodingKeys: String, CodingKey {
        case itemId = "item_id"
        case itemTitle = "item_title"
        case previousScore = "previous_score"
        case currentScore = "current_score"
        case delta
        case isNew = "is_new"
    }
}
