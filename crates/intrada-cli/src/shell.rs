use anyhow::Result;
use crux_core::Core;
use intrada_core::app::{Effect, StorageEffect};
use intrada_core::{Event, Intrada, ViewModel};

use crate::storage::SqliteStore;

pub struct Shell {
    core: Core<Intrada>,
    store: SqliteStore,
}

impl Shell {
    pub fn new(store: SqliteStore) -> Self {
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
                Effect::Storage(request) => {
                    match &request.operation {
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

    fn test_shell() -> Shell {
        let store = SqliteStore::new_in_memory().unwrap();
        Shell::new(store)
    }

    #[test]
    fn test_load_empty_data() {
        let shell = test_shell();
        let vm = shell.load_data().unwrap();
        assert_eq!(vm.item_count, 0);
        assert!(vm.items.is_empty());
    }

    #[test]
    fn test_add_piece_round_trip() {
        let shell = test_shell();
        shell.load_data().unwrap();

        let vm = shell.run(Event::Piece(PieceEvent::Add(CreatePiece {
            title: "Clair de Lune".to_string(),
            composer: "Debussy".to_string(),
            key: Some("Db Major".to_string()),
            tempo: None,
            notes: None,
            tags: vec![],
        }))).unwrap();

        assert_eq!(vm.item_count, 1);
        assert_eq!(vm.items[0].title, "Clair de Lune");
        assert!(vm.error.is_none());

        // Verify persisted — create a new shell with same store
        // (can't reuse store, but verify via load_data which re-reads model)
        // Instead, verify the view is correct
        let vm2 = shell.run(Event::ClearError).unwrap();
        assert_eq!(vm2.item_count, 1);
    }

    #[test]
    fn test_add_piece_persists_to_db() {
        let store = SqliteStore::new_in_memory().unwrap();

        // Add via shell
        let shell = Shell::new(store);
        shell.load_data().unwrap();
        shell.run(Event::Piece(PieceEvent::Add(CreatePiece {
            title: "Sonata".to_string(),
            composer: "Beethoven".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec!["classical".to_string()],
        }))).unwrap();

        // Verify in DB by loading again
        let vm = shell.load_data().unwrap();
        // DataLoaded replaces model, so we get only what's in DB
        assert_eq!(vm.item_count, 1);
        assert_eq!(vm.items[0].title, "Sonata");
    }

    #[test]
    fn test_validation_error_surfaces() {
        let shell = test_shell();
        shell.load_data().unwrap();

        let vm = shell.run(Event::Piece(PieceEvent::Add(CreatePiece {
            title: "".to_string(), // invalid: empty title
            composer: "Debussy".to_string(),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        }))).unwrap();

        assert!(vm.error.is_some());
        assert_eq!(vm.item_count, 0);
    }
}
