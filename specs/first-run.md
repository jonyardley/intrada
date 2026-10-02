# First run: the welcome and the first-session steps

> Tier 3 (a new bridge shape and a new saved singleton). Rides as the first
> commit of the core PR for #2116, part of #2121. Native iOS only.

## Problem

A new install opens on an empty Library with nothing pointing onward. The
first-run screens (#2117, #2118) need two answers the shell must not work out
for itself: whether to show the welcome, and which of the four first-session
steps are done. Both are decisions, so both live in the core.

## Approach: the steps are facts the core already holds

Nothing about progress is stored. Each step reads data the musician made,
so a tick can never disagree with the library or the history. The one new
piece of state is whether the welcome has been dismissed.

### The four steps

| Step | Ticked when |
|---|---|
| Added | the library holds a piece or an exercise |
| Built | a session is being played or summarised, or one has been saved |
| Played | a saved session has an item played (`EntryStatus::Completed`) |
| Marked | a saved session carries a mark: a session mark, or any play's mark |

Step one also carries the title of the earliest item added, for the card's
line ("Added Clair de Lune").

**Built means started, not assembled.** The builder's state is not saved,
so a tick for a half-built setlist would vanish on the next launch and the
card would lie. A session counts from the moment it starts playing, which is
also what starting from the First session card does (Jon's decision on
#2121): that card skips the builder and the tick still lands.

**An abandoned session leaves no tick.** Discarding before saving keeps
nothing, so Built drops back. Ending early and saving keeps the session, so
Built stays; Played needs one item actually played.

**Played and Marked count saved sessions only.** A mark given on the summary
is not kept until Save, so the tick lands with the save, which is also when
the musician comes back to Practice and sees the card. A session the store is
still writing (`saving_session`) counts, so the tick does not flicker.

### When the welcome shows

`shows_welcome` is true only when all of these hold:

1. the welcome has not been dismissed on this device;
2. both the library and the history have finished loading from the store;
3. the library and the history are both empty;
4. no profile has been saved (it is still the default).

Conditions 2 to 4 keep the welcome away from every musician who already uses
the app: an update writes no welcome blob, so condition 1 alone would show it
to all of them. Condition 2 stops it flashing at launch before the store
answers. A load that fails leaves the welcome hidden: an unknown library is
not an empty one.

### When the Start here card shows

`shows_start_here` is true once both loads have landed and until a saved
session carries a mark. A musician who never marks keeps the card; marking
is the step the loop exists for, and a dismiss would need state of its own.
Revisit if testers in #2120 find it nagging.

### Data model (core)

New module `domain/first_run.rs`:

```rust
pub struct FirstRun {
    pub welcome_seen: bool,
}

impl FirstRun {
    pub const BLOB_VERSION: u32 = 1;
}

pub enum FirstRunEvent {
    SkipWelcome,      // the welcome's Skip
    Loaded(FirstRun), // the shell replaying the stored blob at launch
}

pub struct FirstRunView {
    pub shows_welcome: bool,
    pub shows_start_here: bool,
    pub added: bool,
    pub built: bool,
    pub played: bool,
    pub marked: bool,
    pub first_item_title: Option<String>,
}
```

`ViewModel.first_run: FirstRunView` carries all of it, so the screens PR
does not change the core.

- `SkipWelcome` sets `welcome_seen`, emits `AppEffect::SaveFirstRun` and
  renders. A second Skip writes nothing.
- `ProfileEvent::Save` also sets `welcome_seen` when it is still false and
  emits the save beside `SaveProfile`: saving a profile is the welcome's
  other way out, and any saved profile means the musician has met the app.
  A refused profile save leaves the welcome where it was.
- `Loaded` sets the model and renders, never re-saving.

Whether each list has loaded comes from `ListSync`, which learns to say so
the first time a load lands and is applied.

### Persistence: a fifth singleton beside the profile

The same fire-and-forget `AppEffect` route as the profile and the practice
defaults (offline-first invariant 8).

- **Write:** `AppEffect::SaveFirstRun(FirstRun)`; `Store.swift`
  bincode-serialises it under `intrada.first-run.v1`, the key built from
  `first_run_blob_version()` across the bridge, as #2026 did for the profile.
- **Read:** at launch, beside the profile restore, the shell decodes the blob
  and sends `Loaded`. Missing or undecodable is a no-op and the welcome
  conditions decide.
- A core test pins the wire bytes of a fixture as hex with the version.

## Key decisions

1. **No progress stored.** The ticks are derived, so they cannot drift.
2. **Built is started.** The builder is not saved; a tick must survive a
   restart.
3. **Existing musicians never see the welcome.** Empty library, empty
   history and default profile, after both loads, as well as not dismissed.
4. **Saving a profile dismisses the welcome.** One event from the shell, no
   sequencing in Swift.
5. **The card stays until the first mark.** No dismiss, no stored state.

## Docs riding with this PR

- `docs/design-principles.md` T34: the first run teaches by doing. No tour,
  no cards, no sample content (T21); one welcome, then a card whose ticks
  are the musician's own data.
- `docs/tone-of-voice.md` V8: the welcome has no maker's note. It speaks in
  the app's own voice, and rule 4's ban on `I` holds there too.

## Deliberately not doing

- The five-card carousel in `specs/onboarding-welcome.md`, the web-era
  record this replaces; T34 says why.
- Seeded runs (`--seed-sample-data`) skip the store, so neither list ever
  counts as loaded and neither the welcome nor the card shows there.

- Screens: the welcome, the profile step, the first piece and the card
  (#2117, #2118), and the empty-tab buttons (#2119). The save, the restore at
  launch and the preview fixtures ride with this PR, since the shell's effect
  switch must stay exhaustive for the app to build.
- Sample content to try the app with (#2115).
- A way to see the welcome again.

## Testing

- **Core, test-first:** a table test walking an empty app to a marked
  session, one row per step, where deleting the marked check fails the last
  row; starting from today's plan ticks Built with no builder; discarding an
  active session drops Built; ending early with nothing played ticks Built
  but not Played; the welcome hidden before the loads land, hidden for a
  library with an item, a history with a session or a saved profile, and
  hidden after Skip; Skip saves once; a profile save dismisses and saves; a
  refused profile save does not; `Loaded` never re-saves; the first item is
  the earliest added; the wire hex pinned; events, effect and view
  round-trip on the bincode wire (#846).
- **iOS:** the save writes the blob under the versioned key and decodes
  through `LiveBridge`; restore replays `Loaded`; restore with nothing
  stored is a no-op; the welcome stays dismissed across a restart.
