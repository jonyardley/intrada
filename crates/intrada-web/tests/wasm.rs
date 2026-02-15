use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Clear all localStorage keys to ensure test isolation.
fn clear_local_storage() {
    web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .clear()
        .unwrap();
}

/// Read raw JSON from a localStorage key.
fn read_storage_key(key: &str) -> Option<String> {
    web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .get_item(key)
        .unwrap()
}

// Session-in-progress round-trip (crash recovery via localStorage — FR-008)
#[wasm_bindgen_test]
fn test_session_in_progress_round_trip() {
    use intrada_core::{ActiveSession, EntryStatus, SetlistEntry};
    use intrada_web::core_bridge::{load_session_in_progress, SESSION_IN_PROGRESS_KEY};

    clear_local_storage();

    // Verify empty initially
    assert!(read_storage_key(SESSION_IN_PROGRESS_KEY).is_none());
    assert!(load_session_in_progress().is_none());

    // Write a session-in-progress via localStorage directly
    let now = chrono::Utc::now();
    let session = ActiveSession {
        id: "test-session-1".to_string(),
        entries: vec![SetlistEntry {
            id: "e1".to_string(),
            item_id: "p1".to_string(),
            item_title: "Test Piece".to_string(),
            item_type: "piece".to_string(),
            position: 0,
            duration_secs: 0,
            status: EntryStatus::NotAttempted,
            notes: None,
        }],
        current_index: 0,
        session_started_at: now,
        current_item_started_at: now,
    };

    let json = serde_json::to_string(&session).unwrap();
    web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .set_item(SESSION_IN_PROGRESS_KEY, &json)
        .unwrap();

    // Read back via the public API
    let recovered = load_session_in_progress();
    assert!(recovered.is_some(), "Expected session-in-progress to load");
    let recovered = recovered.unwrap();
    assert_eq!(recovered.entries.len(), 1);
    assert_eq!(recovered.entries[0].item_title, "Test Piece");
    assert_eq!(recovered.current_index, 0);
}

// Empty localStorage returns None for session-in-progress
#[wasm_bindgen_test]
fn test_empty_local_storage_no_session_in_progress() {
    use intrada_web::core_bridge::load_session_in_progress;

    clear_local_storage();

    let result = load_session_in_progress();
    assert!(
        result.is_none(),
        "Expected no session-in-progress on empty localStorage"
    );
}
