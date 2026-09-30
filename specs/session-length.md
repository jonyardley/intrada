# Session length: a preferred length, and today's length in the builder

> Tier 3 (a bridge shape and a change to a saved singleton). Rides as the
> first commit of the core PR for #1736, part of #1974. Native iOS only.

## Problem

A practice session assumes nothing about how long it runs. A musician with
twenty minutes before work and one with a free hour build the same way, and
neither was asked. How long you have is the biggest thing shaping a session,
and it is knowable in one question; asking it every time is friction, and
never asking it builds the session for someone else.

The repetition default from the same issue shipped in #1915
(`specs/practice-defaults.md`); this spec is the session length and the way to
say today is different.

## Approach: a preferred length on the practice defaults, today's on the build

The core holds the preferred length with the other practice defaults, copies
it onto every new session being built, lets the builder change it for today,
and says how the planned entries sit against it. The shell renders the
Profile control and the builder's length control, and saves the defaults blob
when told to.

### Data model (core)

```rust
pub struct PracticeDefaults {
    pub rep_target: u8,
    pub click: ClickStart,
    pub session_length_mins: Option<u16>, // None until the musician sets one
}

pub struct BuildingSession {
    pub entries: Vec<SetlistEntry>,
    pub length_mins: Option<u16>, // today's length, from the preference
}
```

A length is 10 to 120 minutes in 5 minute steps
(`validation::validate_session_length`). `None` means no length: the builder
looks as it does today. There is no made-up starting length; the stepper's
first value when the musician switches a length on is
`LimitsView.session_length_default_mins`, 30.

### Events

- `PracticeDefaultsEvent::Save` validates the length beside the repetition
  target. A length out of range or off the step refuses the whole save, raises
  the error and writes nothing.
- `PracticeDefaultsEvent::Loaded` clamps a stored length into the range and
  rounds it to the nearest step, as it clamps the target; `None` stays `None`.
- `SessionEvent::SetSessionLength { length_mins: Option<u16> }` sets today's
  length. Building phase only, validated the same way. It never touches the
  preference.
- Every way into the builder (`StartBuilding`, `StartBuildingWith`, the Up
  next CTA, the priorities CTA) starts with today's length copied from the
  preference. A later change to the preference leaves a build in progress
  alone.

### View

`BuildingSetlistView` gains:

- `length_mins: Option<u16>`, the control's value.
- `length_summary: Option<String>`, `None` without a length. With one:
  "30 min today" when no entry has a planned duration, otherwise
  "20 of 30 min planned", counting only entries with a planned duration,
  minutes rounded down. Over the length it reads "35 of 30 min planned": no
  warning colour and no refusal, since the length is a guide.

`LimitsView` gains `session_length_min_mins`, `session_length_max_mins`,
`session_length_step_mins` and `session_length_default_mins`, so both
steppers draw their bounds from the core.

### Persistence

The defaults blob gains the length, so its shape changes. `Store.swift`'s key
moves from `intrada.practice-defaults.v1` to `v2`, and the Rust pin is
re-pinned. A `v1` blob is left unread: it merged on 2026-09-30 (#2173) and is
on no tester's phone, so the only cost is a developer's phone starting its
repetition and click defaults again once. The profile blob is untouched.

**Not in the crash-recovery blob.** Today's length lives on
`BuildingSession`, which is never saved; `StartSession` does not carry it
onto `ActiveSession`, so `BLOB_VERSION` stays as it is.

## Key decisions

1. **Today is different is an adjustment inside the builder** (#1736 plan,
   decision 1a). A step before the builder costs a tap every session.
2. **Off until set.** No length nobody chose.
3. **On the practice defaults, under a new key.** Rejected: a third saved
   value beside it, since nothing shipped reads `v1`.
4. **Only planned entries count.** An entry with no planned duration has no
   honest number to add.
5. **A guide, never enforced.** Over the length is shown, not refused.

## Deliberately not doing

- Filling the setlist to fit the length (#57).
- The Practice hero's plan line (#999).
- Storing the length on a finished session, or showing time left in the
  Focus Player.
- Screens: the Profile control and the builder's length control are the
  screens PR. The key bump, the preview fixtures and the bridge test ride with
  the core PR, since the shell must still build.

## Testing

- **Core, test-first:** every way into the builder starts at the preference,
  and at `None` without one; `SetSessionLength` changes today and leaves the
  preference alone, and is refused outside building, out of range and off
  the step; a later `Save` leaves a build in progress alone; `Save` refuses a
  bad length and writes nothing; `Loaded` clamps 5 and 300 and rounds 32;
  the summary line with no planned entries, some planned, some unplanned, and
  over the length; the limits; the re-pinned wire bytes; events round-trip on
  the bincode wire (#846).
- **iOS, core PR:** the save writes under the `v2` key; a live bridge test
  carries a saved length to the builder's value and summary.
- **iOS, screens PR:** snapshots of the Profile control and the builder with
  a length, and without.
- Bindings regenerated with `just ios-gen`, never hand-edited.
