CREATE TABLE IF NOT EXISTS exercises (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    composer TEXT,
    category TEXT,
    key TEXT,
    tempo_marking TEXT,
    tempo_bpm SMALLINT CHECK (tempo_bpm IS NULL OR (tempo_bpm >= 1 AND tempo_bpm <= 400)),
    notes TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
