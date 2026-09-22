# Save session waits for the store

> Tier 3, Fable list (the crash-recovery blob lifecycle). Rides with the
> implementation branch for #974, finding 3 of `docs/audit-2026-09.md`.
> Native iOS only.

## Problem

Tapping Save session does four things at once: pushes the finished practice
into `model.sessions`, sends the write to GRDB, tells the shell to clear the
crash-recovery copy, and closes the summary. All four happen when the write is
dispatched, not when the store answers.

If the write comes back `Failed`, the core reloads sessions from disk to roll
back the optimistic push (#825). The push is gone, the recovery copy is gone,
and the summary closed a frame ago. The practice exists nowhere. The musician
sees one banner, and if they dismissed a storage banner earlier in the launch
they see nothing at all.

The August fix (#1936) made the banner honest again. This makes the save
honest: nothing is treated as saved until the store says so.

## Approach: park the session until the store acknowledges it

`SaveSession` builds the `PracticeSession` from the summary, parks it in a
new `Model.saving_session`, stays in `SessionStatus::Summary`, and sends only
the write. Nothing else moves.

`SessionStoreWritten` then decides:

- `Ack` with a parked session: push it, rebuild `practice_summaries`, clear
  the recovery copy, `record_ack`, and go `Idle` if the summary being saved
  is still on screen. The push happens even if the musician discarded the
  summary meanwhile, because the row is already on disk and the model must
  say what the disk says.
- `Failed` with a parked session: unpark, stay in Summary, `raise_error`.
  Never `surface_error`: the failure answers the musician's own tap, so the
  dismiss mute does not apply. No sessions reload, since nothing was pushed.
- Either answer with nothing parked: the existing behaviour, unchanged.

A second Save while one is parked is dropped. One save is in flight at a
time, so the event carries no id and the ack needs none.

### What does not change

- No bridge shape: `SaveSession`, `PersistenceOperation::SaveSession`,
  `PersistenceOutput` and the `ViewModel` are as they were. One core PR.
- The recovery copy's contents and the `ActiveSession` graph. `Store.
  sessionInProgressKey` stays at v4.
- `DiscardSession`: clears the copy at once, as before. A discard is the
  musician's decision, not a store outcome.
- The summary screen: Save session is the same button sending the same event.
  While the write is in flight the screen shows the summary; the write is
  milliseconds on device, so no saving state is drawn.

### Failure the musician sees

The summary stays up with the banner "Couldn't save this practice. Your
notes and scores are still here: try Save again." Save session is still
there and does the same thing. Killing the app at that point resumes the
player from the recovery copy, which still holds the last active snapshot
(pre-existing, unchanged by this spec).

## Tests (core, test-first)

- Save parks and sends the write; the model has no new session, the status
  is still Summary, and no `ClearSessionInProgress` is sent.
- Ack after Save pushes the session, rebuilds summaries, clears the copy,
  records the ack and goes Idle.
- Failed after Save keeps the summary, keeps the parked session out of
  `sessions`, sends no reload, and shows the banner even with the mute set.
- Save twice sends one write.
- Discard then Ack still pushes the session and leaves the status Idle.
- Ack and Failed with nothing parked behave as on main (the #1936 table
  tests hold).
- Mutation by deleting each new line, recorded in the PR.

## Out of scope

- Resuming to the summary rather than the player after a kill on the summary
  screen.
- A saving state on the button.
- #1937 (a refused save on the item form), which follows this.
