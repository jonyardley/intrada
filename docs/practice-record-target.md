# Practice record: target state

What intrada keeps about each practice once sections, variations and the capture rules are in, agreed on 2026-10-03 and recorded on epic #50; #2353 builds on it. The visual version is the [practice record target](https://claude.ai/artifact/D8eT5UVKMnKZ99hxNjkhZ3). Delete when #50 and #2353 close.

**The one rule under all of it:** store events and the musician's own answers, never a verdict the app works out. A tap at 84 with the click on is stored, and so is "partly" when you answer it. "A1 is secure" is worked out each time it is read, so a better reading later can change its mind about old practice.

## 1. The model at the end state

The library holds the music; the record holds practice. A play is the join: it can name one section, one key and any number of variations, and every one is optional. Naming nothing means the whole piece, played plain. The finish answers sit on the entry (the item in the session); the mark stays on each play.

**Library side:**

- **Piece or exercise:** title, written key, tempo, notes. It has sections, a list of chosen keys and variations.
- **Exercise links:** an exercise links to any number of sections, or to a whole piece.

**Record side:**

- **Session:** date, length, notes, and a capture version number. It holds entries.
- **Entry:** one item in the session. Planned variations, planned segments (sections with minutes), intention focus and target. It holds plays, segments and the finish answers.
- **Finish answers, on the entry:** the note as written, points from it, how it felt, what got in the way, whether the intention was met (decision 1).
- **Play:** section (one or none), seconds on the clock, tempo once settled, mark if given, key (a value or none, picked from the item's list), variations (any number).
- **Events on the play:** got it, not quite and undo, each with its tempo and whether the click sounded; tempo and target changes.
- **Away and back:** on the open play, with the offer to leave the gap out.

### Sections

A name, an optional bar range, a kind (part of the form, or a trouble spot) and a target tempo only when it differs from the piece's. Score order is for display only. Spots are added mid-practice, typed or spoken, or with a two-number bar picker, never a form. A chord chart's own sections stay apart from these.

### Keys and variations

Keys follow decisions 2 to 4 below. C sharp major stays C sharp, and both spellings count as one key wherever keys are counted (#2106).

Variations are one list, any number on a play: shared ones (hands separately, dotted rhythms, back to front, three chord tones only) plus an item's own (land on the 3rd). Hands is a variation; there is no Approach kind. Pieces can have variations too. Marks follow decisions 1, 5 and 6.

### Editing history

- Renaming a section or a shared variation keeps its history, because records point at it, not at its name.
- Changing its bars keeps history too.
- Deleting one with history is a soft delete: kept for old records, hidden from pickers.
- Duplicates can be merged later.
- Correcting the section or variations on the finish sheet changes only the open play.

## 2. One practice as the record sees it

The practice in progress copy is what survives the app being killed, so changing its shape costs everyone one lost resume across the upgrade.

| Stage | Kept in |
| --- | --- |
| Build | memory only |
| Play | practice in progress copy (crash recovery) |
| Finish | practice in progress copy |
| Saved | database: one session row |

**Build:**

- Intention typed or spoken; focus and target suggested from it for one tap (#2303).
- Section and variations start empty (whole piece, plain) unless the intention or the segments set them.
- Last time is offered as one chip: "Last time: B · dotted".
- Tempo carries from last time; key starts on the written key.
- Linked exercises are suggested, never added unasked.
- Optional split into section segments with minutes; starts even, total stays fixed.

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

**Saved:**

- The session becomes one row; entries, plays and their events ride inside it, with the capture version.
- Missing stays missing: never zero, never a guess.
- Confirmed points are stored as derived from the note, so a better reader can re-read old notes.

**Away and back:** the core waits for a minimum gap before offering to leave time out, so a quick screen lock does not nag. An away left open across a crash closes at the last saved time.

## 3. Where each thing lives

A finished session is one `session` row with entries, plays and events nested inside. A practice in progress is one versioned blob; anything inside it bumps `ActiveSession::BLOB_VERSION`.

| Thing | Core type | Stored where |
| --- | --- | --- |
| Piece or exercise | `Item` | store `item`; its key column holds the one key value |
| Sections | new type (#2245) | `section` table |
| Keys | fixed key value (#2106) | no table: on the play, the item's list, the written key, a chart's key |
| Variations | Variation replaces `Variant` (#1771) | replaces `variant` |
| Exercise links | link table (#2248) | its own table |
| Session | `PracticeSession` | `session` row, with its capture version |
| Entry | `SetlistEntry` | nested in `session.entries`; live copy in the blob |
| Play, taps | `VariationPlay`, `RepEvent` | nested in the entry |
| Finish sheet while open | `ReflectionDraft` | blob only |
| Practice in progress | `ActiveSession` | versioned key on the device |

New events nest in the play: tempo and target changes (#2107), away and back (#2306). New fields nest in the entry: intention focus and target (#2303), segments (#2315), points from the note (#2307), how it felt and what got in the way (#2308). The shell adds one event (the app left and came back) and one effect beside `RecognitionOperation`, the on-device reader.

**The shell stays a dumb pipe.** The core parses plain patterns itself (tempos, bars, targets). The on-device reader only handles fuzzy ones (what got in the way, how it felt), and the shell runs it when the core asks. The core decides what is a gap, what is a suggestion, what counts as met, and when to offer anything.

**Sensitive surfaces.** Schema, bridge and the practice in progress blob: follow `.claude/rules/sensitive-surfaces.md` (#1374).

**Offline first.** The section, variation and exercise link tables each carry a per-row `updated_at` and `deleted_at`. Keys have no table, so they need neither.

## 4. What reads it later

Four rungs, and only the first is stored. A tap's tempo cannot be recovered later, and a verdict stored today cannot be re-judged tomorrow.

| Rung | Kind | Example |
| --- | --- | --- |
| Events and answers | stored | got it at 84, click on, A1, hands separately |
| Facts | worked out when read | fastest three clean in a row on A1: 84 |
| Patterns | worked out when read | misses on A1 cluster above 88 |
| Help | offered, can be overruled | "Start A1 at 80 today?" Yes, or ignore it |

Rules for every help idea, beside the one rule above:

- Help is offered and can be overruled; ignoring it costs nothing.
- A silent click is never evidence of a tempo.
- Correlations say "alongside", never "because".
- Encourage by showing what an input unlocks, never nag.

## 5. Decided on 2026-10-03

1. **Finish answers live on the entry.** The note, how it felt, what got in the way and whether the intention was met belong to the item in the session; the mark stays per play. Each point from the note carries its own section.
2. **A play with no key** is stored as none and read as the piece's written key. An exercise with none has no key.
3. **Keys are a fixed value** in the core: note, spelling and mode (major and minor now, modes later by app update). Each item stores its chosen keys as a list. No keys table.
4. **One key value everywhere:** the piece's written key and a chart's key use it; the piece keeps its own column holding that value.
5. **Shared variation marks** show per item first, with a pooled view on Progress. Display only.
6. **The piece's own mark** is fed only by plain full run-throughs; sections and variations keep their own. Worked out at read time.
7. **Up next offers last time.** Section and variations start empty unless the intention or the segments set them; last time is a one-tap chip; tempo carries; key defaults to the written key. No carried flag.
8. **Capture version on each session,** set by the core.
9. **Tempo kept once settled:** at the next tap, at close, or after about two seconds still, capped like taps. Tempo and target changes stay their own events and undo its own action.
10. **The core moves old plays** once on upgrade through persistence effects; no shell rewrite.
11. **Chart sections stay apart** from piece sections, with no joining rule now.
12. **Coach-era tables** drop in a small release of their own (#2317).
13. **Segments replace the single planned section** (#2315): one section is a list of one.

### Still to design

- Segment details: how many minutes "Stay on A" adds, and what happens on the last segment.
- The screens for marks per variation (#2250).
- Capturing and showing scores and chord charts, thought about separately and joined later.
- Recording (#69), parked.
- Tap to pick an intention, parked.
