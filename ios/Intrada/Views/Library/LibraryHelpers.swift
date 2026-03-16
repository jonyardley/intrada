import SwiftUI

// MARK: - FilterTab

/// Filter tabs for the library list: All, Pieces, or Exercises.
enum FilterTab: String, CaseIterable, Identifiable {
    case all = "All"
    case pieces = "Pieces"
    case exercises = "Exercises"

    var id: String { rawValue }

    /// Maps to the Crux `ItemKind` for query filtering. Nil means "show all".
    var itemKind: ItemKind? {
        switch self {
        case .all: nil
        case .pieces: .piece
        case .exercises: .exercise
        }
    }
}

// MARK: - LibraryFormValidator

/// Client-side validation mirroring intrada-core/src/validation.rs constants.
/// Provides instant feedback before dispatching events to the Crux core.
struct LibraryFormValidator {
    static let maxTitle = 500
    static let maxComposer = 200
    static let maxNotes = 5000
    static let maxTag = 100
    static let maxTempoMarking = 100
    static let minBpm = 1
    static let maxBpm = 400

    /// Validates library item form fields. Returns a dictionary of field-keyed error messages.
    /// An empty dictionary means the form is valid.
    static func validate(
        kind: ItemKind,
        title: String,
        composer: String,
        key: String,
        tempoMarking: String,
        bpm: String,
        notes: String,
        tags: [String]
    ) -> [String: String] {
        var errors: [String: String] = [:]

        // Title: required, max length
        let trimmedTitle = title.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmedTitle.isEmpty {
            errors["title"] = "Title is required"
        } else if trimmedTitle.count > maxTitle {
            errors["title"] = "Title must be \(maxTitle) characters or less"
        }

        // Composer: required for pieces, max length
        let trimmedComposer = composer.trimmingCharacters(in: .whitespacesAndNewlines)
        if case .piece = kind, trimmedComposer.isEmpty {
            errors["composer"] = "Composer is required for pieces"
        } else if !trimmedComposer.isEmpty && trimmedComposer.count > maxComposer {
            errors["composer"] = "Composer must be \(maxComposer) characters or less"
        }

        // Tempo marking: max length
        let trimmedMarking = tempoMarking.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmedMarking.isEmpty && trimmedMarking.count > maxTempoMarking {
            errors["tempoMarking"] = "Tempo marking must be \(maxTempoMarking) characters or less"
        }

        // BPM: numeric, range check
        let trimmedBpm = bpm.trimmingCharacters(in: .whitespacesAndNewlines)
        if !trimmedBpm.isEmpty {
            if let bpmValue = Int(trimmedBpm) {
                if bpmValue < minBpm || bpmValue > maxBpm {
                    errors["bpm"] = "BPM must be between \(minBpm) and \(maxBpm)"
                }
            } else {
                errors["bpm"] = "BPM must be a number"
            }
        }

        // Notes: max length
        if notes.count > maxNotes {
            errors["notes"] = "Notes must be \(maxNotes) characters or less"
        }

        // Tags: each tag max length
        for tag in tags {
            let trimmedTag = tag.trimmingCharacters(in: .whitespacesAndNewlines)
            if trimmedTag.isEmpty {
                errors["tags"] = "Tags cannot be empty"
                break
            }
            if trimmedTag.count > maxTag {
                errors["tags"] = "Each tag must be \(maxTag) characters or less"
                break
            }
        }

        return errors
    }
}

// MARK: - Library Data Helpers

/// Extracts unique composers from the library items for autocomplete.
func uniqueComposers(from items: [LibraryItemView]) -> [String] {
    let composers: [String] = items
        .map { (item: LibraryItemView) -> String in item.subtitle }
        .filter { (s: String) -> Bool in !s.isEmpty }
    return Array(Set(composers)).sorted()
}

/// Extracts unique tags from the library items for autocomplete.
func uniqueTags(from items: [LibraryItemView]) -> [String] {
    let allTags: [String] = items.flatMap { (item: LibraryItemView) -> [String] in item.tags }
    return Array(Set(allTags)).sorted()
}

// MARK: - Date Formatting

/// Formats an RFC3339 date string for display (e.g. "12 Jan 2026").
func formatDate(_ rfc3339: String) -> String {
    let formatter = ISO8601DateFormatter()
    formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    guard let date = formatter.date(from: rfc3339) else {
        // Try without fractional seconds
        formatter.formatOptions = [.withInternetDateTime]
        guard let date = formatter.date(from: rfc3339) else {
            return rfc3339
        }
        return displayFormatter.string(from: date)
    }
    return displayFormatter.string(from: date)
}

private let displayFormatter: DateFormatter = {
    let f = DateFormatter()
    f.dateFormat = "d MMM yyyy"
    return f
}()

/// Parses a formatted tempo string back into marking and BPM components.
/// Handles: "Allegro (132 BPM)", "132 BPM", "Allegro"
func parseTempoDisplay(_ tempo: String?) -> (marking: String, bpm: String) {
    guard let tempo, !tempo.isEmpty else {
        return ("", "")
    }

    // Pattern: "Marking (### BPM)"
    if let parenRange = tempo.range(of: " ("),
       let bpmRange = tempo.range(of: " BPM)") {
        let marking = String(tempo[tempo.startIndex..<parenRange.lowerBound])
        let bpmStart = tempo.index(parenRange.upperBound, offsetBy: 0)
        let bpmStr = String(tempo[bpmStart..<bpmRange.lowerBound])
        return (marking, bpmStr)
    }

    // Pattern: "### BPM"
    if tempo.hasSuffix(" BPM") {
        let bpmStr = String(tempo.dropLast(4))
        // Could be "108 / 120" for combined tempo — take the target (last number)
        if bpmStr.contains(" / ") {
            let parts = bpmStr.components(separatedBy: " / ")
            return ("", parts.last ?? bpmStr)
        }
        return ("", bpmStr)
    }

    // Just marking text
    return (tempo, "")
}
