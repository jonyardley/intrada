use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Clear all localStorage keys to ensure test isolation (FR-009).
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

// T012: Library data round-trip
#[wasm_bindgen_test]
fn test_library_data_round_trip() {
    use intrada_core::{Exercise, LibraryData, Piece, Tempo};
    use intrada_web::core_bridge::{save_to_local_storage, STORAGE_KEY};

    clear_local_storage();

    let now = chrono::Utc::now();
    let data = LibraryData {
        pieces: vec![Piece {
            id: "p1".to_string(),
            title: "Test Piece".to_string(),
            composer: "Test Composer".to_string(),
            key: Some("C Major".to_string()),
            tempo: Some(Tempo {
                marking: Some("Allegro".to_string()),
                bpm: Some(120),
            }),
            notes: Some("Test notes".to_string()),
            tags: vec!["tag1".to_string()],
            created_at: now,
            updated_at: now,
        }],
        exercises: vec![Exercise {
            id: "e1".to_string(),
            title: "Test Exercise".to_string(),
            composer: None,
            category: Some("Scales".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        }],
    };

    // Write to localStorage
    save_to_local_storage(&data);

    // Verify data was written
    let stored = read_storage_key(STORAGE_KEY);
    assert!(stored.is_some(), "Expected data in localStorage");

    // Read back via the public API
    let (pieces, exercises) = intrada_web::core_bridge::load_library_data();
    assert_eq!(pieces.len(), 1, "Expected 1 piece");
    assert_eq!(exercises.len(), 1, "Expected 1 exercise");
    assert_eq!(pieces[0].title, "Test Piece");
    assert_eq!(pieces[0].composer, "Test Composer");
    assert_eq!(exercises[0].title, "Test Exercise");
    assert_eq!(exercises[0].category, Some("Scales".to_string()));
}

// T013: Session data round-trip
#[wasm_bindgen_test]
fn test_session_data_round_trip() {
    use intrada_core::SessionsData;
    use intrada_web::core_bridge::{save_sessions_to_local_storage, SESSIONS_KEY};

    clear_local_storage();

    let now = chrono::Utc::now();
    let data = SessionsData {
        sessions: vec![intrada_core::Session {
            id: "s1".to_string(),
            item_id: "p1".to_string(),
            duration_minutes: 30,
            started_at: now,
            logged_at: now,
            notes: Some("Great practice".to_string()),
        }],
    };

    // Write to localStorage
    save_sessions_to_local_storage(&data);

    // Verify data was written
    let stored = read_storage_key(SESSIONS_KEY);
    assert!(stored.is_some(), "Expected session data in localStorage");

    // Read back via the public API
    let sessions = intrada_web::core_bridge::load_sessions_data();
    assert_eq!(sessions.len(), 1, "Expected 1 session");
    assert_eq!(sessions[0].duration_minutes, 30);
    assert_eq!(sessions[0].notes, Some("Great practice".to_string()));
}

// T014: Empty localStorage seeds stub data
#[wasm_bindgen_test]
fn test_empty_local_storage_seeds_stub_data() {
    use intrada_web::core_bridge::STORAGE_KEY;

    clear_local_storage();

    // Verify localStorage is empty
    assert!(read_storage_key(STORAGE_KEY).is_none());

    // load_library_data should seed stub data when localStorage is empty
    let (pieces, exercises) = intrada_web::core_bridge::load_library_data();
    assert!(!pieces.is_empty(), "Expected stub pieces to be seeded");
    assert!(
        !exercises.is_empty(),
        "Expected stub exercises to be seeded"
    );

    // Verify localStorage now contains data
    let stored = read_storage_key(STORAGE_KEY);
    assert!(
        stored.is_some(),
        "Expected localStorage to contain seeded data"
    );
}
