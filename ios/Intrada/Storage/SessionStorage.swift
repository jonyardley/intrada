// SessionStorage.swift
//
// UserDefaults wrapper for active session crash recovery.
// Equivalent of localStorage key `intrada:session-in-progress` in the web shell.

import Foundation

/// Persists an in-progress practice session to UserDefaults for crash recovery.
///
/// When the app restarts, the shell checks for a stored session and restores it
/// via `Event.session(.restoreSession(...))` if one is found.
enum SessionStorage {

    private static let key = "intrada:session-in-progress"

    /// Save an active session for crash recovery.
    static func save(_ session: ActiveSession) {
        do {
            let data = try JSONEncoder.intrada.encode(session)
            UserDefaults.standard.set(data, forKey: key)
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
            return try JSONDecoder.intrada.decode(ActiveSession.self, from: data)
        } catch {
            print("[SessionStorage] Failed to load session: \(error)")
            return nil
        }
    }

    /// Clear the stored session (called after successful save or discard).
    static func clear() {
        UserDefaults.standard.removeObject(forKey: key)
    }
}
