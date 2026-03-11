// CodableExtensions.swift
//
// Codable conformances for auto-generated types that cross the REST API boundary.
//
// The shared_types crate generates Swift types with BCS (Binary Canonical Serialization)
// for the Crux FFI bridge. This file adds JSON Codable for the subset of types that
// also need to be serialised as JSON for the REST API and UserDefaults storage.
//
// Types that ONLY cross the BCS bridge (Event, Effect, ViewModel, view types) do NOT
// need Codable and are intentionally omitted.
//
// IMPORTANT: This file must stay in sync with the Rust serde attributes.
// When domain types change, regenerate with `just typegen` and update this file.

import Foundation

// MARK: - Indirect Property Wrapper Codable

// Auto-generated structs use @Indirect for all properties. These transparent
// extensions let Swift's auto-synthesised Codable encode/decode the wrapped value.

extension Indirect: Encodable where T: Encodable {
    public func encode(to encoder: Encoder) throws {
        try wrappedValue.encode(to: encoder)
    }
}

extension Indirect: Decodable where T: Decodable {
    public init(from decoder: Decoder) throws {
        self.init(wrappedValue: try T(from: decoder))
    }
}

// MARK: - JSON Coders for REST API

extension JSONDecoder {
    /// Decoder for the Intrada REST API.
    ///
    /// Uses `.convertFromSnakeCase` to match Rust's default serde field naming.
    /// Date fields are String in the auto-generated types, so no custom date
    /// strategy is needed.
    static let api: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return decoder
    }()
}

extension JSONEncoder {
    /// Encoder for the Intrada REST API.
    ///
    /// Uses `.convertToSnakeCase` to match Rust's default serde field naming.
    static let api: JSONEncoder = {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        return encoder
    }()
}

// MARK: - Enum Codable (matching Rust serde formats)

// ItemKind — Rust: #[serde(rename_all = "lowercase")]
// JSON: "piece", "exercise"
extension ItemKind: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let raw = try container.decode(String.self)
        switch raw {
        case "piece": self = .piece
        case "exercise": self = .exercise
        default:
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Unknown ItemKind: \(raw)"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .piece: try container.encode("piece")
        case .exercise: try container.encode("exercise")
        }
    }
}

// GoalStatus — Rust: #[serde(rename_all = "snake_case")]
// JSON: "active", "completed", "archived"
extension GoalStatus: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let raw = try container.decode(String.self)
        switch raw {
        case "active": self = .active
        case "completed": self = .completed
        case "archived": self = .archived
        default:
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Unknown GoalStatus: \(raw)"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .active: try container.encode("active")
        case .completed: try container.encode("completed")
        case .archived: try container.encode("archived")
        }
    }
}

// GoalKind — Rust: #[serde(rename_all = "snake_case", tag = "type")]
// JSON (internally tagged): {"type": "session_frequency", "target_days_per_week": 5}
extension GoalKind: Codable {
    private enum CodingKeys: String, CodingKey {
        case type
        case targetDaysPerWeek
        case targetMinutesPerWeek
        case itemId
        case targetScore
        case description
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        switch type {
        case "session_frequency":
            let v = try container.decode(UInt8.self, forKey: .targetDaysPerWeek)
            self = .sessionFrequency(targetDaysPerWeek: v)
        case "practice_time":
            let v = try container.decode(UInt32.self, forKey: .targetMinutesPerWeek)
            self = .practiceTime(targetMinutesPerWeek: v)
        case "item_mastery":
            let id = try container.decode(String.self, forKey: .itemId)
            let score = try container.decode(UInt8.self, forKey: .targetScore)
            self = .itemMastery(itemId: id, targetScore: score)
        case "milestone":
            let desc = try container.decode(String.self, forKey: .description)
            self = .milestone(description: desc)
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .type,
                in: container,
                debugDescription: "Unknown GoalKind type: \(type)"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .sessionFrequency(let targetDaysPerWeek):
            try container.encode("session_frequency", forKey: .type)
            try container.encode(targetDaysPerWeek, forKey: .targetDaysPerWeek)
        case .practiceTime(let targetMinutesPerWeek):
            try container.encode("practice_time", forKey: .type)
            try container.encode(targetMinutesPerWeek, forKey: .targetMinutesPerWeek)
        case .itemMastery(let itemId, let targetScore):
            try container.encode("item_mastery", forKey: .type)
            try container.encode(itemId, forKey: .itemId)
            try container.encode(targetScore, forKey: .targetScore)
        case .milestone(let description):
            try container.encode("milestone", forKey: .type)
            try container.encode(description, forKey: .description)
        }
    }
}

// CompletionStatus — Rust: no rename (default externally tagged)
// JSON: "Completed", "EndedEarly"
extension CompletionStatus: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let raw = try container.decode(String.self)
        switch raw {
        case "Completed": self = .completed
        case "EndedEarly": self = .endedEarly
        default:
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Unknown CompletionStatus: \(raw)"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .completed: try container.encode("Completed")
        case .endedEarly: try container.encode("EndedEarly")
        }
    }
}

// EntryStatus — Rust: no rename (default externally tagged)
// JSON: "Completed", "Skipped", "NotAttempted"
extension EntryStatus: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let raw = try container.decode(String.self)
        switch raw {
        case "Completed": self = .completed
        case "Skipped": self = .skipped
        case "NotAttempted": self = .notAttempted
        default:
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Unknown EntryStatus: \(raw)"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .completed: try container.encode("Completed")
        case .skipped: try container.encode("Skipped")
        case .notAttempted: try container.encode("NotAttempted")
        }
    }
}

// RepAction — Rust: #[serde_repr] #[repr(i8)]
// JSON: -1 (Missed), 1 (Success) — serialised as integers
extension RepAction: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let raw = try container.decode(Int8.self)
        switch raw {
        case -1: self = .missed
        case 1: self = .success
        default:
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Unknown RepAction: \(raw)"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .missed: try container.encode(Int8(-1))
        case .success: try container.encode(Int8(1))
        }
    }
}

// MARK: - Struct Codable
//
// Auto-synthesised Codable for API-facing structs is appended to the generated
// SharedTypes.swift by build-ios.sh. Swift 6 requires the conformance to be in
// the same file as the struct definition for auto-synthesis to work.
