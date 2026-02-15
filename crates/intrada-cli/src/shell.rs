use anyhow::Result;
use crux_core::Core;
use intrada_core::app::{Effect, StorageEffect};
use intrada_core::{Event, Intrada, ViewModel};

use crate::storage::JsonStore;

pub struct Shell {
    core: Core<Intrada>,
    store: JsonStore,
}

impl Shell {
    pub fn new(store: JsonStore) -> Self {
        Self {
            core: Core::new(),
            store,
        }
    }

    pub fn run(&self, event: Event) -> Result<ViewModel> {
        let effects = self.core.process_event(event);
        self.handle_effects(effects)?;
        Ok(self.core.view())
    }

    pub fn load_data(&self) -> Result<ViewModel> {
        let (pieces, exercises) = self.store.load_all()?;
        self.run(Event::DataLoaded { pieces, exercises })
    }

    fn handle_effects(&self, effects: Vec<Effect>) -> Result<()> {
        for effect in effects {
            match effect {
                Effect::Render(_) => {
                    // Fire-and-forget — view will be read after all effects
                }
                Effect::Storage(boxed_request) => {
                    match &boxed_request.operation {
                        StorageEffect::LoadAll => {
                            // Shell handles LoadAll via load_data(), not here
                        }
                        StorageEffect::SavePiece(piece) => {
                            self.store.save_piece(piece)?;
                        }
                        StorageEffect::SaveExercise(exercise) => {
                            self.store.save_exercise(exercise)?;
                        }
                        StorageEffect::UpdatePiece(piece) => {
                            self.store.update_piece(piece)?;
                        }
                        StorageEffect::UpdateExercise(exercise) => {
                            self.store.update_exercise(exercise)?;
                        }
                        StorageEffect::DeleteItem { id } => {
                            self.store.delete_item(id)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intrada_core::domain::piece::PieceEvent;
    use intrada_core::domain::types::CreatePiece;

    fn test_shell() -> (Shell, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("library.json");
        let store = JsonStore::new_with_path(path);
        (Shell::new(store), dir)
    }

    #[test]
    fn test_load_empty_data() {
        let (shell, _dir) = test_shell();
        let vm = shell.load_data().unwrap();
        assert!(vm.items.is_empty());
    }

    #[test]
    fn test_add_piece_round_trip() {
        let (shell, _dir) = test_shell();
        shell.load_data().unwrap();

        let vm = shell
            .run(Event::Piece(PieceEvent::Add(CreatePiece {
                title: "Clair de Lune".to_string(),
                composer: "Debussy".to_string(),
                key: Some("Db Major".to_string()),
                tempo: None,
                notes: None,
                tags: vec![],
            })))
            .unwrap();

        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Clair de Lune");
        assert!(vm.error.is_none());

        let vm2 = shell.run(Event::ClearError).unwrap();
        assert_eq!(vm2.items.len(), 1);
    }

    #[test]
    fn test_add_piece_persists_to_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("library.json");
        let store = JsonStore::new_with_path(path);

        let shell = Shell::new(store);
        shell.load_data().unwrap();
        shell
            .run(Event::Piece(PieceEvent::Add(CreatePiece {
                title: "Sonata".to_string(),
                composer: "Beethoven".to_string(),
                key: None,
                tempo: None,
                notes: None,
                tags: vec!["classical".to_string()],
            })))
            .unwrap();

        // Reload from JSON file to verify persistence
        let vm = shell.load_data().unwrap();
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Sonata");
    }

    #[test]
    fn test_unicode_piece_round_trip() {
        let (shell, _dir) = test_shell();
        shell.load_data().unwrap();

        let vm = shell
            .run(Event::Piece(PieceEvent::Add(CreatePiece {
                title: "Ménuet in G".to_string(),
                composer: "Dvořák".to_string(),
                key: Some("G Máj".to_string()),
                tempo: None,
                notes: Some("Très belle pièce — «magnifique»".to_string()),
                tags: vec!["romántico".to_string(), "日本語".to_string()],
            })))
            .unwrap();

        assert!(vm.error.is_none());
        assert_eq!(vm.items.len(), 1);
        assert_eq!(vm.items[0].title, "Ménuet in G");
        assert_eq!(vm.items[0].subtitle, "Dvořák");

        // Reload from JSON to verify round-trip
        let vm2 = shell.load_data().unwrap();
        assert_eq!(vm2.items.len(), 1);
        assert_eq!(vm2.items[0].title, "Ménuet in G");
        assert_eq!(vm2.items[0].subtitle, "Dvořák");
        assert_eq!(vm2.items[0].key, Some("G Máj".to_string()));
        assert_eq!(
            vm2.items[0].notes,
            Some("Très belle pièce — «magnifique»".to_string())
        );
        assert_eq!(
            vm2.items[0].tags,
            vec!["romántico".to_string(), "日本語".to_string()]
        );
    }

    #[test]
    fn test_unicode_exercise_round_trip() {
        let (shell, _dir) = test_shell();
        shell.load_data().unwrap();

        let vm = shell
            .run(Event::Exercise(
                intrada_core::domain::exercise::ExerciseEvent::Add(
                    intrada_core::domain::types::CreateExercise {
                        title: "Übung für die linke Hand".to_string(),
                        composer: Some("Czerny".to_string()),
                        category: Some("Técnica".to_string()),
                        key: None,
                        tempo: None,
                        notes: None,
                        tags: vec!["größe".to_string()],
                    },
                ),
            ))
            .unwrap();

        assert!(vm.error.is_none());

        let vm2 = shell.load_data().unwrap();
        assert_eq!(vm2.items[0].title, "Übung für die linke Hand");
        assert_eq!(vm2.items[0].tags, vec!["größe".to_string()]);
    }

    #[test]
    fn test_field_length_at_boundary() {
        let (shell, _dir) = test_shell();
        shell.load_data().unwrap();

        let vm = shell
            .run(Event::Piece(PieceEvent::Add(CreatePiece {
                title: "x".repeat(500),
                composer: "y".repeat(200),
                key: None,
                tempo: None,
                notes: Some("z".repeat(5000)),
                tags: vec!["t".repeat(100)],
            })))
            .unwrap();
        assert!(vm.error.is_none());
        assert_eq!(vm.items.len(), 1);

        let vm2 = shell.load_data().unwrap();
        assert_eq!(vm2.items[0].title.len(), 500);
    }

    #[test]
    fn test_field_length_over_boundary() {
        let (shell, _dir) = test_shell();
        shell.load_data().unwrap();

        let vm = shell
            .run(Event::Piece(PieceEvent::Add(CreatePiece {
                title: "x".repeat(501),
                composer: "Composer".to_string(),
                key: None,
                tempo: None,
                notes: None,
                tags: vec![],
            })))
            .unwrap();
        assert!(vm.error.is_some());
        assert_eq!(vm.items.len(), 0);
    }

    #[test]
    fn test_validation_error_surfaces() {
        let (shell, _dir) = test_shell();
        shell.load_data().unwrap();

        let vm = shell
            .run(Event::Piece(PieceEvent::Add(CreatePiece {
                title: "".to_string(),
                composer: "Debussy".to_string(),
                key: None,
                tempo: None,
                notes: None,
                tags: vec![],
            })))
            .unwrap();

        assert!(vm.error.is_some());
        assert_eq!(vm.items.len(), 0);
    }
}
