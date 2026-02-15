use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use intrada_core::domain::exercise::Exercise;
use intrada_core::domain::piece::Piece;
use intrada_core::LibraryData;

pub struct JsonStore {
    path: PathBuf,
}

impl JsonStore {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_local_dir()
            .context("Could not determine local data directory")?
            .join("intrada");
        std::fs::create_dir_all(&data_dir)
            .with_context(|| format!("Could not create data directory: {}", data_dir.display()))?;

        Ok(Self {
            path: data_dir.join("library.json"),
        })
    }

    #[cfg(test)]
    pub fn new_with_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load_all(&self) -> Result<(Vec<Piece>, Vec<Exercise>)> {
        if !self.path.exists() {
            return Ok((Vec::new(), Vec::new()));
        }

        let contents = std::fs::read_to_string(&self.path)
            .with_context(|| format!("Could not read {}", self.path.display()))?;

        let data: LibraryData = serde_json::from_str(&contents)
            .with_context(|| format!("Invalid JSON in {}", self.path.display()))?;

        Ok((data.pieces, data.exercises))
    }

    pub fn save_piece(&self, piece: &Piece) -> Result<()> {
        let mut data = self.read_library()?;
        data.pieces.push(piece.clone());
        self.write_library(&data)
    }

    pub fn save_exercise(&self, exercise: &Exercise) -> Result<()> {
        let mut data = self.read_library()?;
        data.exercises.push(exercise.clone());
        self.write_library(&data)
    }

    pub fn update_piece(&self, piece: &Piece) -> Result<()> {
        let mut data = self.read_library()?;
        if let Some(existing) = data.pieces.iter_mut().find(|p| p.id == piece.id) {
            *existing = piece.clone();
        }
        self.write_library(&data)
    }

    pub fn update_exercise(&self, exercise: &Exercise) -> Result<()> {
        let mut data = self.read_library()?;
        if let Some(existing) = data.exercises.iter_mut().find(|e| e.id == exercise.id) {
            *existing = exercise.clone();
        }
        self.write_library(&data)
    }

    pub fn delete_item(&self, id: &str) -> Result<()> {
        let mut data = self.read_library()?;
        data.pieces.retain(|p| p.id != id);
        data.exercises.retain(|e| e.id != id);
        self.write_library(&data)
    }

    fn read_library(&self) -> Result<LibraryData> {
        if !self.path.exists() {
            return Ok(LibraryData::default());
        }
        let contents = std::fs::read_to_string(&self.path)
            .with_context(|| format!("Could not read {}", self.path.display()))?;
        serde_json::from_str(&contents)
            .with_context(|| format!("Invalid JSON in {}", self.path.display()))
    }

    fn write_library(&self, data: &LibraryData) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;

        // Atomic write: write to temp file in same directory, then rename.
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        let tmp_path = dir.join(".library.json.tmp");
        std::fs::write(&tmp_path, &json)
            .with_context(|| format!("Could not write temp file: {}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, &self.path)
            .with_context(|| format!("Could not rename temp file to {}", self.path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use intrada_core::Tempo;

    fn temp_store() -> (JsonStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("library.json");
        (JsonStore::new_with_path(path), dir)
    }

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
        let (store, _dir) = temp_store();
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
        let (store, _dir) = temp_store();
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
        let (store, _dir) = temp_store();
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
        let (store, _dir) = temp_store();
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
        let (store, _dir) = temp_store();
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
    fn test_empty_store() {
        let (store, _dir) = temp_store();
        let (pieces, exercises) = store.load_all().unwrap();
        assert!(pieces.is_empty());
        assert!(exercises.is_empty());
    }

    #[test]
    fn test_piece_with_no_optional_fields() {
        let (store, _dir) = temp_store();
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

    #[test]
    fn test_missing_file_returns_empty() {
        let (store, _dir) = temp_store();
        // File doesn't exist yet
        let (pieces, exercises) = store.load_all().unwrap();
        assert!(pieces.is_empty());
        assert!(exercises.is_empty());
    }

    #[test]
    fn test_malformed_json_returns_error() {
        let (store, _dir) = temp_store();
        std::fs::write(&store.path, "{ not valid json !!!").unwrap();

        let result = store.load_all();
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_fields_deserialise_ok() {
        let (store, _dir) = temp_store();
        let json = r#"{
            "pieces": [],
            "exercises": [],
            "future_field": "some value",
            "another_unknown": 42
        }"#;
        std::fs::write(&store.path, json).unwrap();

        let (pieces, exercises) = store.load_all().unwrap();
        assert!(pieces.is_empty());
        assert!(exercises.is_empty());
    }
}
