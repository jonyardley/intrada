# Practice defaults: the repetition target and the metronome a session item starts with

> Tier 3 (a new bridge shape and a new saved singleton). Rides as the first
> commit of the core PR for #1915, part of #1974. Native iOS only.

## Problem

A repetition target always starts at 10 and the metronome always starts on
every beat. A musician who works to five clean repetitions, or always wants
the click on 2 and 4, changes both again on every item in every session.
Both are habits the musician keeps from item to item.

Two copies of the old choice sit in the wrong places. The core's
`DEFAULT_REP_TARGET` is a constant, and the every-beat start is chosen in
Swift (`ClickController.reseed`), against the rule that the shell makes no
domain decisions.

## Approach: two defaults the core owns, saved as their own singleton

The core holds the defaults, validates them, and applies them. The shell
writes one blob to UserDefaults when told to, replays it at launch, renders
the Profile screen's controls, and starts the metronome on the beats the
view names.

### Data model (core)

New module `domain/practice_defaults.rs`:

```rust
pub struct PracticeDefaults {
    pub rep_target: u8,     // 3 to 10; 10 until the musician changes it
    pub click: ClickStart,  // EveryBeat until the musician changes it
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
}
```

- `Save` validates the target with `validate_rep_target`, sets
  `model.practice_defaults`, emits `AppEffect::SavePracticeDefaults` and
  renders. An out-of-range target leaves the model alone, raises the error
  and writes nothing.
- `Loaded` sets the model and renders, never re-saving. A target outside the
  current range is clamped into it rather than refused: a later change to
  the range must not throw away the musician's choice.

What the defaults change:

- A play's target, when the item's own settings named none, is the default
  at the first tap (`plays.rs`), and the counter draws that many slots before
  it (`ActiveSessionView.current_rep_slots`).
- `LimitsView.rep_target_default` carries the musician's value, so the item
  settings sheet opens on it (#1512).
- `ActiveSessionView.current_click_sounding` names the beats the click starts
  on for the current item, from the item's metre or 4/4 when it has none.
  `ClickController.reseed` reads it instead of choosing every beat itself.
  The default applies only when an item starts: a bar changed on the click
  sheet is the musician taking over, and starts on every beat as today
  (#1915, decided on review). No screen writes an item's metre yet, so in
  practice every item starts in 4/4.
- `ViewModel.practice_defaults` carries the saved value for the Profile
  screen.

A target already recorded on a play, or set on an item in the builder, is
never rewritten by a later change to the default.

### Persistence: a fourth singleton beside the profile

The same fire-and-forget `AppEffect` route as the library sort and the
profile, with a versioned key and a Rust wire pin (offline-first invariant 8).

- **Write:** `Save` emits `AppEffect::SavePracticeDefaults`; `Store.swift`
  bincode-serialises it under `Store.practiceDefaultsKey`,
  `intrada.practice-defaults.v1`.
- **Read:** at launch, beside the profile restore, the shell decodes the blob
  and sends `Loaded`. Missing or undecodable is a no-op; the defaults stand.

**Not on the profile.** The saved profile is positional bincode; a new field
there makes every stored profile fail to decode, and the restore swallows the
failure, so every musician would lose their name, instrument and highlighter
on update without a word. The profile blob and its pin are untouched.

**Not in the crash-recovery blob.** Nothing inside `ActiveSession` changes:
a play's target is still `None` until the first tap, so no blob version bump.

## Key decisions

1. **The core owns both defaults and applies them.** The shell stops
   choosing the every-beat start.
2. **Their own singleton, not fields on the profile.** No profile is lost.
3. **The fallback lives in the core.** 2 and 4 in 3/4 or 6/8 starts on every
   beat; the shell never works that out.
4. **The default applies at the first tap, not when the item opens.** Same
   moment as today's constant; nothing in the recovery blob changes.
5. **Clamp on restore, refuse on save.** A save outside the range is a
   mistake to show; a stored value outside it is a range that moved.
6. **On the Profile screen** (#1915 plan): the musician already goes there
   for the highlighter; no new Settings screen.

## Deliberately not doing

- Session length and the flow for a day that is different (#1736).
- Other click presets as a default (downbeat, group starts): the issue asks
  for every beat or 2 and 4.
- A default tempo. The click starts from the piece's own tempo, as today.
- Screens: the Profile section and the metronome reseed are the screens PR.
  The save, the restore at launch and the preview fixtures ride with the core
  PR, as the profile's did, since the shell's effect switch must stay
  exhaustive for the app to build.

## Testing

- **Core, test-first:** `Save` updates the model and emits the save; a
  target of 2 or 11 is refused, keeps the last value and writes nothing;
  `Loaded` never re-saves and clamps 0 and 200 into range; a new play's
  first tap takes the default, and a builder target beats it; the counter's
  slots follow the default; the limits' default follows it; the starting
  mask for every beat and 2 and 4 in 4/4, 5/4, 3/4, 2/4, 6/8 and 7/8, and in
  4/4 when the item has no metre; the wire bytes of a fixture pinned as hex,
  tail variant included; events, effect and view round-trip on the bincode
  wire (#846).
- **iOS, core PR:** the save writes under the versioned key and leaves the
  profile blob alone; restore replays `Loaded`, and is a no-op when nothing
  is stored; a live-bridge test carries a saved value to the counter, the
  click mask and the limits.
- **iOS, screens PR:** the Profile section's snapshots; the metronome starts
  on the core's mask.
- Bindings regenerated with `just ios-gen`, never hand-edited.
