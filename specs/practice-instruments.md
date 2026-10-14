# Practice instruments: the session timer, the rep counter, and an honest click

> Tier 3. Covers #1364 (session timer), #1367 (rep counter) and #1499 (which
> beats the click sounds, and the time signature behind it). Epic #1975.
> Design: [`practice-instruments/design/focus-player-instruments.dc.html`](practice-instruments/design/focus-player-instruments.dc.html).
> Rulings: design-principles **T19**.

## The rule

The click sounds only the beats the metre and the chosen pattern say, so it
never misstates the tempo. A pattern gates which beats sound; it never divides
the bpm.

## The three questions #1499 said must settle before code

### 1. Where the time signature lives

On the item. The chart reads it, so a second source of truth cannot exist.

```rust
pub struct Metre {
    pub beats: u8,                // numerator: 2 to 12
    pub unit: u8,                 // denominator: 2, 4 or 8
    pub groups: Option<Vec<u8>>,  // sums to `beats`; enforced in validation.rs
}
```

- `Item.metre: Option<Metre>`. `None` means the piece does not declare one.
- `ChordChart.metre` is removed.
- The live sheet's metre is a **session-local override** that never writes
  back to the item.

### 2. Whether `achieved_tempo` still means one thing

**Yes, and it takes two mechanisms to keep it that way.**

- **A pattern never divides the bpm.** The pulse grid stays the same rate;
  the pattern gates which beats sound. A click on beat 4 at 120 is still 120.
- **A denominator other than 4 changes the unit, so the core normalises.** In
  7/8 the row reads `♪ = 168`. `achieved_tempo` keeps its meaning of
  **crotchet BPM**, and the core converts: `crotchet_bpm = round(displayed * 4
  / unit)`, rounding half away from zero, i.e. `(displayed * 4 + unit / 2) /
  unit` in integer arithmetic.

  **The rounding rule is stated here on purpose.** For `unit: 8` every odd
  displayed value has a half-BPM to lose: `♪ = 169` must become 85, not 84.
  `unit: 2` and `unit: 4` are exact.

The shell reports what it displayed plus the click's state; the core rules and
normalises.

```rust
pub struct ClickState {
    pub metre: Metre,
    /// Bitmask over `metre.beats`, LSB = beat 1.
    pub sounding: u16,
}
```

### 3. Whether the click pattern is recorded alongside the tempo

**Yes.** "120 with the click on 2 and 4" is a different achievement from "120
straight". `SetlistEntry.click_pattern: Option<ClickState>` is written by the
same handler that writes `achieved_tempo`, and only when the click was
sounding.

## The rep counter

The counter is always on screen and can be ignored. A target is written when
the counter is first used, never at session start, so an untouched entry
records nothing. `DEFAULT_REP_TARGET` is 10; the `3...10` stepper lets a
musician ask for fewer. Each tap is timestamped (`RepEvent`).

## The click's band and presets come from the core (#2225)

The tempo band, step and default, the named bars, the groupings and the
patterns live in `crates/intrada-core/src/domain/metre.rs` and reach both
shells through `LimitsView`. The band is crotchets 40 to 208, default 96,
step 2, projected per beat unit through `Metre::displayed_bpm`. The beat
indicator derives the current beat from the same grid the audio uses (T19).

## Copy

`Repetitions`, `Got it`, `Not quite` (#1938). `Not quite right` at
accessibility sizes, where the pair stacks.

## Open questions

- **Compound metre display.** 6/8 renders as six cells grouped 3+3, but the
  app has no notion that a dotted crotchet is the felt beat. Showing `♩. = 56`
  alongside `♪ = 168` is a later refinement.
- **The trend's axis label.** Once a session can be practised in quaver units,
  the trend chart draws crotchet-normalised values while the player showed
  something else. The chart must name its unit; whether it should also offer
  the reading the user saw is not settled.
- **An accent timbre.** Out of scope: a second click sound is a different
  question from which beats sound at all.
