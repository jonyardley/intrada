//! The shared store across the bridge (#2421): the phone passes the
//! operation's bincode and resolves the core with the output's.

use std::sync::{Mutex, PoisonError};

use crux_core::bridge::{BincodeFfiFormat, FfiFormat};
use intrada_core::PersistenceOperation;
use intrada_store::Store;

use crate::ffi::{install_panic_hook, with_panic_location};
use crate::CoreError;

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreAnswer {
    /// A bincode `PersistenceOutput`, `Failed` when the operation did not complete.
    pub output: Vec<u8>,
    /// Stored values the store could not read, for the phone to report.
    pub unreadable: Vec<String>,
    /// Why the operation failed, for the phone to report.
    pub error: Option<String>,
}

/// One request at a time: the lock is the queue the phone's disk jobs share.
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct StoreFFI {
    store: Mutex<Store>,
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl StoreFFI {
    /// Opens or creates the database at `path` and migrates it.
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn open(path: String) -> Result<Self, CoreError> {
        install_panic_hook();
        with_panic_location(|| Self::from(Store::open(path)))
    }

    /// The fallback when the file will not open, so the app still runs.
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn in_memory() -> Result<Self, CoreError> {
        install_panic_hook();
        with_panic_location(|| Self::from(Store::in_memory()))
    }

    pub fn handle(&self, operation: &[u8]) -> Result<StoreAnswer, CoreError> {
        with_panic_location(|| {
            let operation: PersistenceOperation = BincodeFfiFormat::deserialize(operation)
                .map_err(|e| CoreError::Bridge(format!("persistence operation: {e}")))?;
            // A panic mid-operation rolled its transaction back, so the
            // connection is still sound.
            let answer = self
                .store
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .handle(&operation);
            let mut output = Vec::new();
            BincodeFfiFormat::serialize(&mut output, &answer.output)
                .map_err(|e| CoreError::Bridge(format!("persistence output: {e}")))?;
            Ok(StoreAnswer {
                output,
                unreadable: answer.unreadable,
                error: answer.error,
            })
        })
    }
}

impl StoreFFI {
    fn from(opened: Result<Store, intrada_store::StoreError>) -> Result<Self, CoreError> {
        opened
            .map(|store| Self {
                store: Mutex::new(store),
            })
            .map_err(|e| CoreError::Bridge(format!("store: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intrada_core::{Item, ItemKind, PersistenceOutput};

    fn bytes(value: &PersistenceOperation) -> Vec<u8> {
        let mut out = Vec::new();
        BincodeFfiFormat::serialize(&mut out, value).expect("encodes");
        out
    }

    fn output(answer: &StoreAnswer) -> PersistenceOutput {
        BincodeFfiFormat::deserialize(&answer.output).expect("decodes")
    }

    fn piece() -> Item {
        let at = "2026-09-01T10:00:00Z".parse().expect("a time");
        Item {
            id: "p1".into(),
            title: "Nocturne".into(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: at,
            updated_at: at,
            priority: false,
            chord_chart: None,
            photo_id: None,
            metre: None,
            sections: vec![],
            variation_ids: vec![],
            keys: vec![],
            exercise_links: vec![],
        }
    }

    #[test]
    fn a_saved_item_is_there_when_the_file_opens_again() {
        let path = std::env::temp_dir().join(format!(
            "intrada-store-{}-{}.sqlite",
            std::process::id(),
            "reopen"
        ));
        let _ = std::fs::remove_file(&path);
        let path_text = path.to_string_lossy().to_string();

        let store = StoreFFI::open(path_text.clone()).expect("opens");
        let saved = store
            .handle(&bytes(&PersistenceOperation::SaveItem(piece())))
            .expect("answers");
        assert_eq!(output(&saved), PersistenceOutput::Ack);
        drop(store);

        let reopened = StoreFFI::open(path_text).expect("opens again");
        let loaded = reopened
            .handle(&bytes(&PersistenceOperation::LoadItems))
            .expect("answers");
        assert_eq!(output(&loaded), PersistenceOutput::Items(vec![piece()]));
        assert_eq!((loaded.unreadable, loaded.error), (vec![], None));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_failed_operation_answers_failed_with_the_reason() {
        let store = StoreFFI::in_memory().expect("opens");
        let mut broken = piece();
        broken.sections = vec![intrada_core::domain::section::ItemSection {
            id: "s1".into(),
            name: "A".into(),
            bars: None,
            kind: intrada_core::domain::section::SectionKind::Form,
            target_bpm: None,
            position: usize::MAX,
            updated_at: broken.updated_at,
            deleted_at: None,
        }];
        let answer = store
            .handle(&bytes(&PersistenceOperation::SaveItem(broken)))
            .expect("answers");
        assert_eq!(output(&answer), PersistenceOutput::Failed);
        assert!(answer.error.is_some());
    }

    #[test]
    fn bytes_that_are_not_an_operation_are_refused() {
        let store = StoreFFI::in_memory().expect("opens");
        assert!(store.handle(&[0xff, 0xff, 0xff, 0xff]).is_err());
    }

    #[test]
    fn a_path_that_cannot_hold_a_database_is_refused() {
        assert!(StoreFFI::open("/nonexistent-dir/intrada.sqlite".into()).is_err());
    }
}
