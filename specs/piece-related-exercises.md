# Related exercises on a piece: yours first, the chart's second

> Tier 3 by domain sensitivity (one new `Event` crosses the FFI bridge); the
> surface is otherwise a Tier 2 card redesign. Issue [#1431]. Two PRs: core
> first (the event and its tests), screens second (the card, the copy and the
> snapshots). No migration, no schema change, no API change, no new token and
> no new primitive.
>
> Mockups: [`design/`](piece-related-exercises/design/) and the
> [canvas](https://claude.ai/code/artifact/c646b020-6a05-4fe9-9479-73bdbd42528e).

[#1431]: https://github.com/jonyardley/intrada/issues/1431

## Problem

On a piece with a chord chart, the loudest thing on the screen is the app
writing five exercises for you. **See the curriculum** is a full-width brand
bar on the chart card; it opens a sheet of five derived exercises, all
pre-ticked, and **Add 5** creates them as real linked exercises.

Writing your own is four taps away and hidden inside the wrong flow: the
**Related exercises** card offers **Add a related exercise**, which opens a
picker of exercises you already have, and **Create new exercise** is a row
inside that picker. Worse, the exercise you create there is **not linked** to
the piece: `LibraryAddScreen` sends `ItemEvent::Add` and returns you to the
picker, where you must then find and tick the thing you just made.

So the app's own suggestion is one tap and lands linked; yours is four taps and
does not. Jon's call (2026-08-28): a person creates their own related
exercises, and the app may pre-populate some examples.

## Approach

Reverse the two, and add the missing link.

1. **The primary action on a piece is creating your own exercise**, as the
   card's only brand bar, and it saves already related to the piece.
2. **Choosing an existing exercise stays**, demoted to plain text beneath it.
3. **The derived five become suggestions**: a quiet row below a hairline inside
   the same card, not a brand bar on a different one, and the word
   *curriculum* goes.

The derivation itself is untouched. `derive_scaffold`, `ScaffoldPreviewView`
and `CommitScaffold` all keep working exactly as they do; only their framing
and their place on the screen change.

## The contract

One new event, modelled directly on `CommitScaffold`, which already creates
exercises and links them to a piece atomically in the core:

```rust
/// Create one hand-written exercise already linked to `piece_id`, in a single
/// event so the shell never has to learn the new item's id. Local-first only
/// (invariant 6); mirrors `CommitScaffold`'s create-and-link, for one item the
/// user typed rather than a batch the core derived.
AddLinkedExercise {
    piece_id: String,
    input: CreateItem,
},
```

Handler, in order:

1. `validate_create_item(&input)` after `normalize_create_item`, exactly as
   `ItemEvent::Add` does. A bad title fails the same way it does today.
2. The piece must exist and be `ItemKind::Piece`. `validate_link_exercise`
   already carries that predicate for an existing pair; the new event needs the
   host half of it before the exercise exists.
3. `input.kind` is forced to `ItemKind::Exercise`. The form cannot offer a
   choice here and the core should not trust one.
4. Mint the ulid, push the item, extend `piece.linked_exercise_ids`, stamp both
   `updated_at`, then `save_items(vec![exercise, piece])` and `record_success()`.

**Local-first only.** Online returns the same "not available online yet" shape
`AddVariant` uses. The online create path reassigns ids server-side, so a link
minted against the client ulid would dangle; that is the same trap already
marked `FIXME(#1108)` inside `CommitScaffold`, and this event must not walk
into it a second time.

**No ViewModel change.** The new exercise reaches the screen through
`LibraryItemView.linked_exercises`, which already projects the piece's links.
Nothing new needs to cross the wire in that direction.

## Key decisions

**D1 — One event, not a created-id channel.** The alternative was to surface
the last created item's id on the ViewModel and let the shell fire
`LinkExercise` after `Add`. Rejected: it puts a two-step transaction in the
shell, which is the dumb-pipe rule inverted, and it leaves a window where the
exercise exists unlinked. `CommitScaffold` settled this shape already.

**D2 — The suggestions live in the Related exercises card**, not on the chart
card. Everything about exercises for this piece then sits in one place, and the
idea does not vanish on the (much more common) piece with no chart. The
alternative is on the canvas as `AltOnChartCard` if this is revisited.

**D3 — The suggestions row is a control, not a section.** It sits on
`surfaceSunken` with the gold exercise tint on its icon and chevron. Drawn as a
plain row with only a chevron, Jon read the eyebrow above it as the button, and
it creates five library items, which is too much to hang on a chevron. The gold
also says what tapping it will produce before it is tapped. See
[T18](../docs/design-principles.md#t18).

**D4 — The brand bar belongs to the empty state.** Once the card has exercises
in it, the primary action recedes to a plain footer action alongside "choose
one". One-primary-action-per-screen is about the screen having one obvious next
step, and once you have related exercises, adding another is not it.

## Copy

Against [`docs/tone-of-voice.md`](../docs/tone-of-voice.md); no possessives on
chrome (rule 3), sentence case, no full stops on buttons.

| Where | Today | Becomes |
|-------|-------|---------|
| Chart card action | `See the curriculum` | *(removed)* |
| Sheet title | `Curriculum` | `From the chord chart` |
| Sheet subhead | `Derived in G minor · 5 exercises` | `Worked out from these changes, in G minor` |
| Spec flag | `Already linked` | `Already added` |
| Card primary | *(none)* | `Create an exercise` |
| Card secondary | `Add a related exercise` | `Choose one from the library` |
| Suggestions eyebrow | *(none)* | `From the chord chart` |
| Empty-state help | `Add scales, arpeggios, or any exercise you practise alongside this piece.` | `Scales, arpeggios, and anything else you practise alongside this piece.` |
| Chart empty help | `Paste the changes to see the exercises they imply.` | `Paste the changes to keep them with the piece.` |

Two `ScaffoldSpec` rationales in `domain/chart.rs` carry an em dash, which the
tone doc bans outright. They take the house middle dot:

- Shells: `3rd + 7th of every chord · the voice-leading skeleton`
- Constrained improv: `Chord tones only, then rhythm · one ladder`

## Out of scope

- [#1389] — whether `Learn the melody` and `Constrained improv` should be
  `ItemKind::Exercise` at all. Renaming the surface does not settle it.
- [#1390] — the same capture problem on the **create** form rather than the
  piece page.
- [#1432] — drawing the chord chart properly, and showing it while practising.
- [#1363] — many-to-many links and score attribution.
- The online path for the new event, per invariant 6 and [#1108].

[#1389]: https://github.com/jonyardley/intrada/issues/1389
[#1390]: https://github.com/jonyardley/intrada/issues/1390
[#1432]: https://github.com/jonyardley/intrada/issues/1432
[#1363]: https://github.com/jonyardley/intrada/issues/1363
[#1108]: https://github.com/jonyardley/intrada/issues/1108

## Open questions

1. **Does the create form return to the piece, or to the new exercise?**
   Returning to the piece shows the link landing, which is the point of the
   change. Leaning that way; it is a one-line difference in the screens PR.
2. **Should `Choose one from the library` keep the full picker** (star, sort,
   tag, search) now that it is the secondary path, or drop to a plain list?
   Keeping it; the picker is already built and its filters earn their place on
   a long library.

## Phasing

**PR 1 (core).** `AddLinkedExercise`, its validation, its handler, and its
tests: happy path, non-piece host, missing piece, invalid title, kind coerced
to exercise, online rejection. A real-bridge round-trip for the new event in
`assert_round_trips` **before** either screen touches it, per the #846 rule.
Plus the two `chart.rs` rationale strings, which are pure copy.

**PR 2 (screens).** The card rebuilt from the mockup, the sheet's copy, the
snapshots re-recorded, and the accessibility labels for the new controls.
