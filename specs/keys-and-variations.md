# Keys and variations: steps give way, and the record gets richer

> Tier 3 (a bridge shape, a GRDB migration and the practice in progress blob).
> Rides as the first commit of the core PR for #2246, step 2 of epic #50. It
> carries #2106 (a key is a real value) and #2107 (every tap keeps its tempo).
> Decisions: the plan comment on #2246 and Jon's answers of 2026-10-03. The
> target shape is the practice record target page linked from #50.

## Problem

An exercise has Steps, a ladder of typed labels. A key is text the core
guesses at (`is_key_label`), in three places with three shapes: a piece's
written key, a chord chart's key and a step. A play names one step at most,
and a piece has none. Taps keep no tempo, a tap past the target is dropped,
and a miss doubles as an undo, so the record loses what later advice needs.

No device holds steps (Jon, 2026-10-03), so nothing is moved across: steps
are retired, not migrated.

## Data shape

### A key is one value everywhere (#2106)

New in `domain/key.rs`, beside the wheel helpers:

```rust
pub struct Key { pub letter: Letter, pub accidental: Accidental, pub mode: Option<Modality> }
pub enum Letter { C, D, E, F, G, A, B }
pub enum Accidental { Natural, Sharp, Flat }
```

- **The spelling is the letter and accidental**, so C sharp stays C sharp.
  `Key::pitch_class()` is what counting and comparing use, so both spellings
  are one key (`Key::same_key`).
- **`mode: None` is a key written with no mode**; nothing guesses major.
- **One value in every place**: `Item.key: Option<Key>` replaces `Item.key`
  text plus `Item.modality`; `ChordChart.key: Option<Key>` replaces its text;
  `Item.keys: Vec<Key>` is the item's chosen list for practice; a play's key.
- **Reading text is the core's job.** `Key::parse` reads what musicians type
  ("Eb", "E flat major", "F#m" is refused, as today) and replaces
  `is_key_label` and the freeform healing in `wheel_selection`. The form
  inputs (`CreateItem`, `UpdateItem`) and the wheel send a `Key`.
- **Storage keeps its columns**: `item.key` and `item.modality` hold the
  spelling and mode; the codec converts through two core functions over the
  bridge (`key_from_stored`, `key_to_stored`), so Swift never parses. A
  stored key the core cannot read loads as `None` and its text is kept in the
  column untouched until the musician picks a key. The chart's key rides the
  chart JSON the same way.

### Variations are library-wide, like tags

`Variant` becomes `Variation` (closing #1771); `variant.rs` becomes
`variation.rs`:

```rust
pub struct Variation { pub id: String, pub label: String,
                       pub updated_at: DateTime<Utc>, pub deleted_at: Option<DateTime<Utc>> }
```

- **One list for the whole library.** A variation made on one exercise is
  offered on every item. Labels are unique among live rows, case-insensitive.
- **Each item keeps the set it uses**: `Item.variation_ids: Vec<String>`, in
  the order chosen, as tags are. Removing one from an item leaves it in the
  library with its history. Pieces and exercises alike.
- **Four built-ins** are ordinary rows the core seeds, with fixed ids so two
  devices agree: hands separately, dotted rhythms, back to front, three chord
  tones only. Renamed or deleted like any other. Seeded only into an empty
  table, so a deleted built-in stays deleted.
- **Deleting a variation tombstones it**: hidden from pickers, kept for plays.
- Retired with steps: `Item.variants`, `SetVariants`, `UpdateVariants`,
  `ladder_is_all_keys`, `shows_key_field`, `SOLID_SCORE_MIN` and the current
  step in Up next (`suggestion.rs`).

### A play, and its taps (#2107)

`VariationPlay` becomes `Play`:

```rust
pub struct Play {
    pub id: String,
    pub section_id: Option<String>,     // None: the whole piece
    pub key: Option<Key>,               // None: the written key, or no key
    pub variation_ids: Vec<String>,     // empty: plain
    pub started_at: DateTime<Utc>,
    pub seconds: u64,
    pub rep_target: Option<u8>,
    pub rep_count: Option<u8>,
    pub rep_history: Option<Vec<RepEvent>>,
    pub tempo_changes: Vec<TempoChange>,
    pub achieved_tempo: Option<u16>,
    pub click_pattern: Option<ClickState>,
    pub score: Option<u8>,
}
pub struct RepEvent { pub action: RepAction, pub at: DateTime<Utc>,
                      pub tempo: Option<u16>, pub click_sounding: Option<bool> }
pub enum RepAction { Missed, Success, Undo }   // Undo appended last
pub struct TempoChange { pub at: DateTime<Utc>, pub tempo: u16, pub click_sounding: bool }
```

- **Every tap keeps its tempo** in crotchets and whether the click sounded.
  A silent tap keeps its number; only a sounding one is evidence (T16). A
  setting that gives no crotchet tempo lands the tap with none, no message.
- **The count keeps going past the target** (Jon, 2026-10-03): got it adds
  one, uncapped (saturating at 255); a miss steps back one, floor 0, as now.
  `rep_target_reached` leaves the record; the view works it out.
- **Undo is its own action**: it reverses the last got it or miss on the
  count and is stored as `Undo`, so a correction is never a failure.
- **A tempo change is kept once settled.** Each change appends; one arriving
  under two seconds after the last, with no tap between, replaces it. A climb
  from 60 to 84 by stepper keeps where it rested, and the core needs no
  timer. Capped at `MAX_REP_HISTORY` (500), as taps are.
- **Target changes are not recorded**: the target is fixed once practice
  starts, the plan stays on the entry, and the taps show going past it.

### The entry and the session

- `SetlistEntry.planned_variation_id` becomes `planned_section_ids:
  Vec<String>` (at most one now, so segments in #2315 need no second shape)
  and `planned_variation_ids: Vec<String>`.
- `PracticeSession.capture_version: Option<u32>`, appended last. The core
  stamps `CAPTURE_VERSION` (1) on every session it saves; older rows are
  `None`.
- **The piece's own score** (`SetlistEntry::score_summary`) reads plain full
  run-throughs only: no section and no variations. A key does not exclude a
  play. Old plays carry neither, so history reads as before.

## Events

Changed shapes keep their slot; new ones append last (positional wire):

- `RepGotIt` and `RepMissed` gain `reading: TempoReading`; `RepUndo { now,
  reading }` and `TempoChanged { now, reading }` are new.
- `SwitchPlay { entry_id, section_id, key, variation_ids, now, reading }`
  replaces `SwitchVariation`; changing nothing writes nothing.
- `SetEntryPlan { entry_id, section_ids, variation_ids }` replaces
  `SetEntryVariant`.
- `ItemEvent::UpdateItemVariations { id, variation_ids, new_labels }` sets
  an item's set, minting library rows for new labels (an existing live label
  is reused, not duplicated); `UpdateKeys { id, keys }` sets its list.
  `VariationEvent::Rename` and `Delete` act on the library row.

A new play starts on the whole piece, plain, with no key: no current step and
no default order. Offering last time is #2249's, with the chip that shows it.

## Persistence

GRDB `v19_keys_and_variations`, additive:

```sql
CREATE TABLE variation (id TEXT PRIMARY KEY NOT NULL, label TEXT NOT NULL,
  updated_at TEXT NOT NULL, deleted_at TEXT);
ALTER TABLE item ADD COLUMN variation_ids TEXT;   -- JSON, null is empty
ALTER TABLE item ADD COLUMN keys TEXT;            -- JSON, null is empty
ALTER TABLE session ADD COLUMN capture_version INTEGER;
```

- `PersistenceOperation::LoadVariations` and `SaveVariations(Vec<Variation>)`,
  output `Variations`, all appended last, with their own list sync (#2067).
- The `variant` table stays, unread, under the append-only rule; its drop
  joins #2317. Old session rows' `variation_id` is ignored on decode.
- Android's in-memory store gains the variation list and regenerates its
  bindings; it holds no saved data yet.

## The practice in progress

`ActiveSession::BLOB_VERSION` goes 5 to 6, then the pin is re-pinned. A v5
blob is never read. The shell reports finding a retired key
(`Event::RetiredSessionFound`), deletes it, and the core shows: "A practice
left open before the update couldn't be picked up again."

## Out of scope

The item screen (#2247), section and variation pickers on the player and the
last time chip (#2249), merging duplicate variations, modes beyond major and
minor (#830), dropping the `variant` and coach-era tables (#2317).

## Two PRs

1. **Core, this spec first**: the types, events, views, `v19`, the codecs,
   both bindings, the bridge tests. The screens still build.
2. **Screens**: the key wheel and forms on the new `Key`, the undo control,
   the count past the target, tempo change wiring on the player, and the
   builder's plan, at their simplest until #2247 and #2249.

## Tests

- **Table tests from what musicians type**: `Key::parse` against every label
  `is_key_label` accepts and refuses today and every stored key text the
  wheel has written; enharmonic pairs are one key and keep their spelling;
  `key_from_stored` round trips every wheel spoke.
- **Variations**: a new label mints one row; a live label is reused across
  items; removing from an item keeps the row; deleting tombstones; the
  built-ins seed once and never return after deletion.
- **Taps**: sounding, silent and unusable readings; got it past the target
  counts on; undo reverses and is stored as undo; deleting the push past the
  target fails a test.
- **Tempo changes**: a climb inside two seconds keeps one; a tap settles one.
- **Scores**: a play with a variation or a section leaves the piece's score;
  a keyed plain play counts.
- **Bridge**: `Key`, a play with a key and two variations, each new event and
  operation, through `LiveBridge` (#846) and Android's `BridgeRoundTripTest`.
- **The blob**: the pin fails before the bump; a v5 blob never half restores.
- **iOS**: a database built at `v18_section` upgrades with every item's key,
  chart and session intact.
