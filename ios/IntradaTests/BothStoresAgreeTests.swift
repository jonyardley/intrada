import Foundation
import GRDB
import SharedTypes
import Testing

@testable import Intrada

/// While GRDB and the shared store both exist (#2432): the same file, written
/// at an earlier migration, must read the same through each, and a file the
/// shared store wrote must still read in a GRDB build, so a revert is safe.
@MainActor
struct BothStoresAgreeTests {
  struct Case: Sendable, CustomTestStringConvertible {
    let version: String
    let seed: String
    var testDescription: String { version }
  }

  nonisolated static let early = "2026-01-01T00:00:00Z"

  nonisolated static func v3Session(_ id: String, _ entries: String) -> String {
    """
    INSERT INTO session (id, started_at, completed_at, total_duration_secs,
      completion_status, session_notes, session_intention, entries, updated_at, deleted_at)
    VALUES ('\(id)', '2026-01-01T00:00:00Z', '2026-01-01T00:01:00Z', 60, 'completed', NULL,
      NULL, '\(entries)', '2026-01-01T00:00:00Z', NULL);
    """
  }

  nonisolated static let cases: [Case] = [
    Case(
      version: "v1_item",
      seed: """
        INSERT INTO item (id, title, kind, composer, key, tempo_marking, tempo_bpm, notes, tags,
          created_at, updated_at)
        VALUES ('p1', 'Waltz', 'piece', 'Chopin', 'Ab major', 'Lento', 60, 'rubato',
          '["waltz"]', '\(early)', '\(early)');
        INSERT INTO item (id, title, kind, tags, created_at, updated_at, deleted_at)
        VALUES ('gone', 'Gone', 'exercise', '[]', '\(early)', '\(early)', '\(early)');
        """),
    Case(
      version: "v4_session_score",
      seed: [
        v3Session(
          "good", #"[{"id":"e1","score":2},{"id":"e2","score":4},{"id":"e3","score":7}]"#),
        v3Session(
          "kept",
          #"[{"id":"e1","itemId":"i1","itemTitle":"Bach","itemType":"piece","position":0,"durationSecs":120,"status":"completed","score":5,"notes":"keep"},{"id":"e2","itemId":"i2","itemTitle":"Scales","itemType":"exercise","position":1,"durationSecs":60,"status":"completed"}]"#
        ),
        v3Session("legacy", #"[{"id":"e1","score":3,"legacyKey":"x"}]"#),
        v3Session("bad", "not json"),
        v3Session("object", #"{"a":{"score":3}}"#),
      ].joined()),
    Case(
      version: "v16_item_photo",
      seed: """
        INSERT INTO item (id, title, kind, key, modality, tags, linked_exercise_ids,
          created_at, updated_at, priority, chord_chart)
        VALUES ('p1', 'Waltz', 'piece', 'G', 'minor', '[]', '[]', '\(early)', '\(early)', 0,
          '{"key":"G","modality":"minor","metre":3,"sections":[]}');
        INSERT INTO item (id, title, kind, tags, created_at, updated_at)
        VALUES ('p2', 'Plain', 'piece', '[]', '\(early)', '\(early)');
        """),
    Case(
      version: "v18_section",
      seed: """
        INSERT INTO item (id, title, kind, key, modality, tags, created_at, updated_at,
          chord_chart)
        VALUES ('p1', 'Nocturne', 'piece', 'Eb', 'major', '[]', '\(early)', '\(early)',
          '{"key":"G","modality":"minor","sections":[]}');
        INSERT INTO item (id, title, kind, key, modality, tags, created_at, updated_at)
        VALUES ('p2', 'Freeform', 'piece', 'F# major', NULL, '[]', '\(early)', '\(early)');
        INSERT INTO item (id, title, kind, key, modality, tags, created_at, updated_at)
        VALUES ('p3', 'Odd', 'piece', 'H dorian', NULL, '[]', '\(early)', '\(early)');
        INSERT INTO session (id, started_at, completed_at, total_duration_secs,
          completion_status, session_notes, session_intention, entries, updated_at,
          deleted_at)
        VALUES ('s1', '2026-09-01T10:00:00Z', '2026-09-01T10:05:00Z', 300, 'completed',
          NULL, NULL, '[{"id":"e1","itemId":"p1","itemTitle":"Nocturne","itemType":"piece","position":0,"durationSecs":300,"status":"completed","plannedVariationId":"v-old","plays":[{"id":"pl1","variationId":"v-old","startedAt":"2026-09-01T10:00:00Z","seconds":300,"repHistory":[{"action":"success","at":"2026-09-01T10:01:00Z"}],"score":7}]}]',
          '2026-09-01T10:05:00Z', NULL);
        """),
    Case(
      version: "v19_keys_and_variations",
      seed: [
        ("p1", "piece", #"["b", "a", "b"]"#), ("p2", "piece", "{"),
        ("p3", "piece", #"{"x": "y"}"#), ("p4", "piece", #"[1, "a"]"#),
        ("x-y", "piece", #"["z"]"#), ("x", "piece", #"["y-z"]"#),
        ("a", "exercise", "[]"), ("b", "exercise", "[]"),
      ].map { id, kind, links in
        """
        INSERT INTO item (id, title, kind, tags, linked_exercise_ids, created_at, updated_at)
        VALUES ('\(id)', '\(id)', '\(kind)', '[]', '\(links)', '\(early)',
          '2026-01-02T00:00:00Z');
        """
      }.joined()),
  ]

  struct Notebook: Equatable {
    let items: [Item]
    let sessions: [PracticeSession]
    let variations: [Variation]
  }

  private static func file() -> String {
    FileManager.default.temporaryDirectory
      .appendingPathComponent("both-stores-\(UUID().uuidString).sqlite").path
  }

  private static func read(_ store: LibraryStore) throws -> Notebook {
    Notebook(
      items: try store.loadItems(), sessions: try store.loadSessions(),
      variations: try store.loadVariations())
  }

  private static func read(_ store: SharedItemStore) throws -> Notebook {
    guard case .items(let items) = try store.handle(.loadItems),
      case .sessions(let sessions) = try store.handle(.loadSessions),
      case .variations(let variations) = try store.handle(.loadVariations)
    else { throw SharedItemStore.StoreFailure(description: "a load answered the wrong shape") }
    return Notebook(items: items, sessions: sessions, variations: variations)
  }

  @Test(arguments: cases)
  func anEarlierFileReadsTheSameThroughBoth(_ c: Case) throws {
    let grdbPath = Self.file()
    let rustPath = Self.file()
    defer {
      try? FileManager.default.removeItem(atPath: grdbPath)
      try? FileManager.default.removeItem(atPath: rustPath)
    }
    do {
      let queue = try DatabaseQueue(path: grdbPath)
      try LibraryStore.migrator.migrate(queue, upTo: c.version)
      try queue.write { db in try db.execute(sql: c.seed) }
      try queue.close()
    }
    try FileManager.default.copyItem(atPath: grdbPath, toPath: rustPath)

    let viaGRDB = try Self.read(LibraryStore(DatabaseQueue(path: grdbPath)))
    let viaRust = try Self.read(SharedItemStore.at(path: rustPath))

    #expect(!viaGRDB.items.isEmpty || !viaGRDB.sessions.isEmpty)
    #expect(viaRust == viaGRDB)
  }

  @Test
  func aCurrentFileGRDBWroteReadsTheSameThroughBoth() throws {
    let grdbPath = Self.file()
    let rustPath = Self.file()
    defer {
      try? FileManager.default.removeItem(atPath: grdbPath)
      try? FileManager.default.removeItem(atPath: rustPath)
    }
    do {
      let queue = try DatabaseQueue(path: grdbPath)
      let store = try LibraryStore(queue)
      try Self.writeEveryKind(store)
      try queue.close()
    }
    try FileManager.default.copyItem(atPath: grdbPath, toPath: rustPath)

    let viaGRDB = try Self.read(LibraryStore(DatabaseQueue(path: grdbPath)))
    let viaRust = try Self.read(SharedItemStore.at(path: rustPath))

    #expect(viaGRDB.items.count == 2)
    #expect(viaRust == viaGRDB)
  }

  @Test
  func aFileTheSharedStoreWroteReadsInAGRDBBuild() throws {
    let path = Self.file()
    defer { try? FileManager.default.removeItem(atPath: path) }
    let viaRust: Notebook
    do {
      let store = try SharedItemStore.at(path: path)
      _ = try store.handle(
        .saveItems([
          SharedStoreFixtureTests.piece, SharedStoreFixtureTests.exercise,
          SharedStoreFixtureTests.deleted,
        ]))
      _ = try store.handle(
        .deleteItem(id: SharedStoreFixtureTests.deleted.id, deletedAt: "2026-09-05T08:00:00Z"))
      _ = try store.handle(.saveVariations(SharedStoreFixtureTests.variations))
      _ = try store.handle(.saveSession(SharedStoreFixtureTests.session))
      viaRust = try Self.read(store)
    }

    let viaGRDB = try Self.read(LibraryStore(DatabaseQueue(path: path)))

    #expect(viaRust.items.count == 2)
    #expect(viaGRDB == viaRust)
  }

  private static func writeEveryKind(_ store: LibraryStore) throws {
    try store.save([
      SharedStoreFixtureTests.piece, SharedStoreFixtureTests.exercise,
      SharedStoreFixtureTests.deleted,
    ])
    try store.delete(id: SharedStoreFixtureTests.deleted.id, deletedAt: "2026-09-05T08:00:00Z")
    try store.save(SharedStoreFixtureTests.variations)
    try store.saveSession(SharedStoreFixtureTests.session)
  }
}
