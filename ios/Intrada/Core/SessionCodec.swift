import Foundation
import GRDB
import SharedTypes

extension LibraryStore {
  // ── Row ↔ PracticeSession codec ──────────────────────────────────────

  static func session(from row: Row) -> PracticeSession {
    let score: Int64? = row["session_score"]
    // The intention and the three reflection columns stay in the table unread:
    // nothing can set them and the core no longer carries them (#1766, #1374).
    return PracticeSession(
      id: row["id"], entries: decodeEntries(row["entries"], sessionStartedAt: row["started_at"]),
      sessionNotes: row["session_notes"],
      startedAt: row["started_at"], completedAt: row["completed_at"],
      totalDurationSecs: UInt64(row["total_duration_secs"] as Int64),
      completionStatus: completionStatuses.decode(row["completion_status"]) ?? .completed,
      sessionScore: score.map { UInt8(clamping: $0) })
  }

  // Entries (a nested, optional-heavy aggregate) go to JSON via a Codable DTO,
  // not bincode: bincode is positional, so a future field change would fail to
  // decode old rows, unacceptable when the device is the only copy.
  /// The per-play fields below `plannedDurationSecs` are LEGACY: every row
  /// written before #1739 carries them at entry level and has no `plays`, and
  /// `decodeEntries` folds them into one play. Nothing writes them any more,
  /// and nothing on device is rewritten (#1739 decision 8).
  struct StoredEntry: Codable {
    var id: String
    var itemId: String
    var itemTitle: String
    var itemType: String
    var position: UInt64
    var durationSecs: UInt64
    var status: String
    var notes: String?
    var intention: String?
    var plannedDurationSecs: UInt32?
    var groupId: String?
    var plannedVariationId: String?
    var plannedRepTarget: UInt8?
    var plays: [StoredPlay]?

    var score: UInt8?
    var repTarget: UInt8?
    var repCount: UInt8?
    var repTargetReached: Bool?
    var repHistory: [StoredRepEvent]?
    var achievedTempo: UInt16?
    var variantId: String?
    var clickPattern: StoredClickState?
  }

  struct StoredPlay: Codable {
    var id: String
    var variationId: String?
    var startedAt: String
    var seconds: UInt64
    var repTarget: UInt8?
    var repCount: UInt8?
    var repTargetReached: Bool?
    var repHistory: [StoredRepEvent]?
    var achievedTempo: UInt16?
    var clickPattern: StoredClickState?
    var score: UInt8?
  }

  struct StoredClickState: Codable {
    var metre: StoredMetre
    var sounding: UInt16
  }

  static func storedClick(_ state: ClickState?) -> StoredClickState? {
    state.map {
      StoredClickState(
        metre: StoredMetre(beats: $0.metre.beats, unit: $0.metre.unit, groups: $0.metre.groups),
        sounding: $0.sounding)
    }
  }

  static func clickState(_ stored: StoredClickState?) -> ClickState? {
    stored.map {
      ClickState(
        metre: Metre(beats: $0.metre.beats, unit: $0.metre.unit, groups: $0.metre.groups),
        sounding: $0.sounding)
    }
  }

  /// Rows written before the history was timestamped hold bare action strings;
  /// those decode with no `at`, and the session start stands in for it.
  struct StoredRepEvent: Codable {
    var action: String
    var at: String?

    init(action: String, at: String?) {
      self.action = action
      self.at = at
    }

    init(from decoder: Decoder) throws {
      if let legacy = try? decoder.singleValueContainer().decode(String.self) {
        action = legacy
        at = nil
        return
      }
      let keyed = try decoder.container(keyedBy: CodingKeys.self)
      action = try keyed.decode(String.self, forKey: .action)
      at = try keyed.decodeIfPresent(String.self, forKey: .at)
    }
  }

  static func storedPlay(_ p: VariationPlay) -> StoredPlay {
    StoredPlay(
      id: p.id, variationId: p.variationId, startedAt: p.startedAt, seconds: p.seconds,
      repTarget: p.repTarget, repCount: p.repCount, repTargetReached: p.repTargetReached,
      repHistory: p.repHistory.map {
        $0.map { StoredRepEvent(action: repActions.encode($0.action), at: $0.at) }
      },
      achievedTempo: p.achievedTempo, clickPattern: storedClick(p.clickPattern), score: p.score)
  }

  static func encodeEntries(_ entries: [SetlistEntry]) throws -> String {
    let dtos = entries.map { e in
      StoredEntry(
        id: e.id, itemId: e.itemId, itemTitle: e.itemTitle, itemType: itemKinds.encode(e.itemType),
        position: e.position, durationSecs: e.durationSecs, status: entryStatuses.encode(e.status),
        notes: e.notes, intention: e.intention, plannedDurationSecs: e.plannedDurationSecs,
        groupId: e.groupId, plannedVariationId: e.plannedVariationId,
        plannedRepTarget: e.plannedRepTarget, plays: e.plays.map(storedPlay))
    }
    return try encodeJSON(dtos)
  }

  /// A row written before #1739 has no `plays` and carries one score, one rep
  /// count and one tempo on the entry itself. It folds into a single play so
  /// the record survives; nothing on device is rewritten (#1739 decision 8).
  /// An entry that was skipped or never reached keeps no play, which is what a
  /// zero-play entry means.
  static func plays(from d: StoredEntry, sessionStartedAt: String) -> [VariationPlay] {
    if let stored = d.plays, !stored.isEmpty {
      return stored.map { p in
        VariationPlay(
          id: p.id, variationId: p.variationId, startedAt: p.startedAt, seconds: p.seconds,
          repTarget: p.repTarget, repCount: p.repCount, repTargetReached: p.repTargetReached,
          repHistory: p.repHistory.map {
            $0.map { RepEvent(action: repAction(from: $0.action), at: $0.at ?? sessionStartedAt) }
          },
          achievedTempo: p.achievedTempo, clickPattern: clickState(p.clickPattern), score: p.score)
      }
    }

    // A mark or a banked repetition is a record of practice whatever the status
    // says: rows written before #1739 froze rep state on a skip, and the core
    // still keeps that play, so the fold must not lose it on the way in.
    let recorded = d.score != nil || (d.repCount ?? 0) > 0
    guard entryStatus(from: d.status) == .completed || recorded else { return [] }

    return [
      VariationPlay(
        id: "\(d.id)-play", variationId: d.variantId, startedAt: sessionStartedAt,
        seconds: d.durationSecs, repTarget: d.repTarget, repCount: d.repCount,
        repTargetReached: d.repTargetReached,
        repHistory: d.repHistory.map {
          $0.map { RepEvent(action: repAction(from: $0.action), at: $0.at ?? sessionStartedAt) }
        },
        achievedTempo: d.achievedTempo, clickPattern: clickState(d.clickPattern), score: d.score)
    ]
  }

  static func decodeEntries(_ json: String, sessionStartedAt: String) -> [SetlistEntry] {
    guard let dtos = decodeJSON([StoredEntry].self, from: json, field: "entries") else {
      return []
    }
    return dtos.map { d in
      SetlistEntry(
        id: d.id, itemId: d.itemId, itemTitle: d.itemTitle, itemType: kind(from: d.itemType),
        position: d.position, durationSecs: d.durationSecs, status: entryStatus(from: d.status),
        notes: d.notes, intention: d.intention, plannedDurationSecs: d.plannedDurationSecs,
        groupId: d.groupId, plannedVariationId: d.plannedVariationId ?? d.variantId,
        plannedRepTarget: d.plannedRepTarget ?? d.repTarget,
        plays: plays(from: d, sessionStartedAt: sessionStartedAt))
    }
  }

  static let completionStatuses = StoredEnum<CompletionStatus>(
    kind: "CompletionStatus", cases: [.completed, .endedEarly]
  ) {
    switch $0 {
    case .completed: "completed"
    case .endedEarly: "ended_early"
    }
  }

  static let entryStatuses = StoredEnum<EntryStatus>(
    kind: "EntryStatus", cases: [.completed, .skipped, .notAttempted]
  ) {
    switch $0 {
    case .completed: "completed"
    case .skipped: "skipped"
    case .notAttempted: "not_attempted"
    }
  }

  static let repActions = StoredEnum<RepAction>(kind: "RepAction", cases: [.missed, .success]) {
    switch $0 {
    case .missed: "missed"
    case .success: "success"
    }
  }

  // Conservative: an unknown status or rep must not inflate stats (#949).
  static func entryStatus(from raw: String) -> EntryStatus {
    entryStatuses.decode(raw) ?? .notAttempted
  }

  static func repAction(from raw: String) -> RepAction {
    repActions.decode(raw) ?? .missed
  }
}
