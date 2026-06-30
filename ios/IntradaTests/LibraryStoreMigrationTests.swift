import GRDB
import XCTest

@testable import Intrada

final class LibraryStoreMigrationTests: XCTestCase {
  func testV5RescalesEntryScoresAndAddsSessionScore() throws {
    let queue = try DatabaseQueue()  // in-memory

    // Migrate only up to v3 (the pre-rescale schema), then seed a session
    // whose single entry scored 3 on the old 1–5 scale.
    try LibraryStore.migrator.migrate(queue, upTo: "v3_session")
    try queue.write { db in
      let entriesJSON =
        #"[{"id":"e1","itemId":"i1","itemTitle":"Scales","itemType":"exercise","position":0,"durationSecs":60,"status":"completed","score":3}]"#
      try db.execute(
        sql: """
          INSERT INTO session (id, started_at, completed_at, total_duration_secs,
            completion_status, session_notes, session_intention, entries, updated_at, deleted_at)
          VALUES ('s1','2026-01-01T00:00:00Z','2026-01-01T00:01:00Z',60,'completed',NULL,NULL,?, '2026-01-01T00:00:00Z',NULL)
          """, arguments: [entriesJSON])
    }

    // Run the remaining migrations (v4 add column, v5 rescale).
    try LibraryStore.migrator.migrate(queue)

    try queue.read { db in
      let row = try Row.fetchOne(db, sql: "SELECT entries, session_score FROM session WHERE id='s1'")!
      let entries: String = row["entries"]
      XCTAssertTrue(entries.contains("\"score\":6"), "old score 3 should rescale ×2 to 6")
      XCTAssertNil(row["session_score"] as Int64?, "session_score column exists, null for old rows")
    }
  }
}
