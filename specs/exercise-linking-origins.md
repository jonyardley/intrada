# How exercises came to be linked to pieces, and to have variations

> **Origin note, not a contract.** This folds four shipped-record specs into the
> decisions that still hold and the reasons behind them. For how the app works
> now, read the live specs: [`exercise-variations.md`](exercise-variations.md)
> (variations and the record of what was played, #1739),
> [`exercise-relations.md`](exercise-relations.md) (the "Used in" list) and
> [`piece-related-exercises.md`](piece-related-exercises.md) (writing your own
> exercise first). Verify anything here against the code before building on it.

**Vocabulary gap.** The code says Variant (`Variant`, `SetVariants`,
`UpdateVariants`, `variant_id`), while the screens, the live spec and the
session record (`VariationPlayView`) say "variation"; the rename is #1771.

## Where this came from

The folded documents are recoverable from git at the commit before this note
landed (`git log --diff-filter=D -- specs/<name>.md`):

| Folded file | What it specified | Where it shipped |
|---|---|---|
| `piece-linked-exercises.md` | A piece carries an ordered list of exercises, each showing its latest score | #1015, committed as a spec in #1090 |
| `piece-linked-exercises-design-brief.md` | The design brief for the above, and the first score ring (#1009) | #1015 |
| `related-exercises-redesign.md` | Matching the iOS screens to the finished design, in six phases | #1026, #1030, #1036, #1038, #1041, #1042, #1043 |
| `exercise-variants.md` | An exercise owning an ordered list of variations, each scored on its own (#1083) | #1112 (schema), #1118 (mechanism) |

The mocks the first three drew from stay in [`../design/`](../design/)
(`linked-exercises-mock.dc.html`, `Linked Exercises.dc.html`); the
`exercise-variants/design/` folder went with its parent and is in git too.

## Linking exercises to a piece (#1015)

The problem: a piece has its scales, shells and runs, and nothing tied them to
it, so there was no way to see how each drill for a piece was going.

1. **Shared, not owned.** An exercise is a first-class library item that many
   pieces can link; its edits and score are live everywhere it appears. No
   gated stages and no separate status: tracking is the exercise's own score.
2. **An ordered id list on the piece** (`Item.linked_exercise_ids`), resolved
   to live exercises in the core view, rather than a link entity or a reuse of
   routines. The trade-off accepted: the whole list is last-writer-wins on the
   piece, fine for one musician on one device.
3. **Three events** (`LinkExercise`, `UnlinkExercise`,
   `ReorderLinkedExercises`), each bumping the piece's `updated_at`.
   Validation: the host is a piece, the target an existing exercise, no
   duplicates, no self-link.
4. **A tombstoned or missing exercise drops out of the resolved list**; the
   view filters it, the stored ids are not rewritten.
5. **The reverse view is computed, never stored**: which pieces link this
   exercise is a scan over pieces. It started as "Linked from", became a
   "Related to" breadcrumb, and is now the "Used in" list
   ([`exercise-relations.md`](exercise-relations.md)).

## The screens catching up with the design (#1026 to #1043)

Every decision here took the path with no migration, because on the device the
store is the only copy of the data: a genre is a tag, not a field; the recent
sessions list shows score, date and trend with no note line; the reflection
sheet is a score selector and a note, with no live ring. The one bridge write
the work added from Swift, `updateEntryNotes`, got a `LiveBridge` round trip
(#846). The work also dropped the per-exercise "include today" toggle; that
was later overturned (`docs/design-principles.md`, #1101). On iPad the library
is a hand-built split rather than `NavigationSplitView`, because the custom
scrolling library does not fit its `List(selection:)` model, and building it
by hand left the iPhone layout and its snapshots unchanged.

## Variations on an exercise (#1083)

The problem: "all twelve keys" or "add a note each round" is a ladder, and one
flat score on the exercise turns progress along it into noise. One generic
mechanism, no code per preset.

1. **On the wire the list rides inside `Item`; on the device it lives in its
   own `variant` table** (GRDB migration `v9_variant`), the store's first child
   table, so each variation carries its own `updated_at` and `deleted_at` and
   two edits to different variations never conflict as a whole item.
2. **No hard deletes, and tombstones stay in the model.** The core owns
   reconciliation and the shell never diffs child rows; a session entry that
   names a removed variation still finds its label. So the store loads
   tombstoned rows too: filtering them out on load (as #1112 first did) would
   make a re-added variation lose its history. Positions are unique only
   among live variations, never across tombstones.
3. **The whole list is written at once and reconciled by label**,
   case-insensitively: a match keeps its id and its history, a removed label
   is tombstoned, a re-added one comes back with its history, and a call that
   changes nothing writes nothing. Renaming cannot be told apart from remove
   and add by label alone, so it needs the id (`UpdateVariants`, #1783).
4. **"Solid" is a latest score of 8 or more out of 10** (`SOLID_SCORE_MIN`),
   a named constant rather than a setting until real use argues otherwise.
   The "current step" built on it was retired by #1739.
5. **Scope capped at label, position and score history**: every extra field
   per variation is a migration on the only copy of the data.
6. **Limits**: exercises only, at most `MAX_VARIANTS` (24), each label 1 to
   `MAX_VARIANT_LABEL` (100) characters, no case-insensitive duplicates.

Superseded by [`exercise-variations.md`](exercise-variations.md): the current
rung, and a session entry holding one score, one tempo and one variation; an
entry now holds a list of plays.
