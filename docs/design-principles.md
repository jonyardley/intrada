# intrada Design Principles

> Guiding principles, not hard gates. When two pull against each other, or
> against the app, decide it and log it under **Open tensions & decisions**.
> Code and issues cite the T numbers: keep each number and its meaning. Tokens
> live in `ios/Intrada/DesignSystem/Theme.swift` and `.claude/rules/ios-ui.md`;
> on-screen words in [`tone-of-voice.md`](tone-of-voice.md); the design process
> in [`design-workflow.md`](design-workflow.md).

---

## Visual principles

- **Paper, ink and one marker**: the highlighter is the only bright colour,
  interactive colour on paper is ink (T24, T27).
- **Type is colour-coded**: blue-grey Piece, sand Exercise, never colour alone
  (T3).
- **Warmth bias in semantics**: brick red danger, olive green success.
- **Grotesk for words, mono for figures; five sizes, no capitals** (T35).
- **Reuse before creating; extend, don't clone.** Hand-rolled copies of a
  primitive are the top source of visual drift.

---

## Interaction principles

The app's job is to get the musician *practising*, not operating software.
Every screen is the shortest honest path to what they came to do, except at
the two moments where stopping to think *is* what they came to do.

### A. Spend friction deliberately

- **Friction is a tool, not a defect.** *Bad friction* (admin, setup,
  navigation, configuration): remove it ruthlessly, and rethink any admin
  flow that grows steps. *Good friction* (an intention just before an item, a
  reflection just after it): keep it (T1).
- **One primary action per screen.** One obvious, visually dominant next step;
  secondary actions recede.
- **Defaults beat configuration.** Every flow completes with zero setup.
- **Direct manipulation where the platform allows.** Prefer acting on the thing
  itself (swipe, drag, tap-to-toggle) over opening a form, always with a
  visible path beside it on both apps (T4).

### B. Simplicity — defer complexity, don't remove capability

- **Progressive disclosure.** The common path first; advanced options behind a
  sheet, "More" or Settings.
- **Reversible by default; confirm only the irreversible.** Reversible actions
  are instant, optimistic and undoable. Confirm only genuinely destructive,
  non-recoverable actions.
- **The app disappears during practice.** Strip non-essential chrome. The
  musician operates the music, not the interface.

### C. No clutter — easy to reason about

- **Content over chrome.** Maximise the share of the screen showing the
  musician's own data.
- **Glanceable, not just readable.** Decode by pattern, not by reading every
  label.
- **Consistency is a simplicity tool.** The same gesture means the same thing
  everywhere; novelty is a cost the user pays.
- **One concept per screen.** A second unrelated section usually means a
  second screen or a sheet.

---

## Open tensions & decisions

Each entry: the ruling, its status and its issues. Numbers are never reused.

### T1: Where friction belongs, admin vs intention and reflection

**DECIDED 2026-05-31.** Setup, planning and admin are bad friction. Setting an
intention just before each item and reflecting just after it are good friction,
kept on purpose, provided they are not padded with admin. Resuming a known
routine stays short (#760).

### T2: Mid-session configurability vs "the app disappears during practice"

**DECIDED 2026-05-31.** Mid-session editing of duration, reps and focus stays
**one layer down**, revealed by a deliberate gesture, never persistent chrome.
The default live-session screen stays bare. Config is reachable, not resident.

### T3: Decode-by-colour vs accessibility

**DECIDED (revisit) 2026-05-31.** **Colour is always an accelerator, never the
sole carrier of meaning.** Every type signal also carries a shape or text cue
(dot, badge text, icon); `TypeBadge` meets this (T29).

### T4: Direct manipulation on both apps

**DECIDED 2026-05-31.** The principle is **"the most direct affordance the
platform offers"**, with one hard floor: **no action may be reachable *only*
through a hidden gesture.** Gestures (swipe-to-delete, long-press menu) are
accelerators on top of a visible, non-gesture path on both apps, such as the
action in the detail screen or a sheet, and offered to VoiceOver and TalkBack.
Audit: #761.

### T5: Content over chrome vs the visual identity

**DECIDED 2026-05-31.** Chrome earns its space on navigation and structural
surfaces. On content surfaces every element serves recognition, hierarchy or
navigation; the type bars stay because they encode type. Ornament only: cut it.

### T6 — Reversible-by-default vs today's confirm sheets

**DECIDED (principle) + audit pending, 2026-05-31.** **Recoverable actions:
optimistic plus undo toast; only genuinely irreversible actions: confirm
dialog.** New destructive actions default to the undo path unless the loss is
unrecoverable. Existing confirm sheets need a per-action audit (#761).

### T7: Reflection capture vs "the app disappears during practice"

**DECIDED 2026-07-14.** Capture never costs flow: **mid-item capture is one tap
from the player, saves, and returns straight to the running timer; it never
forces an advance.** End-of-session reflection is structured (what improved,
what is still broken, what to target next), shown against the intention,
skippable, never a gate on saving. What is captured feeds forward and stays
re-readable: **a note the musician can never see again is admin, not
reflection.**

### T8: Segmented pills are inputs, not a second view of a list

**DECIDED 2026-07-15 (#1081).** Pills are for genuine **inputs**: an either/or
choice (`KindSegment`) or tag selection. **Do not add a pill row as a filter
over content already shown as a list on the same screen.** Let the rows carry
it.

### T9: Session builder: direct manipulation always on, Edit is bulk remove only

**DECIDED 2026-07-15.** Every line (block header, nested exercise, standalone)
is its own List row: long-press reorders outside Edit, swipe removes, a tap
opens its settings sheet with "Remove from this session" (T4's floor). **Edit
is bulk remove only.** The "include today" toggle returns (#1101). A block
moves by its header row; its exercises follow on drop.

### T10 to T13: retired

Decided for the coach and retired with it on 2026-08-13 (#1344). The voice
rules from T13 live in [`tone-of-voice.md`](tone-of-voice.md).

### T14: The click starts on a tap and only then offers a tempo

**DECIDED 2026-08-19 (#1398).** An instance of T2. Tapping the tempo line
sounds a click at that tempo; only while it sounds does the row grow a
slower/faster pair. Stopping is the same tap.

- **The marking belongs to the target, not the click.** At rest the row reads
  "Andante · ♩ = 66"; once the tempo moves off the item's own it reads
  "♩ = 72" until the next item reseeds it.
- **The row always exists**, reading "Metronome" (V7, #1708) at a neutral 96
  when the item declares no number.
- The flat click and no-pulse rulings are superseded by T19.

### T15: The Practice hero carries the suggestion, not a play glyph

**DECIDED 2026-08-27 (#1082).** Where a suggestion exists it is the hero: the
piece, why, its items with their reasons, and one `Start · N min`. "Build my
own instead" sits under it as the secondary (amended by T22 and T32).

- **One primary action.** Never a second card that also starts a session.
- **Last practised keeps its line in the header subtitle** (T25).
- **Suggest, never gate.** With no suggestion the hero is the plain one. Every
  reason string is the core's.

### T16: A tempo is recorded when it was measured, never as a default

**DECIDED 2026-08-27 (#1420).** **A tempo is recorded only with evidence
behind it:** the musician moved the stepper, or the click was sounding when the
play closed. Anything else is a default, and a default is not data.

- **The sheet does not change.** The stepper still pre-fills; an untouched,
  unmeasured number is not kept, with no banner. Dialling a silent click is
  not evidence.
- **The shell reports, the core rules** what counts as evidence.
- **Per play (#1761).** Each play is stamped from what the click was doing when
  it closed; a skipped item keeps no tempo. Contract:
  [`specs/exercise-variations.md`](../specs/exercise-variations.md).

### T17: The tempo trend reports what was played, and shows where it did not

**DECIDED 2026-08-28 (#1420, #1425).** Follows T16.

- **A session with no measured tempo breaks the line** (never a zero), with a
  tick below the axis; `7 of 9 sessions measured` carries the honesty.
- **Fewer than two measurements is not a trend** (`TempoTrendView.has_trend`,
  the core's); nothing measured draws no card (the shell's).
- **The scale is relative, with an 8 BPM floor; the chip is neutral**, never
  a success colour. The declared target is not drawn.

### T18: On a piece, the exercise you write outranks the one the app suggests

**DECIDED 2026-08-28 (#1431).**

- **The musician's own exercise is the primary action and the card's only
  brand bar**; choosing an existing one is plain text beneath it.
- **Creating an exercise on a piece saves it already related.**
- **The chord-chart five are suggestions, offered second** (`From the chord
  chart`, never *curriculum*), in the Related exercises card, drawn as a
  control with the exercise tint because they create several library items.
- **The brand bar belongs to the empty state**; once the card holds exercises
  the action recedes to a footer.

### T19: The click can sound a subset of the bar, and then it must show the bar

**DECIDED 2026-09-02 (#1364, #1367, #1499;
[`specs/practice-instruments.md`](../specs/practice-instruments.md)).** Partly
reverses T14. Counting passes is practising, not configuration; metre, pattern
and target stay one or two layers down.

- **Screen budget.** The session timer sits in the top band as orientation,
  labelled apart from the item timer. **The pass counter is resident and
  ignorable**: untouched, it records nothing. The ring shrinks to about 200pt,
  the price of the resident counter.
- **A pattern never divides the tempo.** The grid keeps its rate; the pattern
  gates which beats sound.
- **The row never implies every beat.** Pattern dots, filled and hollow, never
  colour alone (T3).
- **The bar gets its own line under the tempo**, while sounding, as a sunken
  pill with a settings glyph (#2139) that opens the pattern sheet, so T14's
  start/stop tap is never contested.
- **The metre is offered with an answer already in it**, from the piece; the
  grouping (3+2+2) is the accent pattern.
- **A travelling ring on the dots, not a flash**, driven by the audio's clock,
  never a view timer that drifts.
- **A displayed target is not a recorded one**: `rep_count` and `rep_history`
  stay unset until the first tap. Reaching it is not congratulated (V1).
- **`Repetitions`, `Got it`, `Not quite`** (#1938).

### T20: The starred route is a text action under the hero, never a second button

**DECIDED 2026-09-05 (#981, #1540, #1544).** Reads T15 forward: a star glyph
and "Practise your priorities" in secondary ink under whichever hero shows.

- **It shows only from Idle** (`PracticeScreen.showsPriorities`).
- **Nothing starred, nothing offered**, derived before the library filter.
- **Two secondaries under one primary is fine; two primaries is not.**

### T21: Adding a piece captures what it can carry, in one pass

**DECIDED 2026-09-07 (#1390;
[`specs/one-pass-create.md`](../specs/one-pass-create.md)).**

- **The minimal form stays minimal.** Chart and exercises are two collapsed
  rows below the fields.
- **Nothing is written until Add.** Staged material is shell state; the core
  validates everything before it mutates anything.
- **What the form captures is the musician's own material, never the app's
  suggestions.** Generating the chart five at create time would be
  prescription wearing capture's clothes.
- **The chart sheet is extended, not cloned.**

Not settled: the edit path, per-bar parse feedback (#1387), the row wording.

### T22: "Build my own instead" opens the builder, and dismissal is never a one-way door

**DECIDED 2026-09-10 (#1617, #1618).** Amends T15.

- **The tap does the thing it names:** it hides the suggestion and opens the
  empty builder.
- **Not a one-way door.** The plain hero then carries "Show suggestion", a
  secondary text action like T20's. `suggestionDismissed` stays shell-local.

### T23: A weak photo read dims the glyph, never the label

**DECIDED 2026-09-11 (#1458).** "From the photo" stays at `inkSecondary` in
both states, because text needs 4.5:1. Only the glyph dims, to `inkFaintIcon`,
which clears the 3:1 floor for graphics. VoiceOver speaks the weak state.

### T24: One marker on paper replaces indigo and serif

**DECIDED 2026-09-11 (#1676).** Paper, white cards, ink, 3pt corners, a faint
lift, and the highlighter as the one bright colour. Indigo and serif titles are
retired.

- **Interactive colour on paper is ink**; the marker is a fill with ink on it.
  The hero and celebration toast are dark ink with the marker as accent.
- Type colours are duller than the marker, so a badge never reads as a button.
- The player stays light for now; a dark focus screen is #1679, and choosing
  your own marker is #1677.

### T25: The Practice greeting leads the subtitle

**DECIDED 2026-09-11 (#1690).** The subtitle under the Practice title reads
"Morning, Jon · Last practised Tuesday": last practised stays in the header
(T15). Morning is 04:00 to 11:59, afternoon 12:00 to 17:59, evening 18:00 to
03:59; the core owns the rule and the string (V6). The only way into the
profile is the musician's instrument icon on their highlighter, top right.

### T26: The reading scale sits at the platform's sizes, not below them

**DECIDED 2026-09-12 (#1723); sizes superseded by T35.** Reading sizes sit at
the platform defaults, not under them. **No size step is smaller than a
point**: a half-point change can render identically. Body text grows on the
body Dynamic Type curve.

### T27: Text ink drops most of its brown; the hero gradient keeps its warmth

**DECIDED 2026-09-13 (#1723).** `ink` and `inkSecondary` move to near-neutral,
both clearing AA with more margin. The hero's lower stop stays warm, a
deliberate two-tone read (now T33's). There is no terracotta accent: `accent`
is `ink`.

### T28: Header actions are native toolbar items; the page's own add and identity stay by the title

**DECIDED 2026-09-14 (#1868).** On iPhone, Cancel, Edit and the priority star
are system toolbar items with system type and tint, which gives them the glass
pill. The round "+" that adds to a page and the Practice profile badge stay
beside the large title as our own drawing. The bar is an opaque `paperTop`
fill with no seam. An iPhone ruling; Android follows T36.

### T29: Section, field and tag each have one look; the type badge is a tag

**DECIDED 2026-09-15 (#1876); looks amended by T35.** Three label kinds:

- **A section** (replaced by T35) is named inside its card, with a count as
  "· 3" and Edit at the trailing edge.
- **A field label** (replaced by T35) is quieter than its value.
- **A tag** is a pill, and the piece or exercise badge is the first pill in
  the same row, tinted by type. The badge is a pill wherever it appears, the
  player included: one shape means "what kind of thing this is".

### T30: The banner has two tones, and grey means it worked

**DECIDED 2026-09-22 (#1325).** `GlobalBanner` has two tones:

- **Danger**, for a refusal or failed write: `danger` on `dangerBanner`, the
  warning triangle, no success haptic.
- **Notice**, for something that worked but needs a word: `inkSecondary` on
  `cardFill`, a hairline underneath, the info circle; the success haptic stays.

Both can stand at once, error on top, each dismissed on its own.

### T31: Not played is said by a colour step, never a fade

**DECIDED 2026-09-24 (#2032, #2033).** Following T23, words stay readable and
only the glyph dims. The status line ("Saved for next time", "Not played") is
`inkSecondary` at full strength; the title steps from `ink` to `inkSecondary`;
the hollow dot is `inkFaintIcon`. The earlier score in a mastery change and
future days in the week strip take `inkSecondary`.

### T32: The hero carries today's plan, and Start plays it

**DECIDED 2026-10-01 (#57, #999).** Amends T15: with a preferred length the
suggestion is a plan of several blocks, and Start plays it in one tap.

- The eyebrow reads "Today's plan" with no minutes; the count line carries
  items and minutes.
- The lead block keeps its rows and reasons; later blocks are one line each.
- "Change it first" opens the builder with the plan in it, beside "Build my
  own instead"; with T20's action, three secondary text actions sit under one
  primary, never a second filled button.

### T33: The hero's ground follows the musician's colour

**DECIDED 2026-10-01 (#1877).** The Practice hero and Up next card fade from
ink to the highlighter's hue at 22% lightness, so the light text, reason lines
and highlighter button keep contrast for every swatch; a test holds the three
floors. Butter keeps the original brown.

### T34: The first run teaches by doing

**DECIDED 2026-09-24 (#2121).** A new musician learns by making their first
session.

- **One welcome, then the work**: one screen, set up a profile or skip, no
  tour. **One short splash ahead of it, once** (#2277), skippable.
- **No sample content** (T21); example content is #2115.
- **The Start here card ticks from the musician's own data**, never stored.
  Built means a session has started.
- **Musicians who already use the app never see the welcome**: it needs an
  empty library and history and no profile. The Start here card stays until a
  session is marked.

### T35: Five sizes, sentence case, and one place for a field's label

**DECIDED 2026-10-03 (#1881, settling #1729 and #1907).**

- **Five sizes.** 30 page title; 20 hero or key picker; 17 card titles, body,
  fields and emphasis; 15 section titles, field labels, secondary lines and
  buttons; 13 tabs, badges and chips. Weight and colour carry the rest. No
  screen passes a size; only the timer, tempo readouts and score and mastery
  numerals are sized by their dial or readout.
- **Every label looks the same.** Section titles, field labels and toggle
  titles are 15 point medium in secondary ink, sentence case
  (`IntradaFont.label`; `SectionTitle` and `FieldLabel`), at the top of what
  they name. Values are 17 point in full ink. The dark Practice and Up next
  cards keep light labels. Replaces T29's small capitals.
- **A field's label sits inside its card, above the value** (`FieldCard`,
  `FieldLabel`), on every form and sheet. A read-only fact row on a details
  screen keeps label left, value right.
- **Mono is for figures.** A clock, duration or count set alone; mixed words
  and numbers are Hanken with even-width figures.

### T36: Android behaves like Android, and looks like intrada

**DECIDED 2026-10-13 (#2599).** Review page: https://claude.ai/artifact/5djCUQ6eUx8EvAPirGQDXY.

On Android, intrada keeps its look and adopts the platform's behaviour. The
look is paper, ink, one highlighter, Hanken Grotesk and DM Mono, the five
sizes and the 3dp card; no Material colour, shape or type stands in for it,
and wallpaper colour never replaces the highlighter. Behaviour and structure
follow Android: back always does something, and predictive back shows where it
goes; a pushed screen has a back arrow and a modal form has a close cross,
with its one action on the right; a press ripples in ink; quick choices rise
as bottom sheets; a reversible removal offers Undo rather than asking first;
the keyboard's Next and Done move you on. When an iOS ruling such as T28 is
about where something sits or how it moves, it binds iOS only, and Android
takes the platform's answer drawn in intrada's look.

### T37: One catalogue says how every component looks on both apps

**DECIDED 2026-10-13 (#2620).**

Every component the screens are built from has one page in
`design/components/`: what it is for, when to use it and when not, its parts,
variants, states, behaviour, content and accessibility, with pictures from the
snapshot tests on iPhone and Android. The page comes first: a change to a
component changes its page, then the SwiftUI version, then the Compose one,
then both sets of pictures, in one PR. Where the apps differ, the page names
the one look both move to; where the audit found several looks for one job,
the page picks one. `just check` fails when a page and the code disagree on a
file, a symbol or a picture, and an Android gap passes only with a written
reason. The Claude Design system shows the same pages, published from the
repo and never edited there.
