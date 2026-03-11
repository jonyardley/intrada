// SessionStorage.swift
//
// UserDefaults wrapper for active session crash recovery.
// Equivalent of localStorage key `intrada:session-in-progress` in the web shell.
//
// Uses BCS (bincode) serialization to match the CoreFfi bridge format.

import Foundation

/// Persists an in-progress practice session to UserDefaults for crash recovery.
///
/// When the app restarts, the shell checks for a stored session and restores it
/// via `Event.session(.recoverSession(...))` if one is found.
enum SessionStorage {

    private static let key = "intrada:session-in-progress"

    /// Save an active session for crash recovery.
    static func save(_ session: ActiveSession) {
        do {
            let bytes = try session.bincodeSerialize()
            UserDefaults.standard.set(Data(bytes), forKey: key)
        } catch {
            print("[SessionStorage] Failed to save session: \(error)")
        }
    }

    /// Load a previously saved active session, if any.
    static func load() -> ActiveSession? {
        guard let data = UserDefaults.standard.data(forKey: key) else {
            return nil
        }
        do {
            return try ActiveSession.bincodeDeserialize(input: [UInt8](data))
        } catch {
            // BCS deserialization failed — likely a legacy JSON-encoded session
            // from before the CoreFfi migration. Clear it and move on.
            print("[SessionStorage] Failed to load session (clearing stale data): \(error)")
            clear()
            return nil
        }
    }

    /// Clear the stored session (called after successful save or discard).
    static func clear() {
        UserDefaults.standard.removeObject(forKey: key)
    }
}
