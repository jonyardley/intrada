# Sections: the parts of a piece or exercise, and its trouble spots

> Tier 3 (a bridge shape and a GRDB migration). Rides as the first commit of
> the core PR for #2245, step 1 of epic #50. Both shells regenerate bindings.

## Problem

A piece is one block. A musician cannot say "A1, B, A2", or mark bars 12 to 14
as the spot that keeps going wrong, so nothing can say which part needs work.
Every later step in #50 points at a section (#2246 to #2249). This step gives
pieces and exercises an ordered list of sections in the core and on disk, and
nothing on screen.

A section is library data. How it is going is worked out from plays when
read, never stored on it (`docs/practice-record-target.md`, #2321).

## Data shape

New module `domain/section.rs`, held on the item like its variations today:

```rust
pub struct Section {
    pub id: String,                    // ulid; plays and links point here
    pub name: String,                  // "A1"; may be empty when bars are set
    pub bars: Option<BarRange>,
    pub kind: SectionKind,
    pub target_bpm: Option<u16>,       // None: the piece's own tempo
    pub position: usize,               // score order, display only
    pub updated_at: DateTime<Utc>,     // per-row LWW (offline-first)
    pub deleted_at: Option<DateTime<Utc>>,
}
pub struct BarRange { pub first: u16, pub last: u16 } // "bar 12" is 12..12
pub enum SectionKind { Form, TroubleSpot }
```

`Item` gains `sections: Vec<Section>`, appended last with `#[serde(default)]`,
tombstones included, exactly as `variants` is carried today. Pieces and
exercises both have sections.

- **A section needs a name or bars, or both.** A spot added as "bars 12 to 14"
  has no name; the view labels it "Bars 12 to 14". The label is derived, not
  stored, so editing the bars keeps the label true.
- **Duplicate names are allowed.** Rondo form repeats A; two "A" rows with
  different bars are two sections. Rows reconcile by id only.
- **The target tempo is stored as given.** None means "the piece's tempo". The
  core does not clear a target that happens to equal the piece's, because the
  piece's tempo can change later.
- **Removal always tombstones.** Nothing names a section yet, but #2246 will,
  and a hard delete now would be a second rule to unpick. A tombstoned section
  stays on `Item.sections`, hidden from every view.

Chart sections (`ChartSection` in `domain/chart.rs`) stay apart, with no rule
joining them (decision 11 on #50, 2026-10-03). A bar range is not checked
against a chord chart's length.

## Bar ranges

`validation::parse_bar_range(raw: &str) -> Result<Option<BarRange>, LibraryError>`
reads what a musician types or says: "1-16", "1 to 16", "bars 5 to 12",
"bar 12", "bb. 5-12", "mm. 5-12", or a bare "12". It accepts a hyphen, an en
dash (iOS smart punctuation turns "1-16" into one) and "to". Blank is `None`.
It refuses a reversed range ("16-1"), bar 0, a half range ("1-"), more than one
range ("1-16, 20-24"), words around the range ("12-14 left hand") and anything
past `MAX_BAR`. Refusal names the field `sections` and maps to a new
`FormErrorField::Sections`, appended last.

New limits in `validation.rs`: `MAX_SECTION_NAME` 100 characters, `MAX_BAR`
9999. The target tempo reuses the piece's bpm bounds. The number of sections
has no cap: a long piece can hold many trouble spots (Jon, 2026-10-03).

## Events

One variant appended last on `ItemEvent` (the wire is positional):

```rust
UpdateSections { id: String, sections: Vec<SectionEdit> },

pub struct SectionEdit {
    pub id: Option<String>,   // None: a new row
    pub name: String,
    pub bars: BarsInput,
    pub kind: SectionKind,
    pub target_bpm: String,   // typed text, blank is None, as TempoInput
}
pub enum BarsInput { Blank, Picked { first: u16, last: u16 }, Typed(String) }
```

- **`UpdateSections`** is the item screen's whole list in one write, the
  pattern `UpdateVariants` set in #1783: add, rename, bars, kind, target
  tempo, reorder and remove land together. A row whose id names a section
  claims it; a row with no id is new; a live section the list leaves out is
  tombstoned. The whole write is refused on the first invalid row, nothing
  saved. A list identical to what is stored writes nothing and leaves
  `updated_at` alone.
It validates in the core and persists through `SaveItem`. Marking a spot
mid-practice (`AddTroubleSpot`) moved to #2249, the screen that sends it:
code with no reader is not built ahead (#1176; Jon, 2026-10-03).

## View

`LibraryItemView` gains `sections: Vec<SectionView>`: live rows only, by
position.

`SectionView` carries `id`, `name`, `kind`, `target_bpm`, `first_bar` and
`last_bar` (to prefill the picker), `label` (the name, else "Bars 12 to 14"
or "Bar 12") and `bars_caption` ("Bars 1 to 16", beside a named section).

`LimitsView` gains `section_name_max` and `bar_max`, so the
item screen's controls take their bounds from the core. Scores per section
are #2250's and are not added here.

## Persistence and migration

GRDB migration `v18_section`, additive:

```sql
CREATE TABLE section (id TEXT PRIMARY KEY NOT NULL, item_id TEXT NOT NULL,
  name TEXT NOT NULL, bar_first INTEGER, bar_last INTEGER, kind TEXT NOT NULL,
  target_bpm INTEGER, position INTEGER NOT NULL, updated_at TEXT NOT NULL,
  deleted_at TEXT);
CREATE INDEX index_section_on_item_id ON section(item_id);
```

- **Old rows read unchanged.** An upgraded install has an empty table, so
  every item loads with no sections; no existing column or row is touched.
- **Writes ride the item's transaction.** `LibraryStore.swift` upserts each
  section row beside the item row, keyed by id, with no delete-missing: the
  core carries its tombstones and writes them back, as it does for `variant`
  (#1113). Loads read tombstones too, so the core owns reconciliation.
- **No new persistence operation:** `Item` carries its sections.
- **Android:** the in-memory store needs only regenerated bindings; the
  androidx.sqlite store (`specs/android-shell.md`) creates `section` at birth.
- **The practice in progress blob is untouched:** `ActiveSession` holds no
  `Item`, so `BLOB_VERSION` stays at 5.

## What crosses the bridge

`Section`, `BarRange`, `SectionKind`, `SectionEdit`, `BarsInput`,
`SectionView`, the new `Item` and `LibraryItemView` fields, the new
`ItemEvent` variant, `FormErrorField::Sections` and the two limits. All
appended last; no JSON-only serde attributes (#846). Swift and Kotlin bindings
are regenerated, never edited.

## Room for what follows, without building it

- **#2246:** a play gains `section_id: Option<String>`; tombstoned sections
  keep old plays resolvable. Up next and the practice screen start on the whole
  piece, played plain, in the written key, and offer last time (answering
  #1501's question on #50); with no play naming a section yet, this step
  changes neither.
- **#2247:** `UpdateSections` and `SectionView` cover the item screen.
- **#2248:** a link table references `Section.id`; a tombstoned section's links
  hide with it.
- **#2249:** adds `AddTroubleSpot`, a nameless `TroubleSpot` placed in score
  order, sent from the practice screen.
- **#2307:** reuses `parse_bar_range` to offer bars found inside a longer note.
- **#2315:** segments are a list of section ids with minutes.

## Out of scope

Screens (#2247, #2249); plays naming a section, keys, variations (#2246);
exercise links (#2248); section scores (#2250); bars inside a full sentence
(#2307); joining chart sections; seed data, which arrives with #2247.

## Two PRs

1. **Core, this spec first:** types, events, validation, view, `v18_section`,
   `LibraryStore.swift`, `ItemCodec.swift`, fixtures, both bindings, the
   bridge test. Nothing visible changes.
2. **Screens:** #2247, after #2246's core (order inside v0.16 in
   `docs/practice-record-target.md`).

## Tests

- **Core, test-first:** a table test of `UpdateSections` with typed bars
  against `parse_bar_range`, from inputs a musician types:
  "1-16", "1 - 16", `"1\u{2013}16"`, "1 to 16", "bars 5 to 12", "Bars 5-12",
  "bar 12", "12", "bb. 5-12", "mm. 5-12", blank and spaces (no bars);
  refused: "16-1", "0-4", "1-", "-4", "1-16, 20-24", "12-14 left hand",
  "five to twelve", "10000", "1.5-3".
- Add, rename, rebar, change kind and target, reorder and remove in one
  `UpdateSections`; a swap of two names keeps each id; a removed row
  tombstones; a refused row saves nothing; an identical list emits no
  `SaveItem`; a nameless row with no bars is refused.
- The view hides tombstones, orders by position, and derives "Bar 12" and
  "Bars 12 to 14". Deleting the line that filters tombstones must fail a test.
- **Bridge:** `UpdateSections` and an `Item` with sections round-trip through
  `LiveBridge` (#846).
- **iOS, core PR:** upgrading a database built at `v17_item_metre` loads every
  item unchanged with no sections; a saved section and a tombstone survive a
  reload; `just ios-test` passes.
