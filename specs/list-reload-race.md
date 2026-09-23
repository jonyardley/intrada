# A list load never overwrites a newer edit

> Tier 3 (what the core keeps between a load and its result). Core only: no
> event, effect or view shape changes, so nothing crosses the bridge. Issue
> #2067, epic #2002; unblocks #2068 (#2004).

## Problem

The core asks the app for the whole library (`LoadItems`) at launch and again
after a save the storage refused, to roll back the change it could not keep.
Practice history works the same way (`LoadSessions`). When the result lands,
the core replaces its list with it.

While the app ran every read and write inside the tap, nothing could happen
between asking for a list and getting it back. Once disk work moves off the
main thread (#2068), a musician can edit an item while the list is still on
its way. The edit reaches storage, then the older list lands and replaces it
on screen. Edit the same item again from that stale copy and the first edit is
overwritten on disk. The September audit named the gap before it opened
(#1952, finding 25).

Nothing in `PersistenceOutput` says which request a result answers, and
adding a request id would change the bridge.

## Approach: the core counts what it has sent

Each list keeps three facts in the model, in a small `ListSync`:

- writes sent and not yet answered;
- loads sent and not yet answered;
- whether the next load result must be dropped (stale).

The rules:

1. **Sending a write** counts it, and marks the list stale if a load is out:
   that load may have read the disk before the write reached it.
2. **A load result lands.** It is applied only when the list is not stale and
   no write is out. Otherwise it is dropped and the list is marked stale.
3. **Asking again.** Whenever the list is stale with nothing out, neither
   writes nor loads, the core sends one fresh load and clears the mark. That
   load is the first one sent after every write it could have missed.
4. **A write the storage refuses** marks the list stale instead of reloading
   straight away, then settles like any other answer. The rollback reload
   therefore waits for every write still out, and a load already out cannot
   land over it.
5. **A load that fails** counts as answered and never asks again by itself,
   so a broken store cannot loop (#825). A later write that settles picks the
   stale mark back up.

Dropping a result while a write is out is stricter than it needs to be when
the app runs disk jobs in order, as #2068 does: a write sent before the load
is already inside it. Keeping the stricter rule means the core relies on no
promise about ordering from the app.

### Where the counting happens

The write helpers in `persistence.rs` take the model, so a write that skips
the count does not compile. Loads go through the same file. The `ListSync`
methods return what to do (apply, drop, or reload) and `app.rs` acts on it,
so the rules live and are tested in one place.

### Sessions

The same rules, on a second `ListSync`. A finished practice waits in
`saving_session` until the store answers (#974); an acknowledged save still
adds it to the list, and a reload the rules ask for afterwards brings the same
row back from disk.

## Not in this change

- **A "library loaded" flag on the view.** It would show nothing until a
  screen reads it, and #2068 adds it only if the blank library shows at
  launch on a real phone. A new view field is a bridge change and would split
  this into two PRs.
- **The shell.** #2068 moves disk work off the main thread; it needs no change
  for this, since every result already comes back as the same event.
- **#1952's wording** in `CLAUDE.md` and the offline-first rule.
- **Sample data.** `LoadSampleData` sends no load or write.

## Tests (core, written first)

In `persistence.rs`, each driven through `App::update`:

- a write sent while a load is out: the load's result is dropped, and a load
  is asked for once the write is acknowledged;
- a write acknowledged before the older load lands: the result is dropped and
  a load is asked for as it lands;
- a load landing while a write sent before it is still out: dropped, reload
  after the write;
- a refused write while a load is out: no reload until that load lands, then
  one;
- a refused write with nothing out: reloads at once, as before;
- two loads out at once around a write: neither applies, one reload;
- a failed load with a write out: no reload from the failure itself;
- the same first case for practice history;
- no write or load out: a result applies as before.

Each is checked by deleting the line it guards, not by inverting it (#1423).
