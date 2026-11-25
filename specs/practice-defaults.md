# Practice defaults: the click start and the session length

> Tier 3 (it added a bridge shape and a saved singleton). Shipped with #1915
> and #1736, part of #1974. The repetition target it once held went with
> #2791 (`simpler-removals.md`). Native iOS only.

## Problem

The metronome always started on every beat, and a new session always started
at the same length. A musician who always wants the click on 2 and 4, or
always practises for 25 minutes, changed both again every time. Both are
habits the musician keeps from session to session.

The every-beat start was chosen in Swift (`ClickController.reseed`), against
the rule that the shell makes no domain decisions.

## Approach: two defaults the core owns, saved as their own singleton

The core holds the defaults, validates them, and applies them. The shell
writes one blob to UserDefaults when told to, replays it at launch, renders
the Profile screen's controls, and starts the metronome on the beats the
view names.

### Data model (core)

`domain/practice_defaults.rs`:

```rust
pub struct PracticeDefaults {
    pub click: ClickStart,                 // EveryBeat until the musician changes it
    pub session_length_mins: Option<u16>,  // None until the musician sets one
}

pub enum ClickStart {
    EveryBeat,
    TwoAndFour,
}
```

`ClickStart::sounding(&self, metre: &Metre) -> u16` gives the starting mask
over the bar, least significant bit first. `EveryBeat` sounds every beat.
`TwoAndFour` sounds beats 2 and 4 only where the click sheet offers that
preset, an ungrouped bar of four or more beats; anywhere else (3/4, 2/4, and
6/8 or 7/8 as the sheet builds them, grouped) it falls back to every beat, so
the default never starts a click the bar cannot hold.

### Events and view

```rust
pub enum PracticeDefaultsEvent {
    Save(PracticeDefaults),   // the Profile screen's controls
    Loaded(PracticeDefaults), // the shell replaying the stored blob at launch
    Carried(Vec<u8>),         // the previous blob's bytes, sent once
}
```

- `Save` validates the session length, sets `model.practice_defaults`, emits
  `AppEffect::SavePracticeDefaults` and renders. An out-of-range length
  leaves the model alone, raises the error and writes nothing.
- `Loaded` sets the model and renders, never re-saving. A length outside the
  current range is clamped into it rather than refused: a later change to
  the range must not throw away the musician's choice.
- `Carried` reads the version 2 blob, which still held a repetition target,
  drops the target, clamps, and saves in the new shape (#2791).

What the defaults change:

- `ActiveSessionView.current_click_sounding` names the beats the click starts
  on for the current item, from the item's metre or 4/4 when it has none.
  `ClickController.reseed` reads it instead of choosing every beat itself.
  The default applies only when an item starts: a bar changed on the click
  sheet is the musician taking over, and starts on every beat (#1915,
  decided on review).
- A new session starts at the saved length (#1736).
- `ViewModel.practice_defaults` carries the saved value for the Profile
  screen.

### Persistence: a singleton beside the profile

The same fire-and-forget `AppEffect` route as the library sort and the
profile, with a versioned key and a Rust wire pin (offline-first invariant 8).

- **Write:** `Save` emits `AppEffect::SavePracticeDefaults`; `Store.swift`
  bincode-serialises it under `Store.practiceDefaultsKey`, named by
  `PracticeDefaults::BLOB_VERSION` (#2216).
- **Read:** at launch, beside the profile restore, the shell decodes the blob
  and sends `Loaded`. With no current blob it sends the previous one's bytes
  as `Carried`. Missing or undecodable is a no-op; the defaults stand.

**Not on the profile.** The saved profile is positional bincode; a new field
there makes every stored profile fail to decode, and the restore swallows the
failure, so every musician would lose their name, instrument and highlighter
on update without a word. The profile blob and its pin are untouched.

**Not in the crash-recovery blob.** Nothing inside `ActiveSession` holds a
default, so a change here never bumps that blob's version.

## Key decisions

1. **The core owns both defaults and applies them.** The shell does not
   choose the every-beat start.
2. **Their own singleton, not fields on the profile.** No profile is lost.
3. **The fallback lives in the core.** 2 and 4 in 3/4 or 6/8 starts on every
   beat; the shell never works that out.
4. **Clamp on restore, refuse on save.** A save outside the range is a
   mistake to show; a stored value outside it is a range that moved.
5. **On the Profile screen** (#1915 plan): the musician already goes there
   for the highlighter; no new Settings screen.

## Deliberately not doing

- Other click presets as a default (downbeat, group starts): the issue asks
  for every beat or 2 and 4.
- A default tempo. The click starts from the piece's own tempo.

## Testing

- **Core:** `Save` updates the model and emits the save; an out-of-range
  length is refused, keeps the last value and writes nothing; `Loaded` never
  re-saves and clamps into range; `Carried` drops the old target and saves
  once; the starting mask for every beat and 2 and 4 in 4/4, 5/4, 3/4, 2/4,
  6/8 and 7/8, and in 4/4 when the item has no metre; the wire bytes of a
  fixture pinned as hex; events, effect and view round-trip on the bincode
  wire (#846).
- **iOS:** the save writes under the versioned key and leaves the profile
  blob alone; restore replays `Loaded`, and is a no-op when nothing is
  stored; the Profile section's snapshots; the metronome starts on the core's
  mask.
- Bindings regenerated with `just ios-gen`, never hand-edited.
