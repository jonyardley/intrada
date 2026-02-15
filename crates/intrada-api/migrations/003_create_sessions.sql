CREATE TABLE IF NOT EXISTS practice_sessions (
    id TEXT PRIMARY KEY,
    session_notes TEXT,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL,
    total_duration_secs BIGINT NOT NULL CHECK (total_duration_secs >= 0),
    completion_status TEXT NOT NULL CHECK (completion_status IN ('completed', 'ended_early'))
);

CREATE TABLE IF NOT EXISTS setlist_entries (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES practice_sessions(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL,
    item_title TEXT NOT NULL,
    item_type TEXT NOT NULL CHECK (item_type IN ('piece', 'exercise')),
    position INTEGER NOT NULL CHECK (position >= 0),
    duration_secs BIGINT NOT NULL CHECK (duration_secs >= 0),
    status TEXT NOT NULL CHECK (status IN ('completed', 'skipped', 'not_attempted')),
    notes TEXT
);

CREATE INDEX idx_setlist_entries_session_id ON setlist_entries(session_id);
