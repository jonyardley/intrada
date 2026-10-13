//! Deleting a saved session, and its undo (#2581, `specs/session-delete.md`).

use chrono::{DateTime, Utc};
use crux_core::Command;

use crate::app::{Effect, Event};
use crate::model::Model;
use crate::view::library::build_practice_summaries;

pub(super) fn delete(
    model: &mut Model,
    session_id: String,
    now: DateTime<Utc>,
) -> Command<Effect, Event> {
    let Some(at) = model.sessions.iter().position(|s| s.id == session_id) else {
        return Command::done();
    };
    let session = model.sessions.remove(at);
    model.practice_summaries = build_practice_summaries(&model.sessions).into();
    model.deleted_session = Some(session.clone());
    model.clear_error();
    Command::all([
        crate::persistence::delete_session(model, session, now),
        crux_core::render::render(),
    ])
}

/// A reload after a refused delete may have put it back already.
pub(super) fn undo_delete(model: &mut Model, now: DateTime<Utc>) -> Command<Effect, Event> {
    let Some(session) = model.deleted_session.take() else {
        return Command::done();
    };
    if model.sessions.iter().any(|s| s.id == session.id) {
        return Command::done();
    }
    model.sessions.push(session.clone());
    model.practice_summaries = build_practice_summaries(&model.sessions).into();
    model.clear_error();
    Command::all([
        crate::persistence::restore_session(model, session, now),
        crux_core::render::render(),
    ])
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use crux_core::{App, Command};

    use crate::app::{Effect, Event, Intrada};
    use crate::domain::session::{PracticeSession, SessionEvent, SetlistEntry};
    use crate::model::Model;
    use crate::persistence::{PersistenceOperation, PersistenceOutput};
    use crate::sync::SyncOperation;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_790_000_000 + secs, 0).unwrap()
    }

    fn practised(ids: &[&str]) -> Model {
        let sessions: Vec<PracticeSession> = ids
            .iter()
            .map(|id| PracticeSession {
                entries: vec![SetlistEntry::fixture()],
                ..PracticeSession::fixture(id)
            })
            .collect();
        Model {
            practice_summaries: crate::view::library::build_practice_summaries(&sessions).into(),
            sessions: sessions.into(),
            ..Default::default()
        }
    }

    fn ids(model: &Model) -> Vec<String> {
        model.sessions.iter().map(|s| s.id.clone()).collect()
    }

    fn delete(model: &mut Model, id: &str, now: DateTime<Utc>) -> Command<Effect, Event> {
        Intrada.update(
            Event::Session(SessionEvent::DeleteSaved {
                session_id: id.to_string(),
                now,
            }),
            model,
        )
    }

    fn undo(model: &mut Model, now: DateTime<Utc>) -> Command<Effect, Event> {
        Intrada.update(Event::Session(SessionEvent::UndoDelete { now }), model)
    }

    fn store_request(cmd: &mut Command<Effect, Event>) -> crux_core::Request<PersistenceOperation> {
        cmd.effects()
            .find_map(|e| match e {
                Effect::Persistence(req) => Some(req),
                _ => None,
            })
            .expect("a persistence request")
    }

    fn uploads(cmd: &mut Command<Effect, Event>) -> Vec<crate::sync::SyncRecord> {
        cmd.effects()
            .filter_map(|e| match e {
                Effect::Sync(req) => match req.operation {
                    SyncOperation::Upload(records) => Some(records),
                    _ => None,
                },
                _ => None,
            })
            .flatten()
            .collect()
    }

    #[test]
    fn a_delete_takes_the_session_out_and_tells_the_store_when() {
        let mut model = practised(&["kept", "gone"]);
        let mut cmd = delete(&mut model, "gone", at(10));

        assert_eq!(ids(&model), vec!["kept".to_string()]);
        assert_eq!(
            store_request(&mut cmd).operation,
            PersistenceOperation::DeleteSession {
                id: "gone".to_string(),
                deleted_at: at(10),
            }
        );
    }

    #[test]
    fn a_delete_takes_the_session_out_of_the_practice_summaries() {
        let mut model = practised(&["only"]);
        assert!(!model.practice_summaries.is_empty());
        let _ = delete(&mut model, "only", at(10));
        assert!(model.practice_summaries.is_empty());
    }

    #[test]
    fn a_kept_delete_uploads_the_deleted_record() {
        let mut model = practised(&["gone"]);
        let mut cmd = delete(&mut model, "gone", at(10));
        store_request(&mut cmd)
            .resolve(PersistenceOutput::Ack)
            .expect("resolves once");

        let sent = uploads(&mut cmd);
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].id, "gone");
        assert_eq!(sent[0].deleted_at, Some(at(10)));
        assert_eq!(sent[0].changed_at, at(10));
    }

    #[test]
    fn a_refused_delete_uploads_nothing_and_reloads_the_list() {
        let mut model = practised(&["gone"]);
        let mut cmd = delete(&mut model, "gone", at(10));
        store_request(&mut cmd)
            .resolve(PersistenceOutput::Failed)
            .expect("resolves once");
        assert!(uploads(&mut cmd).is_empty());
        let written = cmd.events().next().expect("the write's answer");

        let mut cmd = Intrada.update(written, &mut model);
        assert!(model.last_error.is_some());
        assert_eq!(
            store_request(&mut cmd).operation,
            PersistenceOperation::LoadSessions
        );
    }

    #[test]
    fn deleting_a_session_not_in_the_list_writes_nothing() {
        let mut model = practised(&["kept"]);
        let mut cmd = delete(&mut model, "missing", at(10));
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
        assert_eq!(ids(&model), vec!["kept".to_string()]);
    }

    #[test]
    fn undo_puts_the_session_back_and_restores_it_as_of_now() {
        let mut model = practised(&["kept", "gone"]);
        let _ = delete(&mut model, "gone", at(10));
        let mut cmd = undo(&mut model, at(20));

        let mut back = ids(&model);
        back.sort();
        assert_eq!(back, vec!["gone".to_string(), "kept".to_string()]);
        assert!(model.practice_summaries.contains_key("item-1"));
        assert_eq!(
            store_request(&mut cmd).operation,
            PersistenceOperation::RestoreSession {
                id: "gone".to_string(),
                restored_at: at(20),
            }
        );
    }

    #[test]
    fn a_kept_undo_uploads_the_live_record_changed_now() {
        let mut model = practised(&["gone"]);
        let _ = delete(&mut model, "gone", at(10));
        let mut cmd = undo(&mut model, at(20));
        store_request(&mut cmd)
            .resolve(PersistenceOutput::Ack)
            .expect("resolves once");

        let sent = uploads(&mut cmd);
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].deleted_at, None);
        assert_eq!(sent[0].changed_at, at(20));
    }

    #[test]
    fn undo_after_a_refused_delete_has_put_it_back_writes_nothing() {
        let mut model = practised(&["gone"]);
        let mut cmd = delete(&mut model, "gone", at(10));
        store_request(&mut cmd)
            .resolve(PersistenceOutput::Failed)
            .expect("resolves once");
        let written = cmd.events().next().expect("the write's answer");
        let mut cmd = Intrada.update(written, &mut model);
        store_request(&mut cmd)
            .resolve(PersistenceOutput::Sessions(vec![PracticeSession::fixture(
                "gone",
            )]))
            .expect("resolves once");
        let loaded = cmd.events().next().expect("the reload");
        let _ = Intrada.update(loaded, &mut model);

        let mut cmd = undo(&mut model, at(20));
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
        assert_eq!(ids(&model), vec!["gone".to_string()]);
    }

    #[test]
    fn the_bridge_shapes_round_trip_on_the_ffi_wire() {
        use crate::domain::types::assert_round_trips;
        assert_round_trips(PersistenceOperation::DeleteSession {
            id: "s".to_string(),
            deleted_at: at(1),
        });
        assert_round_trips(PersistenceOperation::RestoreSession {
            id: "s".to_string(),
            restored_at: at(2),
        });
        assert_round_trips(Event::Session(SessionEvent::DeleteSaved {
            session_id: "s".to_string(),
            now: at(1),
        }));
        assert_round_trips(Event::Session(SessionEvent::UndoDelete { now: at(2) }));
        assert_round_trips(Event::SessionDeleteWritten(PersistenceOutput::Ack));
    }

    #[test]
    fn undo_happens_once() {
        let mut model = practised(&["gone"]);
        let _ = delete(&mut model, "gone", at(10));
        let _ = undo(&mut model, at(20));
        let mut cmd = undo(&mut model, at(30));
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
        assert_eq!(ids(&model), vec!["gone".to_string()]);
    }
}
