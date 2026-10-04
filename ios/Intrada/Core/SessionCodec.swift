import Foundation
import GRDB
import IntradaCoreFFI
import SharedTypes

extension LibraryStore {
  // ── Row ↔ PracticeSession codec, read and written by the core (#2234) ──

  struct UnreadableStoredValue: Error, CustomStringConvertible {
    let description: String
  }

  /// `nil` for a row the core refuses, reported and skipped.
  static func session(from row: Row) -> PracticeSession? {
    // The intention and the three reflection columns stay in the table unread:
    // nothing can set them and the core no longer carries them (#1766, #1374).
    let stored = StoredSessionRow(
      id: row["id"], startedAt: row["started_at"], completedAt: row["completed_at"],
      totalDurationSecs: row["total_duration_secs"], completionStatus: row["completion_status"],
      sessionNotes: row["session_notes"], entries: row["entries"],
      sessionScore: row["session_score"], captureVersion: row["capture_version"])
    do {
      let read = try sessionFromStored(row: stored)
      for value in read.unreadable {
        report(UnreadableStoredValue(description: value), decodeContext)
      }
      return try PracticeSession.bincodeDeserialize(input: [UInt8](read.session))
    } catch {
      report(error, decodeContext)
      return nil
    }
  }

  static func stored(_ session: PracticeSession) throws -> StoredSessionRow {
    try sessionToStored(session: Data(try session.bincodeSerialize()))
  }
}
