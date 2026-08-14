# intrada Tone of Voice

> Started 2026-08-14 (issue #1359, decision 1a of the
> [v0.7.0 audit](audit-2026-08.md)). This is the writing layer of
> [`design-principles.md`](design-principles.md): that doc decides what a screen
> *does*, this one decides what it *says*. When they disagree, the design
> principle wins and this doc gets a new example.
>
> **Scope: every string a user can read.** Screen titles, subtitles, buttons,
> labels, empty states, errors, alerts, accessibility labels, placeholder text.
> Not comments, not docs, not commit messages.

## How to use this doc

- Writing a new string? Read *The stance*, then check your string against
  *The rules*. Most strings are settled by rules 1 to 4 alone.
- A rule fights a real screen? That is a decision, not a licence. Add it to
  *Open questions* rather than quietly writing around it.
- Reviewing copy? *The sweep checklist* at the bottom is the review pass.

---

## The stance

**intrada is a practice notebook, not a practice coach.** It holds what you
played and what you thought about it. It does not have opinions about your
playing, it does not cheer, and it does not have a personality that needs
expressing.

Three consequences, each one a design principle turned into a writing rule:

- **The app disappears during practice** ([§B](design-principles.md#b-simplicity--defer-complexity-dont-remove-capability)).
  On the player, words are as expensive as taps. Labels and numbers only.
- **Content over chrome** ([§C](design-principles.md#c-no-clutter--easy-to-reason-about)).
  The user's own pieces, sessions and scores are the content. App prose is
  chrome, so every sentence has to earn a place the user's data could have had.
- **One concept per screen** ([§C](design-principles.md#c-no-clutter--easy-to-reason-about)).
  A screen that needs a paragraph to explain itself is a design problem. Fix
  the screen, do not write a better paragraph.

The nearest thing to a house voice: **a good accompanist.** Present, precise,
never the thing you are listening to.

### Voice by surface

Adapted from [T13](design-principles.md#t13--voice-splits-by-surface-and-the-engine-never-speaks-on-screen),
which was retired with the coach but whose split survives the direction change.

| Surface | Job | Budget |
|---|---|---|
| **Chrome and admin** (headers, forms, library, builder, settings) | Name things | Labels, not sentences. A subtitle is facts, not a sales line. |
| **In session** (live view, player, rep counter) | Get out of the way | Numbers and one-word buttons. Prose only where nothing else works. |
| **Reflective** (session complete, reflection, item history) | Frame the user's own words | One short sentence at most; the user's quote carries the feeling. |

---

## The rules

### 1. British English, always

`practise` the verb, `practice` the noun. `prioritise`, `organise`, `colour`,
`favourite`, `cancelled`. Dates read `14 Aug`, never `Aug 14`.

Already right across the app (`Start practising`, `days practised`,
`Add to priorities`). It stays right by being checked.

### 2. A musician's words, not a data model's

Say what a musician would say at the piano. Where the core and the screen
disagree, the screen wins and the core keeps its own name.

| Core / spec says | Screen says |
|---|---|
| `variant` | Step |
| `entry`, `block` | the piece, the exercise, the group |
| `item type` | Piece / Exercise |
| `score`, `rating` | how it went |
| BPM, quarter note | `♩ = 132`, crotchet |
| measure | bar |
| `session in progress` | Pick up where you left off? |

Music notation belongs on screen where a musician would write it: `♩ = 132`,
`D♭ major`, `bars 12 to 14`. Do not spell out what notation says more briefly.

### 3. Name the thing, do not narrate the app

The user knows they are in an app and knows the library is theirs. Every word
spent saying so is a word not spent on their music.

- No possessives on chrome: `Library`, not `Your Library`. `Exercises`, not
  `Your exercises`.
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

The app does not know whether the session went well, so it must not say. A bad
practice is still a practice, and `Nice work.` after a session the user knows
was rotten reads as an insult by a machine that was not listening.

### 5. Punctuation

- **No em dashes and no double dashes.** The separator on screen is a middle
  dot with spaces: `Allegro · ♩ = 132`, `7:00 · 3 of 5`. It is already the app's
  house separator and it does the job an em dash was reaching for.
- **Ranges take `to`**, not a dash: `bars 12 to 14`.
- **Sentence case everywhere.** `Add piece or exercise`, not
  `Add Piece or Exercise`. Small caps eyebrows are a type token, not a
  capitalisation choice, so write them sentence case too and let `Eyebrow` style
  them.
- **No full stop on labels, buttons, titles or subtitles.** Full stops only on
  something that is genuinely a sentence (empty-state help, alert bodies).
- **Curly quotes for the user's words**, straight quotes never:
  `“Bars 12 to 14 rush every run past 88.”`
- **No ellipses** on buttons. A button that opens a sheet still names the action
  and nothing more.

### 6. Questions

A question mark is a promise that the user is about to decide something. Keep
the promise.

- **Fine**: `Discard this session?` on an alert; `How did Clair de Lune go?` on
  the reflection sheet. Both sit at a real decision.
- **Not fine**: a question as a screen title where there is no question, only a
  task. `Where do you want to focus?` above a list you build by tapping Add is
  the app pretending to interview the user. That is coach voice, and it is the
  register this whole doc exists to keep out.

### 7. Empty states

Three parts, in order, and nothing else: **what belongs here**, **what to do**,
and the action itself as a button where one exists. No apology, no explanation
of why it is empty.

Good, already shipped: `Your pieces and exercises will live here.` (fails
rule 3 on `Your`, right in every other way).

Never: `Nothing to see here`, `Looks like you haven't added anything yet!`.

### 8. Errors and destructive confirms

- Say what happened and what it means for the user's data, in that order.
  `Storage unavailable · changes this session won't be saved.` is the model.
- Never blame, never apologise, never expose a code.
- A confirm dialog names the consequence, not the mechanism: `This practice
  won't be saved.`, not `This will discard the session record`.
- Per [T6](design-principles.md#t6--reversible-by-default-vs-todays-confirm-sheets),
  most actions should not have a dialog at all. If you are writing confirm copy,
  first check the action is genuinely irreversible.

### 9. Length

Hard budgets, per surface:

- Screen title: **1 to 3 words.**
- Subtitle: **one line of facts**, no verb needed.
- Button: **1 to 4 words.**
- Empty state: **one sentence**, plus the button.
- Live view: **no prose at all** beyond labels and the item title.

If a string will not fit the budget, the screen is doing two things.

---

## Worked examples

The three the audit named. Options are for Jon's red pen; nothing here is
decided.

### Library header

Currently: title `Library`, subtitle `4 pieces · 7 exercises`.

The title is already right by rule 3. The flagged "Your Library" energy is the
*header as a whole* reading like a filing cabinet: a room name and an inventory
count, neither of which is about music. The subtitle is the lever, not the title.

- **a.** Keep both. It is honest, it is short, and the rows below carry the
  interest. *(Lean: this, unless the header is genuinely dead space.)*
- **b.** Keep `Library`, make the subtitle earn its line with something a
  musician would want at a glance: `Last added · Clair de Lune`, or the
  priorities count when any are starred.
- **c.** Title becomes the collection itself: `Pieces & exercises`, subtitle
  drops to counts only when filtered.

### Practice header and hero

Currently: title `Practice`, subtitle `12 sessions`, hero eyebrow `Today`,
hero title `A focused session`, hero caption `Tap to begin — one decision`.

Three separate problems:

- `A focused session` is the app describing its own intent. It is not a fact
  about the user and it says nothing (rule 3).
- `Tap to begin — one decision` breaks rule 5 (em dash) and rule 3 (it explains
  the interface to the person using it). The big circular play button is the
  affordance; a caption under it is chrome explaining chrome.
- The subtitle `12 sessions` is a lifetime total, which is trivia rather than
  orientation.

This is where [#1360](https://github.com/jonyardley/intrada/issues/1360) lands
(hero repurposed to last session plus one-tap start, subline becomes a relative
last-practised line). The wording to choose there:

- **a.** Hero title carries the real content, caption goes entirely:
  `Clair de Lune`, eyebrow `Last practised`, subline `Tuesday`. *(Lean: this.
  It is the user's data, in their words, and the button needs no caption.)*
- **b.** Eyebrow `Today`, title `Last practised Tuesday`, no caption.
- **c.** Keep an app-voice title but make it a fact: `Nothing since Tuesday`.

Header subtitle, independently: `Last practised Tuesday` beats `12 sessions`,
and beats both when there is nothing yet with `No sessions yet` (already
shipped, already right).

### Builder title

Currently: title `Build session`, subtitle the running summary
(`3 items · 25 min`) or `Editing`.

`Build session` is correct by every rule here: it names the task, it is two
words, it is sentence case, it does not narrate. The audit noted this title as
`Where do you want to focus?`, which is not in the codebase and never has been
(checked across all branches), so treat it as the register to avoid rather than
a string to replace. Rule 6 exists because of it.

- **a.** Leave it. *(Lean: this.)*
- **b.** `New session`, if `Build` reads like configuration rather than
  choosing what to play.

---

## The sweep checklist

For [#1359](https://github.com/jonyardley/intrada/issues/1359) step 2, and for
every PR that adds a string afterwards. Run it per string, not per screen.

- [ ] British spelling; `practise` verb / `practice` noun correct
- [ ] Uses the musician's word, not the core's (rule 2 table)
- [ ] No `Your`, no `View`/`Browse`/`Manage`, no app self-narration
- [ ] No banned word from rule 4; no exclamation mark; no `we`/`I`
- [ ] No em dash or double dash; separator is ` · `; ranges use `to`
- [ ] Sentence case; full stop only if it is a sentence
- [ ] A question mark only where the user is deciding
- [ ] Inside the rule 9 budget for its surface
- [ ] Accessibility label matches the visible string's tone (it is copy too)

---

## Open questions

For Jon's red pen. Each one changes strings across several screens, so they are
worth settling before the sweep rather than during it.

1. **`Nice work.` on session complete.** Rule 4 says the app must not judge a
   session it did not hear. But the screen currently pairs it with
   `Ended early.`, so removing it leaves an asymmetry. Options: drop both and
   lead with the facts; keep both as neutral state labels; replace with the
   session's own intention quoted back.
2. **Second person at all.** The app currently uses `you` in a few places
   (`What you played`, `Your intention`, `Pick up where you left off?`). Rule 3
   bans `your` on chrome but these are reflective surfaces where it may be
   right. Proposal: `you` is allowed in reflective copy only, banned on chrome
   and in session.
3. **`priorities` vs `starred`.** The library uses both concepts in one screen
   (`Add to priorities`, `Swipe a row to star it`). One word should win.
4. **`Score` as a word.** It means "how it went" here and "the printed music"
   to every musician alive. Rule 2 says prefer the musician's word, which
   argues for renaming the rating. Not free: it touches the ring, the selector
   and the analytics labels.
