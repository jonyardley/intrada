use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use intrada_core::domain::piece::Piece;
use intrada_core::domain::exercise::Exercise;
use intrada_core::domain::types::Tempo;
use rusqlite::{params, Connection};

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .context("Could not determine local data directory")?
            .join("intrada");
        std::fs::create_dir_all(&data_dir)
            .with_context(|| format!("Could not create data directory: {}", data_dir.display()))?;

        let db_path = data_dir.join("library.db");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Could not open database: {}", db_path.display()))?;

        let store = Self { conn };
        store.initialize_schema()?;
        Ok(store)
    }

    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.initialize_schema()?;
        Ok(store)
    }

    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS pieces (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                composer TEXT NOT NULL,
                key TEXT,
                tempo_marking TEXT,
                tempo_bpm INTEGER,
                notes TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS exercises (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                composer TEXT,
                category TEXT,
                key TEXT,
                tempo_marking TEXT,
                tempo_bpm INTEGER,
                notes TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );"
        )?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<(Vec<Piece>, Vec<Exercise>)> {
        let pieces = self.load_pieces()?;
        let exercises = self.load_exercises()?;
        Ok((pieces, exercises))
    }

    fn load_pieces(&self) -> Result<Vec<Piece>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, composer, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at FROM pieces"
        )?;

        let pieces = stmt.query_map([], |row| {
            let tags_str: String = row.get(7)?;
            let created_str: String = row.get(8)?;
            let updated_str: String = row.get(9)?;
            let tempo_marking: Option<String> = row.get(4)?;
            let tempo_bpm: Option<u16> = row.get(5)?;

            Ok(PieceRow {
                id: row.get(0)?,
                title: row.get(1)?,
                composer: row.get(2)?,
                key: row.get(3)?,
                tempo_marking,
                tempo_bpm,
                notes: row.get(6)?,
                tags_str,
                created_str,
                updated_str,
            })
        })?;

        let mut result = Vec::new();
        for row in pieces {
            let row = row?;
            result.push(piece_from_row(row)?);
        }
        Ok(result)
    }

    fn load_exercises(&self) -> Result<Vec<Exercise>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, composer, category, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at FROM exercises"
        )?;

        let exercises = stmt.query_map([], |row| {
            let tags_str: String = row.get(8)?;
            let created_str: String = row.get(9)?;
            let updated_str: String = row.get(10)?;
            let tempo_marking: Option<String> = row.get(5)?;
            let tempo_bpm: Option<u16> = row.get(6)?;

            Ok(ExerciseRow {
                id: row.get(0)?,
                title: row.get(1)?,
                composer: row.get(2)?,
                category: row.get(3)?,
                key: row.get(4)?,
                tempo_marking,
                tempo_bpm,
                notes: row.get(7)?,
                tags_str,
                created_str,
                updated_str,
            })
        })?;

        let mut result = Vec::new();
        for row in exercises {
            let row = row?;
            result.push(exercise_from_row(row)?);
        }
        Ok(result)
    }

    pub fn save_piece(&self, piece: &Piece) -> Result<()> {
        let tags_json = serde_json::to_string(&piece.tags)?;
        let (tempo_marking, tempo_bpm) = split_tempo(&piece.tempo);

        self.conn.execute(
            "INSERT INTO pieces (id, title, composer, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                piece.id,
                piece.title,
                piece.composer,
                piece.key,
                tempo_marking,
                tempo_bpm,
                piece.notes,
                tags_json,
                piece.created_at.to_rfc3339(),
                piece.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn save_exercise(&self, exercise: &Exercise) -> Result<()> {
        let tags_json = serde_json::to_string(&exercise.tags)?;
        let (tempo_marking, tempo_bpm) = split_tempo(&exercise.tempo);

        self.conn.execute(
            "INSERT INTO exercises (id, title, composer, category, key, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                exercise.id,
                exercise.title,
                exercise.composer,
                exercise.category,
                exercise.key,
                tempo_marking,
                tempo_bpm,
                exercise.notes,
                tags_json,
                exercise.created_at.to_rfc3339(),
                exercise.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update_piece(&self, piece: &Piece) -> Result<()> {
        let tags_json = serde_json::to_string(&piece.tags)?;
        let (tempo_marking, tempo_bpm) = split_tempo(&piece.tempo);

        self.conn.execute(
            "UPDATE pieces SET title=?1, composer=?2, key=?3, tempo_marking=?4, tempo_bpm=?5, notes=?6, tags=?7, updated_at=?8 WHERE id=?9",
            params![
                piece.title,
                piece.composer,
                piece.key,
                tempo_marking,
                tempo_bpm,
                piece.notes,
                tags_json,
                piece.updated_at.to_rfc3339(),
                piece.id,
            ],
        )?;
        Ok(())
    }

    pub fn update_exercise(&self, exercise: &Exercise) -> Result<()> {
        let tags_json = serde_json::to_string(&exercise.tags)?;
        let (tempo_marking, tempo_bpm) = split_tempo(&exercise.tempo);

        self.conn.execute(
            "UPDATE exercises SET title=?1, composer=?2, category=?3, key=?4, tempo_marking=?5, tempo_bpm=?6, notes=?7, tags=?8, updated_at=?9 WHERE id=?10",
            params![
                exercise.title,
                exercise.composer,
                exercise.category,
                exercise.key,
                tempo_marking,
                tempo_bpm,
                exercise.notes,
                tags_json,
                exercise.updated_at.to_rfc3339(),
                exercise.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_item(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM pieces WHERE id=?1", params![id])?;
        self.conn.execute("DELETE FROM exercises WHERE id=?1", params![id])?;
        Ok(())
    }
}

fn split_tempo(tempo: &Option<Tempo>) -> (Option<&str>, Option<u16>) {
    match tempo {
        Some(t) => (t.marking.as_deref(), t.bpm),
        None => (None, None),
    }
}

fn parse_tempo(marking: Option<String>, bpm: Option<u16>) -> Option<Tempo> {
    if marking.is_some() || bpm.is_some() {
        Some(Tempo { marking, bpm })
    } else {
        None
    }
}

fn parse_timestamp(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .with_context(|| format!("Invalid timestamp: {s}"))
}

struct PieceRow {
    id: String,
    title: String,
    composer: String,
    key: Option<String>,
    tempo_marking: Option<String>,
    tempo_bpm: Option<u16>,
    notes: Option<String>,
    tags_str: String,
    created_str: String,
    updated_str: String,
}

struct ExerciseRow {
    id: String,
    title: String,
    composer: Option<String>,
    category: Option<String>,
    key: Option<String>,
    tempo_marking: Option<String>,
    tempo_bpm: Option<u16>,
    notes: Option<String>,
    tags_str: String,
    created_str: String,
    updated_str: String,
}

fn piece_from_row(row: PieceRow) -> Result<Piece> {
    let tags: Vec<String> = serde_json::from_str(&row.tags_str)
        .with_context(|| format!("Invalid tags JSON for piece {}", row.id))?;

    Ok(Piece {
        id: row.id,
        title: row.title,
        composer: row.composer,
        key: row.key,
        tempo: parse_tempo(row.tempo_marking, row.tempo_bpm),
        notes: row.notes,
        tags,
        created_at: parse_timestamp(&row.created_str)?,
        updated_at: parse_timestamp(&row.updated_str)?,
    })
}

fn exercise_from_row(row: ExerciseRow) -> Result<Exercise> {
    let tags: Vec<String> = serde_json::from_str(&row.tags_str)
        .with_context(|| format!("Invalid tags JSON for exercise {}", row.id))?;

    Ok(Exercise {
        id: row.id,
        title: row.title,
        composer: row.composer,
        category: row.category,
        key: row.key,
        tempo: parse_tempo(row.tempo_marking, row.tempo_bpm),
        notes: row.notes,
        tags,
        created_at: parse_timestamp(&row.created_str)?,
        updated_at: parse_timestamp(&row.updated_str)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_piece() -> Piece {
        let now = Utc::now();
        Piece {
            id: "p1".to_string(),
            title: "Clair de Lune".to_string(),
            composer: "Debussy".to_string(),
            key: Some("Db Major".to_string()),
            tempo: Some(Tempo {
                marking: Some("Andante".to_string()),
                bpm: Some(72),
            }),
            notes: Some("Beautiful piece".to_string()),
            tags: vec!["romantic".to_string(), "piano".to_string()],
            created_at: now,
            updated_at: now,
        }
    }

    fn make_exercise() -> Exercise {
        let now = Utc::now();
        Exercise {
            id: "e1".to_string(),
            title: "C Major Scale".to_string(),
            composer: None,
            category: Some("Scales".to_string()),
            key: Some("C Major".to_string()),
            tempo: None,
            notes: None,
            tags: vec!["technique".to_string()],
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_save_and_load_piece() {
        let store = SqliteStore::new_in_memory().unwrap();
        let piece = make_piece();
        store.save_piece(&piece).unwrap();

        let (pieces, exercises) = store.load_all().unwrap();
        assert_eq!(pieces.len(), 1);
        assert_eq!(exercises.len(), 0);

        let loaded = &pieces[0];
        assert_eq!(loaded.id, piece.id);
        assert_eq!(loaded.title, piece.title);
        assert_eq!(loaded.composer, piece.composer);
        assert_eq!(loaded.key, piece.key);
        assert_eq!(loaded.tempo, piece.tempo);
        assert_eq!(loaded.notes, piece.notes);
        assert_eq!(loaded.tags, piece.tags);
    }

    #[test]
    fn test_save_and_load_exercise() {
        let store = SqliteStore::new_in_memory().unwrap();
        let exercise = make_exercise();
        store.save_exercise(&exercise).unwrap();

        let (pieces, exercises) = store.load_all().unwrap();
        assert_eq!(pieces.len(), 0);
        assert_eq!(exercises.len(), 1);

        let loaded = &exercises[0];
        assert_eq!(loaded.id, exercise.id);
        assert_eq!(loaded.title, exercise.title);
        assert_eq!(loaded.composer, exercise.composer);
        assert_eq!(loaded.category, exercise.category);
        assert_eq!(loaded.key, exercise.key);
        assert_eq!(loaded.tempo, exercise.tempo);
        assert_eq!(loaded.tags, exercise.tags);
    }

    #[test]
    fn test_update_piece() {
        let store = SqliteStore::new_in_memory().unwrap();
        let mut piece = make_piece();
        store.save_piece(&piece).unwrap();

        piece.title = "Reverie".to_string();
        piece.tags = vec!["impressionist".to_string()];
        store.update_piece(&piece).unwrap();

        let (pieces, _) = store.load_all().unwrap();
        assert_eq!(pieces[0].title, "Reverie");
        assert_eq!(pieces[0].tags, vec!["impressionist".to_string()]);
    }

    #[test]
    fn test_update_exercise() {
        let store = SqliteStore::new_in_memory().unwrap();
        let mut exercise = make_exercise();
        store.save_exercise(&exercise).unwrap();

        exercise.title = "G Major Scale".to_string();
        exercise.category = Some("Arpeggios".to_string());
        store.update_exercise(&exercise).unwrap();

        let (_, exercises) = store.load_all().unwrap();
        assert_eq!(exercises[0].title, "G Major Scale");
        assert_eq!(exercises[0].category, Some("Arpeggios".to_string()));
    }

    #[test]
    fn test_delete_item() {
        let store = SqliteStore::new_in_memory().unwrap();
        let piece = make_piece();
        let exercise = make_exercise();
        store.save_piece(&piece).unwrap();
        store.save_exercise(&exercise).unwrap();

        store.delete_item("p1").unwrap();
        let (pieces, exercises) = store.load_all().unwrap();
        assert_eq!(pieces.len(), 0);
        assert_eq!(exercises.len(), 1);

        store.delete_item("e1").unwrap();
        let (pieces, exercises) = store.load_all().unwrap();
        assert_eq!(pieces.len(), 0);
        assert_eq!(exercises.len(), 0);
    }

    #[test]
    fn test_empty_database() {
        let store = SqliteStore::new_in_memory().unwrap();
        let (pieces, exercises) = store.load_all().unwrap();
        assert!(pieces.is_empty());
        assert!(exercises.is_empty());
    }

    #[test]
    fn test_piece_with_no_optional_fields() {
        let store = SqliteStore::new_in_memory().unwrap();
        let now = Utc::now();
        let piece = Piece {
            id: "p2".to_string(),
            title: "Untitled".to_string(),
            composer: "Unknown".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
        };
        store.save_piece(&piece).unwrap();

        let (pieces, _) = store.load_all().unwrap();
        assert_eq!(pieces[0].key, None);
        assert_eq!(pieces[0].tempo, None);
        assert_eq!(pieces[0].notes, None);
        assert!(pieces[0].tags.is_empty());
    }
}
