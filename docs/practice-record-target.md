# Practice record: target state

The visual version of this page is the [practice record target](https://claude.ai/artifact/D8eT5UVKMnKZ99hxNjkhZ3); the work sits under epic #50.

This is what intrada keeps about each practice once sections, variations and the capture rules are in. v0.16 builds the shape, v0.17 builds everything that changes the record, and v0.18 adds the on-device reader and the section mark screens. Read from main at v0.15.0 (commit bba9363e) on 2026-10-03, and revised the same day after review; the decisions are recorded on #50.

**The one rule under all of it:** store events and the musician's own answers, never a verdict the app works out. A tap at 84 with the click on is stored, and so is "partly" when you answer it. "A1 is secure" is worked out each time it is read, so a better reading later can change its mind about old practice.

## 1. The model at the end state

The library holds the music; the record holds practice. A play is the join: it can name one section, one key and any number of variations, and every one is optional. Naming nothing means the whole piece, played plain. The finish answers sit on the entry (the item in the session); the mark stays on each play.

**Library side:**

- **Piece or exercise:** title, written key, tempo, notes. It has sections, a list of chosen keys and variations.
- **Sections (v0.16):** name, optional bars, a kind (part of the form, or a trouble spot), and a tempo only when it differs from the piece's.
- **Keys on the item (v0.16):** a list of fixed key values. There is no keys table.
- **Variations (v0.16):** shared ones plus the item's own.
- **Exercise links (v0.16):** an exercise links to any number of sections, or to a whole piece.

**Record side:**

- **Session:** date, length, notes, and a capture version number (v0.16). It holds entries.
- **Entry:** one item in the session. Planned variations (v0.16); planned segments, intention focus and target (v0.17). It holds plays, segments and the finish answers.
- **Segments (v0.17):** planned sections with minutes, on the entry.
- **Finish answers, on the entry:** the note as written (today); points from the note, each with its section; how it felt; what got in the way; whether the intention was met (v0.17).
- **Play:** section (one or none), seconds on the clock, tempo once settled, mark if given, key (a value or none, picked from the item's list), variations (any number).
- **Events on the play (v0.16):** got it, not quite and undo, each with its tempo and whether the click sounded; tempo and target changes.
- **Away and back (v0.17):** on the open play, with the offer to leave the gap out.

### Sections

A name, an optional bar range, a kind and a target tempo only when it differs from the piece's. Score order is for display only. Spots are added mid-practice, typed or spoken, or with a two-number bar picker, never a form. A chord chart's own sections stay apart from these.

### Keys

A key is a fixed value in the core: a note, its spelling, and major or minor. Modes come later, added by an app update. Each item keeps its own list of chosen keys. The piece's written key and a chord chart's key use the same value. C sharp major stays C sharp, and both spellings count as one key wherever keys are counted (#2106). A play with no key reads as the piece's written key; an exercise with none has no key.

### Variations

One list, any number on a play. Shared ones (hands separately, dotted rhythms, back to front, three chord tones only) plus an item's own (land on the 3rd). Hands is a variation. There is no Approach kind. Pieces can have variations too. A shared variation's marks show per item first; a pooled view sits on Progress, for display only.

### Marks and answers

The mark stays on each play. The finish answers live on the entry, and each point pulled from the note carries its own section. Only plain full run-throughs, with no section and no variations, feed the piece's own mark; sections and variations keep theirs. All of it is worked out when read.

### Editing history

- Renaming a section or a shared variation keeps its history, because records point at it, not at its name.
- Changing its bars keeps history too.
- Deleting one with history is a soft delete: kept for old records, hidden from pickers.
- Duplicates can be merged later.
- Correcting the section or variations on the finish sheet changes only the open play.

## 2. One practice as the record sees it

Each stage lives somewhere different. The practice in progress copy is what survives the app being killed, so changing its shape costs everyone one lost resume across the upgrade.

| Stage | Kept in |
| --- | --- |
| Build | memory only |
| Play | practice in progress copy (crash recovery, UserDefaults) |
| Finish | practice in progress copy |
| Saved | database: one session row |

**Build:**

- Intention typed or spoken; focus and target suggested from it for one tap (#2303).
- Section and variations start empty (whole piece, plain) unless the intention or the segments set them.
- Last time is offered as one chip: "Last time: B · dotted".
- Tempo carries from last time; key starts on the written key.
- Linked exercises are suggested, never added unasked.
- Optional split into section segments with minutes; starts even, total stays fixed.
- Skipping the build is fine.

**Play:**

- Switching section, key or variations closes the open play and opens a new one.
- Got it, not quite and undo each keep the tempo and whether the click sounded.
- A tempo is kept once settled: at the next tap, at close, or after about two seconds still.
- Taps past the target are kept; the count on screen stops at the target.
- Mark a spot, typed or spoken, or with the bar picker.
- On return after a real gap: "Away 6 minutes. Leave it out?"
- Segment time up: "On to B" or "Stay on A, 2 more minutes taken from B". Never a hard stop.

**Finish:**

- Done is the one required tap.
- Intention met is worked out from the record, else asked once: yes, partly, not yet. Skippable; skipped stores nothing.
- Mark per play: one tap, skippable; skipped stores nothing.
- Note typed or spoken, kept as written; points pulled from it are suggestions you confirm.
- How it felt and what got in the way: suggested, or from a folded Add detail row.
- Correcting the section or variations here changes only the open play.

**Saved:**

- The session becomes one row; entries, plays and their events ride inside it.
- Each session carries a capture version number, set by the core.
- Missing stays missing: never zero, never a guess.
- Confirmed points are stored as derived from the note, so a better reader can re-read old notes.
- Your answers are stored; verdicts the app works out are not.

**Away and back:** the core waits for a minimum gap before offering to leave time out, so a quick screen lock does not nag. iOS cannot always tell a screen lock from leaving the app, so the threshold carries it. An away left open across a crash closes at the last saved time.

## 3. Where each thing lives

The library lives in its own database tables. A finished session is one `session` row with entries, plays and events nested inside as JSON. A practice in progress lives in one versioned blob in UserDefaults, keyed `intrada.session-in-progress.v5` today. Anything inside that blob bumps `ActiveSession::BLOB_VERSION`, which is 5 on main: v0.16 takes it to 6 and v0.17 to 7, once each.

| Thing | Core type | Stored where | What changes it |
| --- | --- | --- | --- |
| Piece or exercise | `Item` | GRDB `item`; key column holds the one key value | v0.16: `linked_exercise_ids` and `variants` move out; gains its list of chosen keys |
| Sections | new type (#2245) | new `section` table, per-row `updated_at` and `deleted_at` | Migration, v0.16 |
| Keys | fixed key value (#2106) | no table: on the play, the item's list, the written key, a chart's key | v0.16, inside #2246 |
| Variations | Variation replaces `Variant` (#1771) | replaces `variant`; per-row `updated_at` and `deleted_at` | Migration, v0.16; steps move across |
| Exercise links | today `Item.linked_exercise_ids` | link table, per-row `updated_at` and `deleted_at` (#2248) | Migration, v0.16 |
| Session | `PracticeSession` | `session` row | v0.16: capture version, set by the core |
| Old session plays | plays naming a step | past `session` rows | v0.16: the core moves steps to keys and variations once, through persistence effects; no Swift rewrite |
| Entry | `SetlistEntry` | nested in `session.entries`; live copy in the blob | Blob 5 to 6: planned variations replace `planned_variation_id` |
| Play | `VariationPlay` | nested in the entry | Blob 5 to 6: section, key, variations replace `variation_id`; mark stays per play |
| Taps | `RepEvent` | nested in the play | Blob 5 to 6: tempo and click per tap; undo; `rep_target_reached` dropped |
| Tempo and target changes | new events (#2107), kept once settled, capped like taps | nested in the play | Blob 5 to 6 |
| Away and back | new events (#2306); core sets the minimum gap | nested in the play | Blob 6 to 7 |
| Intention and met | `SetlistEntry.intention` plus focus and target (#2303) | nested in the entry | Blob 6 to 7 |
| Segments | sections with minutes; replace the single planned section | nested in the entry | Blob 6 to 7 (#2315) |
| Note and points | entry `notes`; points each with a section (#2307) | nested in the entry | Blob 6 to 7 |
| How it felt, what got in the way | optional fields on the entry (#2308, #2307) | nested in the entry | Blob 6 to 7 |
| Finish sheet while open | `ReflectionDraft` | blob only | Rides each bump |
| Practice in progress | `ActiveSession` | UserDefaults, versioned key | 6 in v0.16, 7 in v0.17 |
| App left and returned to | new event from the shell | shell only reports it | Bridge shape, v0.17 |
| On-device reader | new effect beside `RecognitionOperation` | shell runs it where on-device AI exists | Bridge shape, v0.18 |
| Library sort | sort type (#2089) | UserDefaults `intrada.library-sort` | Core only, v0.16: versioned key and wire pin |
| Coach-era records | none; nothing reads them | nine GRDB tables | Dropped in a small release of their own, not v0.16 |

The nine coach-era tables are `block_record`, `wander_record`, `user_drill`, `journal_item`, `built_session`, `play_through`, `reflection`, `feel_entry` and `unmonitored_play`.

**The shell stays a dumb pipe.** Swift reports two facts the core cannot see: the app went to the background, and it came back. The core parses plain patterns itself (tempos, bars, targets). The on-device reader only handles fuzzy ones (what got in the way, how it felt), and Swift runs it when the core asks. Without on-device AI (older iPhones, much of Android) the full experience is the core's plain patterns plus the folded Add detail row. The core decides what is a gap, what is a suggestion, what counts as met, and when to offer anything.

**Sensitive surfaces.** Schema, bridge and the practice in progress blob. Each phase that touches them ships the core PR first and the screens in the same working session (#1374). If the upgrade fails part way, the app keeps the old data untouched and says so. Android runs on the same core, so each core phase regenerates both sets of bindings, and each migration needs its androidx.sqlite counterpart once that shell holds data.

**Offline first.** The section, variation and exercise link tables each carry a per-row `updated_at` and `deleted_at`. Keys have no table, so they need neither.

## 4. From today to there

| Today, v0.15 | After |
| --- | --- |
| An exercise has Steps: one ladder, in a set order. A key is a typed label the app guesses at (`is_key_label`). | Steps become variations. Key steps become fixed key values on the item's list of keys; the rest become the item's own variations. The core moves every step's history once on upgrade. |
| A piece's key is a text column plus a mode; a chart has its own key text. | One key value everywhere. |
| A practice names one step at most (`variation_id`), and only exercises have them. | A play names one section, one key and any number of variations. Pieces can have variations. |
| #2246 and #50 say one variation of each kind, with a possible Approach kind. | One list, no cap, no kinds. Analysis reads each variation on its own. |
| Up next suggests the first step in ladder order not yet solid. | No step order. Up next offers last time as a one-tap chip. |
| A piece holds a list of linked exercise ids. | A link table: an exercise links to sections or a whole piece. |
| Taps keep got it or not quite and the time; not quite also undoes; taps past the target dropped; tempo read once at close. | Each tap keeps its tempo and click; undo is its own action; taps past the target kept; tempo and target changes kept once settled. |
| `rep_target_reached` is stored. | Worked out from the count and the target each time. |
| Intention is one free text "Aim". | Free text kept, with an optional focus and target (v0.17). |
| A saved session does not say which capture rules wrote it. | Each session carries a capture version number. |

## 5. What reads it later

Four rungs, and only the first is stored. A tap's tempo cannot be recovered later, and a verdict stored today cannot be re-judged tomorrow.

| Rung | Kind | Example |
| --- | --- | --- |
| Events and answers | stored | got it at 84, click on, A1, hands separately |
| Facts | worked out when read | fastest three clean in a row on A1: 84 |
| Patterns | worked out when read | misses on A1 cluster above 88 |
| Help | offered, can be overruled | "Start A1 at 80 today?" Yes, or ignore it |

| Help idea | Needs | Records from |
| --- | --- | --- |
| Safe tempo per section | section on the play, tempo and click per tap, undo apart from not quite | v0.16 |
| First attempt of the day | play start times, section, taps with tempo | v0.16 |
| The section you go round | planned sections (segments) beside played sections, play minutes | played v0.16, planned v0.17 |
| Slow down to speed up (#1793) | tempo per tap, settled tempo changes, target changes | v0.16 |
| A variation for the problem (#1794) | what got in the way on the entry, variations, marks and taps | v0.17; from notes v0.18 |
| Intentions you keep missing | intention focus and target, met worked out or answered (#2303) | v0.17 |
| Exercises alongside a section | exercise links (v0.16), section marks shown back (#2250) | usable from v0.18 |
| Hardest first while fresh | play order and time, section marks, how it felt (#2308) | v0.17 |

Rules for every help idea:

- Store events and the musician's own answers, never a verdict the app works out.
- Help is offered and can be overruled; ignoring it costs nothing.
- A silent click is never evidence of a tempo.
- Correlations say "alongside", never "because".
- Encourage by showing what an input unlocks, never nag.

## 6. Release by release

**v0.16, the shape:**

- #2245: sections on a piece or exercise, with the spec. Core and storage.
- #2246: steps become shared variations; a play names a section, a key and its variations. Carries #2106 and #2107; the core moves old plays once; one blob bump.
- #2106: keys are a fixed value in the core, in either spelling; each item keeps its list.
- #2107: every tap keeps its tempo; taps past the target kept; tempo and target changes kept once settled; undo.
- #2226: the key picker reads its circle of fifths from the core.
- #2247: sections and variations on the item screen.
- #2248: an exercise links to any number of sections.
- #2190: delete the streak no screen shows.
- #2170: delete a past session's start time nothing reads.
- #2089: version the saved library sort and pin its shape. Core only.
- No issue yet: a capture version number on each saved session.
- Also closes #1083, #1771, #1769, #1925, #2232 and #1363.

**v0.17, everything that changes the record** (one blob bump; none carries the v0.17.0 milestone yet):

- #2249: build, play and mark a session by section and variation.
- #2303: an intention the record can check: a focus and a target.
- New issue: segments, an item split into sections with minutes, replacing the single planned section.
- #2306: leave time away out with one tap.
- #2307: plain patterns in the note (tempos, bars, targets) parsed by the core.
- #2308: how it felt, apart from how it went.

**v0.18, the reader and section marks:**

- #2316: the on-device reader for fuzzy points, split out of #2307.
- #2250: each section's mark on the piece; marks per variation.

**A small release of its own:** #2317 drops the nine coach-era tables.

**Later:** #1793, #1794, #1978 (Up next epic), #1501 (which key to practise), #2251, #2252, #69 (recording, parked), and tap to pick an intention (parked, no issue).

**Order inside v0.16:**

1. #2245 core with `specs/sections-and-variations.md` as its first commit; migration only, no blob change. The spec covers editing history and keeps chart sections apart.
2. #2246 core, carrying #2106 and #2107: the big migration, the core moving old plays, the bridge, blob 5 to 6.
3. #2226, then #2247 screens in the same working session as step 2; #2246's screens half adds the undo control on the player.
4. #2248 core, then its screen.
5. #2190, #2170 and #2089 fit around these. #2170 is a bridge shape and goes core first; #2089 is core only.

## 7. The release is right if

Where nothing on screen shows the result, the check is a core test on the saved record.

**v0.16:**

- Upgrading keeps each key step's history under its key, every other step's under the item's own variation; nothing lost.
- A key step such as E flat major lands on the item's list of keys, and C sharp major stays spelt C sharp.
- If the upgrade fails, the app keeps the old data untouched and says so.
- A practice in progress from before the upgrade is discarded with a one-line notice, never half read.
- A piece can hold A1 (bars 1 to 16), B and A2, still there after relaunch; reordering changes the display only.
- Renaming A1 keeps its history; deleting a section with history hides it from pickers and keeps old practice intact.
- A play left untouched is recorded as the whole piece, played plain.
- Every got it and not quite keeps its tempo and click; got it past the target is kept while the count holds (core test).
- Climbing from 60 to 84 with no taps leaves the settled tempo changes in the record, not every step on the way (core test).
- Undoing an accidental got it is stored as an undo, not a not quite; #2246's screens half adds the undo control.
- Each saved session carries the capture version number (core test).
- An exercise linked to A2 of one piece and the coda of another shows both; removing A2 removes only that link.

**v0.17:**

- A play can name A1, in D, dotted rhythms and hands separately (needs #2249's screens).
- Up next shows "Last time: B · dotted" as one chip; ignoring it starts on the whole piece, plain, at last time's tempo, in the written key.
- Typing or saying "A1 at 84" offers section A1 and tempo 84 to confirm with one tap.
- An intention of 84 on A1 reads as met when a play of A1 reached 84 with the click sounding; otherwise the finish sheet asks once, and skipping stores nothing.
- Splitting 20 minutes into A and B starts at 10 and 10, and the stepper keeps the total at 20.
- When A's time is up, the sheet offers On to B or Stay on A; ignoring it never stops the clock.
- Finishing an item with nothing touched takes one tap and stores no mark.
- Locking the screen for a few seconds offers nothing; leaving for six minutes and tapping Leave it out saves the play six minutes shorter.
- A note "left hand rushed in bar 12" is kept word for word and offers bar 12; a confirmed point is stored as from the note, with its section.
- Choosing Tense from Add detail stores it on the entry; finishing without it stores nothing.

**v0.18:**

- On a phone with on-device AI, "left hand rushed" offers rhythm as what got in the way, and a note saying tense offers Tense.
- On a phone without it, the same choices sit in the folded Add detail row.
- Re-reading old notes with a better reader changes derived points, never the note or anything confirmed.
- The piece screen shows its own mark from plain full run-throughs only, with each section's mark beside it.

**Later:**

- Safe tempo per section never counts a tap with the click silent.
- Rates per variation read each variation on its own, never a combination.
- Every help line can be dismissed, and dismissing it changes nothing stored.
- A correlation is worded "alongside", never "because".
- Nothing asks twice for the same missing input.

## 8. Decided on 2026-10-03

1. **Finish answers live on the entry.** The note, how it felt, what got in the way and whether the intention was met belong to the item in the session; the mark stays per play. Each point from the note carries its own section.
2. **A play with no key** is stored as none and read as the piece's written key. An exercise with none has no key.
3. **Keys are a fixed value** in the core: note, spelling and mode (major and minor now, modes later by app update). Each item stores its chosen keys as a list. No keys table.
4. **One key value everywhere:** the piece's written key and a chart's key use it; the piece keeps its own column holding that value.
5. **Shared variation marks** show per item first, with a pooled view on Progress. Display only.
6. **The piece's own mark** is fed only by plain full run-throughs; sections and variations keep their own. Worked out at read time.
7. **Up next offers last time.** Section and variations start empty unless the intention or the segments set them; last time is a one-tap chip; tempo carries; key defaults to the written key. No carried flag. #50's capture rule 3 becomes "carry forward as an offer".
8. **Capture version on each session,** set by the core. Needs an issue or a home in #2246.
9. **Tempo kept once settled:** at the next tap, at close, or after about two seconds still, capped like taps. Tempo and target changes stay their own events and undo its own action, reversing the 2026-09-24 "not now" on #2107.
10. **The core moves old plays** once on upgrade through persistence effects; no Swift rewrite.
11. **Chart sections stay apart** from piece sections, with no joining rule now. Capturing and showing scores and charts is thought about separately.
12. **Coach-era tables** stay through v0.16 and drop in a small release of their own (#2317).
13. **Segments get their own v0.17 issue** under #50 and replace the single planned section: one section is a list of one.
14. **Three releases:** v0.16 the shape; v0.17 #2249, #2303, #2306, #2308, segments (#2315) and #2307's plain patterns; v0.18 the on-device reader (#2316) and #2250's section mark screens.
15. **Issues rewritten to match** before #2246 starts: #50, #2245 (missing the section kind, target tempo, and adding a spot mid-practice), #2246, #2250 and #2303 ("not yet", not "no"). #2249 needs no rewrite.

### Still to design

- Segment details: how many minutes "Stay on A" adds, and what happens on the last segment.
- The screens for marks per variation (#2250).
- Capturing and showing scores and chord charts, thought about separately and joined later.
- Recording (#69), parked.
- Tap to pick an intention, parked.
