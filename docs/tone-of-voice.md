# intrada Tone of Voice

> The writing layer of [`design-principles.md`](design-principles.md): that doc
> decides what a screen *does*, this one decides what it *says*. When they
> disagree, the design principle wins and this doc gets a new example (#1359).
>
> **Scope: every string a user can read**, accessibility labels and placeholder
> text included. Prose around the code follows [`style-guide.md`](style-guide.md).

## How to use this doc

Check a new string against *The rules*; rules 1 to 4 settle most. A rule that
fights a real screen is a decision: append it to *Decisions*. *The sweep
checklist* is the review pass.

---

## The stance

**intrada is a practice notebook, not a practice coach.** It holds what you
played and what you thought about it. It has no opinions about your playing,
does not cheer, and has no personality to express: **a good accompanist**,
present, precise, never the thing you are listening to.

- **The app disappears during practice** ([§B](design-principles.md#b-simplicity--defer-complexity-dont-remove-capability)):
  labels and numbers only on the player.
- **Content over chrome** ([§C](design-principles.md#c-no-clutter--easy-to-reason-about)):
  every sentence of app prose takes a place the user's data could have had.
- **One concept per screen** ([§C](design-principles.md#c-no-clutter--easy-to-reason-about)):
  a screen that needs a paragraph to explain itself is a design problem. Fix
  the screen, do not write a better paragraph.

### Voice by surface

| Surface | Job | Budget |
|---|---|---|
| **Chrome and admin** (headers, forms, library, builder, settings) | Name things | Labels, not sentences. A subtitle is facts, not a sales line. |
| **In session** (live view, player, rep counter) | Get out of the way | Numbers and one-word buttons. Prose only where nothing else works. |
| **Reflective** (session complete, reflection, item history) | Frame the user's own words | One short sentence at most; the user's quote carries the feeling. |

---

## The rules

### 1. British English, always

`practise` the verb, `practice` the noun. `prioritise`, `organise`, `colour`,
`favourite`, `cancelled`. Dates read `14 Aug`, never `Aug 14`, by localising,
not by hard-coding the British order (V5).

### 2. A musician's words, not a data model's

Say what a musician would say at the piano. Where the core and the screen
disagree, the screen wins and the core keeps its own name.

| Core / spec says | Screen says |
|---|---|
| `variant` | Variation |
| `entry`, `block` | the piece, the exercise, the group |
| `item type` | Piece / Exercise |
| `score`, `rating` (the 0 to 10) | Mark ([V4](#v4--the-screen-says-mark-the-core-keeps-score)) |
| `click` (the `Click*` types) | Metronome ([V7](#v7-metronome-never-click)) |
| BPM, quarter note | `♩ = 132`, crotchet |
| measure | bar |
| `session in progress` | Pick up where you left off? |

Music notation belongs on screen where a musician would write it: `♩ = 132`,
`D♭ major`, `bars 12 to 14`. Do not spell out what notation says more briefly.

### 3. Name the thing, do not narrate the app

- No possessives on chrome: `Library`, not `Your Library`. `Exercises`, not
  `Your exercises`. Reflective surfaces are the exception, where the possessive
  points at the user's own words: see [V2](#v2--second-person-is-for-the-users-own-words-only).
- No verbs of navigation: never `View`, `Browse`, `Manage`, `Explore`.
- No screen introducing itself: never `This is where you...`.
- Buttons name the action and its object: `Add piece or exercise`,
  `Save session`. Not `Continue`, not `Get started`, not `Let's go`.

### 4. No AI-isms and no cheerleading

Banned outright, in any string:

- **Enthusiasm**: `Awesome`, `Great job`, `Well done`, `You're on fire`,
  `Nice work`, `Let's`, `Ready to`, `Time to`, any exclamation mark.
- **Marketing**: `seamlessly`, `effortlessly`, `powerful`, `unlock`, `elevate`,
  `curated`, `journey`, `empower`, `simply`, `just`.
- **Fake companionship**: `we` for the app (`We couldn't find...`), `I`,
  `Oops`, `Uh oh`, `Sorry about that`.

The app does not know whether the session went well, so it must not say
([V1](#v1--the-app-never-says-how-the-session-went)).

### 5. Punctuation

- **No em dashes and no double dashes.** The separator on screen is a middle
  dot with spaces: `Allegro · ♩ = 132`, `7:00 · 3 of 5`.
- **Ranges take `to`**, not a dash: `bars 12 to 14`.
- **Sentence case everywhere.** `Add piece or exercise`, not
  `Add Piece or Exercise`. Section titles and field labels too: nothing on
  screen is set in capitals (T35 in `design-principles.md`).
- **No full stop on labels, buttons, titles or subtitles.** Full stops only on
  something that is genuinely a sentence (empty-state help, alert bodies).
- **Curly quotes for the user's words**, straight quotes never.
- **No ellipses** on buttons, even one that opens a sheet.

### 6. Questions

A question mark is a promise that the user is about to decide something. Keep
the promise.

- **Fine**: `Discard this session?` on an alert; `How did it go?` on the
  reflection sheet, with the piece name underneath it as a subtitle. Both sit
  at a real decision.
- **Not fine**: a question as a screen title where there is no question, only a
  task, such as `Where do you want to focus?` above a list. That is coach voice.

### 7. Empty states

Three parts, in order, and nothing else: **what belongs here**, **what to do**,
and the action itself as a button where one exists. No apology, no explanation
of why it is empty.

Never: `Nothing to see here`, `Looks like you haven't added anything yet!`.

### 8. Errors and destructive confirms

- Say what happened and what it means for the user's data, in that order.
  `Storage unavailable · changes this session won't be saved.` is the model.
- Never blame, never apologise, never expose a code.
- A confirm dialog names the consequence, not the mechanism: `This practice
  won't be saved.`, not `This will discard the session record`.
- Per [T6](design-principles.md#t6--reversible-by-default-vs-todays-confirm-sheets),
  most actions have no dialog: check the action is genuinely irreversible first.

### 9. Length

Hard budgets, per surface:

- Screen title: **1 to 3 words.**
- Subtitle: **one line of facts**, no verb needed.
- Button: **1 to 4 words.**
- Empty state: **one sentence**, plus the button.
- Live view: **no prose at all** beyond labels and the item title.

If a string will not fit the budget, the screen is doing two things.

An explanatory caption is allowed on a form, where the user is deciding and the
app's vocabulary is in the way: the Piece/Exercise captions `Repertoire to learn
and keep up` and `Drills and studies to build technique` (#1361). Never in
session, and never where a better label would do the work.

---

## The sweep checklist

For every PR that adds or changes a string. Run it per string, not per screen.

- [ ] British spelling; `practise` verb / `practice` noun correct
- [ ] Uses the musician's word, not the core's (rule 2 table)
- [ ] No `Your` on chrome or in session (V2 allows it on reflective surfaces);
      no `View`/`Browse`/`Manage`; no app self-narration
- [ ] No banned word from rule 4; no exclamation mark; no `we`/`I`
- [ ] No em dash or double dash; separator is ` · `; ranges use `to`
- [ ] Sentence case; full stop only if it is a sentence
- [ ] A question mark only where the user is deciding
- [ ] Inside the rule 9 budget for its surface
- [ ] Accessibility label matches the visible string's tone (it is copy too)

---

## Decisions

Append here rather than deciding silently, as
[`design-principles.md`](design-principles.md) keeps its T-log.

### V1 — The app never says how the session went

**Decided 2026-08-14 (Jon, #1359).** `Nice work.` goes, and so does its partner
`Ended early.`; the summary leads with the facts it actually has (duration,
items done, the scores the user entered).

The app's entire knowledge of a session is what the user typed into it.
Congratulating someone who scored every item 2 is the notebook claiming to have
listened. Dropping only `Nice work.` would leave `Ended early.` reading as a
rebuke by contrast, which is why both go together.

### V2 — Second person is for the user's own words only

**Decided 2026-08-14 (Jon, #1359).** Refines rule 3's first bullet.

- **Reflective surfaces**: `you` and `your` allowed: `Your intention`,
  `What you played`, `Pick up where you left off?`.
- **Chrome and in session**: banned, as in `Your exercises`.

The test is whether the possessive points at content the user authored. If it
points at a screen, cut it.

### V3 — `priorities`, never `starred`

**Decided 2026-08-14 (Jon, #1359).** `priorities` wins because it names what
the user meant; the star is the glyph that shows it, not the word for it.

### V4 — The screen says `Mark`, the core keeps `score`

**Decided 2026-08-14 (Jon, #1359).** Rule 2 applied to the 0-to-10 rating.
`Score` means the printed music to a musician; `Mark` is what an exam board
gives a performance. Per rule 2 the core keeps `score` in its types, events
and columns, so **this is a copy change with no FFI, no migration and no bridge
surface**. Swift view names (`ScoreRing`, `ScoreSelector`) are shell-local and
may be renamed for consistency or left alone; neither is load-bearing.
Rejected: `Grade` (ABRSM levels) and `Rating` (the data model's word).

### V5 — Dates read `14 Aug` by localising, not by pinning

**Decided 2026-08-29 (#1485).** Every short date goes through `DateDisplay`,
built with `setLocalizedDateFormatFromTemplate`: a British device gets `14 Aug`,
an American one `Aug 14`. **Do not "fix" a template back to a literal pattern to
make a snapshot read British.** Snapshot references show `Aug 14` because the
host pins `en_US`. Pinning the snapshot host to `en_GB` instead would make the
references read the way Jon's device does, at the cost of re-recording every
date-bearing reference; left open.

### V6: The Practice greeting is the one sentence in a subtitle

**Decided 2026-09-11 (Jon, #1690).** Rule 9 says a subtitle is one line of
facts. The Practice header is the exception: it opens with a greeting and the
fact that was there stays after it, `Morning, Jon · Last practised Tuesday`.
One of `Morning`, `Afternoon` or `Evening` by the time of day (the bands are in
design-principles T25), then the name the musician typed. Never `Good
morning`, never an exclamation mark, never anything about practice itself.

With no name set there is no greeting, and the subtitle reads
`Last practised Tuesday`, or `No sessions yet`. Only Practice greets; every
other header stays facts.

### V7: `metronome`, never `click`

**Decided 2026-09-13 (#1708).** `click` is studio shorthand; rule 3 says name
the thing. The code keeps `Click` in its identifiers.

### V8: The welcome has no maker's note

**Decided 2026-10-02 (Jon, #2116).** The welcome speaks in the app's own
voice: rule 4 holds, no `I`, no `we`. One plain line says what the app is for,
and the button reads `Set up profile`, never `Get started` (rule 3).
