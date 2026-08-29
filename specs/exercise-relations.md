# Exercise relations: one "Used in" list

> Issue: [#1363](https://github.com/jonyardley/intrada/issues/1363). Design
> exploration outcome. Tier 3 (FFI bridge contract change), no DB migration.
> Mock: [`specs/exercise-relations/design/used-in.html`](exercise-relations/design/used-in.html).

## Problem

An exercise is its own thing. Many exercises serve several pieces (chord shells,
scales in the key of whatever is on the stand), many serve none at all (Hanon,
arpeggios, licks), and the same drill moves between those states over its life.
The exercise detail screen does not say any of that.

What it says today, top to bottom:

- Under the score ring, `↳ Related to <one piece title>`. The turn-down arrow is
  a "child of" glyph and the phrasing is singular, so the screen reads as though
  the exercise belongs to that piece.
- With more than one piece, the line becomes `Related to <first> · +2 more` and
  the rest go behind a tap-to-open menu. The most interesting fact about a
  general-purpose drill is the fact hidden behind the disclosure.
- With no piece at all, the line is absent. "Not tied to a piece" and "the app
  has nothing to say here" look identical.
- Lower down, a **By piece** card lists where the exercise has been practised,
  derived from session history (#1087), and only appears once there is history.

So the screen carries two different lists that answer two different questions,
and neither is complete. A piece you linked but have not yet practised alongside
appears only in the breadcrumb; a piece you practised alongside but never linked
appears only in **By piece**. Linking is also one-directional in the UI: you can
create a link from the piece screen, but from the exercise you can only look.

## Approach

Delete the breadcrumb. Replace both lists with one **Used in** card in the body
of the exercise screen: one row per piece, merged from links and practice
history, always present, editable from here.

A row is in one of three states, distinguished by what is already in the kit:

| State | Row shows |
|-------|-----------|
| Linked and practised together | `ScoreRing` with the score in that piece's context, `<composer> · N sessions · last <date>` |
| Linked, never practised together | `ScoreRing` unrated (the rest glyph it already draws for "not yet played"), `<composer> · not practised together yet` |
| Practised together, never linked | `ScoreRing` with the score, sessions meta, plus a **Link** button that turns the fact into an intent |

**On its own** keeps its row at the bottom: practice with no piece in the block
is a real context and its score belongs next to the others. A piece deleted since
it was practised keeps the muted, non-tappable row shipped in #1093.

When there are no pieces at all, the card stays and states it: title
"On its own", body "Not tied to a piece. Link one when it's serving a piece
you're learning.", and the same **Link a piece** footer. Standalone becomes a state the
screen says out loud rather than a gap.

**Row order:** pieces with practice first, most recently practised first; then
linked-but-unpractised pieces alphabetically; then removed pieces; "On its own"
always last.

**Linking from the exercise side** reuses the multi-select picker sheet the piece
screen already opens, with pieces in it instead of exercises.

### What does not change

- The hero `ScoreRing` and its **Overall** eyebrow (#1087). The eyebrow drops
  when the card is empty, since there is nothing to contrast it against.
- **Recent sessions** below.
- The focus player's `↳ Related to <piece>` line during play. There the piece is
  the session block's anchor and genuinely singular, so the breadcrumb is honest.
- The piece screen's **Related exercises** card. It already lists many exercises
  and reads correctly; unlinking from the exercise side updates it for free.

## Key decisions

Agreed with Jon, 2026-08-29:

1. **The breadcrumb goes.** Not "make it plural" and not a chip row under the
   ring. One card in the body carries the whole relationship.
2. **One merged list, not two cards.** Links and practice history are two sources
   for the same question, so they merge and each row says which it came from.
3. **The empty state is a state.** The card always renders; standalone exercises
   get "On its own" plus the link action.
4. **Linking works from both sides**, reusing the existing picker sheet.

## Core changes

The data model is already many-to-many and needs no change: `linked_exercise_ids`
lives on the piece, is persisted in the `item` table, and one exercise id can
appear on any number of pieces. `ItemEvent::LinkExercise { piece_id, exercise_id }`
and `UnlinkExercise` are symmetric, so linking from the exercise screen sends
events that already exist. **No migration.**

What changes is the derivation and the view surface:

- `build_exercise_contexts` currently folds session history only. Widen it to
  seed a row for every piece that links this exercise, then fold history on top,
  so a linked-but-unpractised piece produces a row with no score.
- Add `linked: bool` to `ExerciseContextView`.
- Delete `LibraryItemView::linked_from_pieces` and `PieceRefView`'s use as a
  standalone list. Its only readers are the breadcrumb and preview fixtures.
- Rename the field to `used_in` if we want the view to read the way the screen
  does; keeping `exercise_contexts` is also defensible. Decide in the core PR.

Both edits move the bincode positional wire, so this is a bridge contract change:
domain-sensitivity override applies, `assert_round_trips` covers
`ExerciseContextView` already and must be updated with the new field. The
crash-recovery `ActiveSession` blob does not carry these types, so the
UserDefaults key does not need bumping.

## Tests

Core, written first (`test-driven-development` applies to `intrada-core`):

- A piece that links the exercise with no sessions yields a row with
  `linked: true`, `latest_score: None`, `session_count: 0`.
- A piece practised alongside but never linked yields `linked: false` with its
  score intact.
- A piece both linked and practised yields exactly one row, not two.
- "On its own" sorts last; a removed piece keeps `piece_removed: true`.
- Round-trip `ExerciseContextView` over the real bincode bridge.
- Mutation check: delete the sort and the ordering test must fail.

iOS: snapshots for the three card states (populated, empty, and a row with the
Link button); the picker sheet is an existing component and needs no new
reference.

## Open questions

- **Score attribution** (#1363's second bullet): how a session score flows to
  both the exercise and the piece it was practised under. Out of scope here; the
  rows read the per-context score #1087 already derives. Leave on #1363.
- **The Link button on an unlinked-but-practised row** is a third affordance in
  one card. If it reads as clutter on device, cut it and leave linking to the
  footer sheet.
- **Card title.** "Used in" is short and plain. "Pieces" is shorter and vaguer.
  Settle on device.

## Phasing

Two PRs, core first, per the multi-surface rule:

1. **Core** — widen the derivation, add `linked`, drop `linked_from_pieces`,
   tests. Reviewed before the screens start.
2. **iOS** — the Used in card, the empty state, the picker from the exercise
   side, snapshots, and deleting the breadcrumb.
