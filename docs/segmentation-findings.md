# Attempt segmentation: what timing can and cannot decide

> Findings from the PR 3 spike (`docs/rebuild-review.md` §6). Code:
> `crates/intrada-core/src/engine/`. Evidence: the five real MIDI takes in
> `crates/intrada-core/tests/fixtures/midi_takes/`, recorded on a Roland LX-706
> over Bluetooth MIDI at 92 bpm. Tests: `tests/engine_segmentation.rs`.
> Written 4 Aug 2026. This note is the spec input for the scoring gate.

## Verdict

Segmentation works, and it works on **content plus timing, never timing alone**.
Every one of the five takes segments correctly. Two of the five cannot be
segmented at all without the target phrase, and one of those two —
resume-versus-restart — is not merely hard from timing but *provably
undecidable*, because the two cases are timing-identical by construction.

So the spike passes, with a condition attached: **an attempt is defined by what
it was an attempt at.** Segmentation is not a preprocessing step that hands
tidy attempts to a scorer; it is already scoring, and it must be given the
prescribed phrase as an input. Design decision 2 (click-always) is necessary
but not sufficient — a shared clock gives bar/beat alignment, not attempt
boundaries.

## What timing alone decides

| Decision | Signal | Evidence |
|---|---|---|
| Bar/beat of a note | grid arithmetic from the click anchor | all takes |
| Chord vs melody | onsets within a 50ms window are one event | take 01: 5-note voicings spread ≤34ms |
| Silence long enough to matter | gap > 1.75× the phrase's step spacing | takes 02, 04: ~1.93s gaps against a 652ms step |
| Consistent feel vs erratic time | mean offset vs spread over an attempt | take 04's clean rep: mean −24ms (ahead of the click), spread 17ms |
| Phrase-ish grouping with no target | rest-separated spans | take 01 splits into 4 spans |

The feel-vs-error split is the one that matters most for scoring, and it holds
on real Bluetooth data: take 04's clean repetition sits at a mean −24ms with a
17ms spread. Offsets are signed `onset − expected`, so that rep is consistently
**ahead** of the click, not laying back. Design decision 6 is measurable either
way — a stable displacement reads as stable, not as eight badly-timed notes —
but the sign has to reach the user's screen the right way round: reporting a
push as a lay-back is precisely the never-bluff failure.

## What timing alone cannot decide

**1. Resume versus restart. Undecidable.** Takes 02 and 04 open with the same
six correct notes and pause for the same length (1.930s and 1.980s — 50ms
apart, under a tenth of the 652ms crotchet the silence interrupts). Take 02
then plays notes 7–8; take 04 goes back to note 1. No gap threshold, at any
tolerance, separates them. Only the pitch content after the silence does.

Worse, content sometimes does not either. The gate phrase is F C F C **B F** B
F: pause after note 5 (B) and the next expected note is F — which is also note
1. Resuming and restarting produce an identical note at an identical time. The
engine resolves this by looking ahead and taking whichever reading matches more
of the phrase (`prefers_restart`), which is a heuristic on *later* content, not
a decision at the boundary. **Consequence for the gate:** a restart must not be
inferred in real time. Either the attempt window is closed by the click (fixed
rep length, the Swift gate drill's approach) or the verdict lags until enough
following content arrives.

**2. Where an attempt starts.** One matching note is not an attempt. Take 03's
scale run crosses the phrase's first note five times, and each crossing opened
a spurious one-step attempt until starts required a confirming run of 2 matched
steps. **Consequence:** an attempt that fails on its second note is
indistinguishable from noodling and will not be recorded as an attempt at all.
For a mastery model fed by pass/fail evidence this is a *bias*, not just a gap:
the worst attempts are silently dropped, so the failure rate is understated.
Either the gate opens attempts on the click rather than on content, or
attempts-to-pass instrumentation (design challenge 4) is measuring a filtered
population.

**3. Collapse versus walking away.** Take 05 plays five notes, hits E instead
of F, and stops. `Collapsed` versus `Diverged` is decided by whether playing
*continued* after the wrong note, and `Abandoned` covers stopping while still
on track — so the outcomes describe what followed, which is the only thing
observable. None of them distinguishes "gave up" from "the doorbell rang".
Timing cannot supply intent, and the app should not guess: an abandoned attempt
is not evidence of failure and must not feed the mastery update.

**4. Whether noodling was an attempt.** Take 01 is 101 notes of unstructured
playing. Against no target it yields no attempts, correctly. Against the gate
phrase it also yields none — but only after two false-positive sources were
closed (see below). The general problem is unsolved: free playing that happens
to contain the phrase's notes is not an attempt at the phrase, and only
intention distinguishes them. This is design decision 3 (deploy-gates are
self-confirmed) arriving early, at the segmentation layer.

**5. Timing of a paused attempt.** Take 02 completes all eight notes, so it is
`Completed` — and its phrase timing is arithmetic about a phrase that was never
played continuously (mean +324ms, spread 557ms). The two post-pause notes are
in fact dead on the click, two beats late relative to the phrase (the silence
itself runs to 2.96 beats). Both facts are true; neither is a score. `Attempt::timing_is_scorable()` gates on
`Completed && pauses.is_empty()` for exactly this reason. **A pause is a
fluency failure, and it must be detected before any timing verdict is
computed, not after.**

## Two false positives worth remembering

Both were found by running the freeplay take against a phrase it never
attempted — the case with no expected answer, which is why it is the most
useful fixture in the set.

- **Pitch-class matching is too permissive with chords.** Octave tolerance was
  implemented as "the onset contains this pitch class", which a wide voicing
  satisfies almost by accident: the gate phrase asks only for F, C or B, and
  each of take 01's five-note voicings contains enough of them to satisfy
  between 4 and all 8 of its steps. A run of **seven consecutive** unrelated
  voicings therefore carried an attempt to seven matched steps before it
  abandoned. Fixed by matching pitch-class *width* exactly: a one-note step
  needs a one-class onset, so octave doublings still pass and voicings do
  not.
- **Single-note attempt starts.** As above: five spurious attempts inside one
  scale run.

Both would have shipped as "gate passes on noodling" bugs in a UI, and neither
is visible in a clean take. Fixture value is inversely proportional to
tidiness.

## Thresholds, and why none of them are milliseconds

Every time-shaped threshold is a ratio of **the target phrase's own step
spacing**, not of a grid beat and not an absolute duration
(`SegmentConfig`). The gate phrase is crotchets, so its step spacing is one
beat; take 03's noodle runs at half that. A pause threshold in beats would
misread a quaver phrase, and one in milliseconds would misread every tempo but
92.

| Threshold | Default | Basis |
|---|---|---|
| `chord_window_us` | 50ms | measured 34ms max spread on real rolled voicings |
| `pause_ratio_milli` | 1.75× step spacing | separates 1.93s pauses from 652ms crotchets |
| `abandon_ratio_milli` | 4× step spacing | no fixture reaches it — synthetic test only, provisional |
| `max_consecutive_deviations` | 2 | take 03 diverges for good after 2 |
| `min_start_run_steps` | 2 | minimum that kills take 03's false starts |

The 50ms chord window is the one that will break first. At 200 bpm a
semiquaver is 75ms, so a 50ms window is a third of the subdivision it must
resolve — chord clustering and fast subdivisions collide, and the window will
have to scale with the phrase's step spacing like everything else. It does not
yet.

## Corrections to the engine spec's FFI contract

`specs/intrada-coach-engine.md` §6 needs two amendments before it is built:

1. **`t_us` must be signed (`i64`, not `u64`).** Two of the five real takes
   open with a note *before* the click anchor (−22ms and −47ms), anticipating
   beat 1. A `u64` cannot represent them, and clamping to zero silently
   destroys the early-note case — the exact thing timing feedback exists to
   measure.
2. **Count-in is beats, not bars.** The capture harness records
   `countInBeats`; a 2-beat count-in is ordinary. `ClickGrid.count_in_bars: u8`
   should be `count_in_beats: u8`.

A third point is forward-looking rather than a correction: `Attempt.span` and
`Segmentation.count_in` are `Range<usize>`, which will not survive facet
typegen. Whatever crosses the bridge carries plain start/len integers, and the
`Range` stays core-side.

Also worth stating in the spec: **count-in playing is untested against real
data.** All five takes have Jon waiting through the count-in, so the count-in
exclusion path is covered by a synthetic stream only. It needs one real take of
someone playing over their own count-in.

## What the scoring gate should therefore assume

1. Segmentation takes the prescribed phrase as an input. There is no
   target-free attempt detection.
2. Attempt boundaries come from the click, not from content, wherever the drill
   allows it. Content-derived boundaries are a fallback with known bias.
3. Only `Completed && pauses.is_empty()` attempts produce a timing verdict.
   Pauses, restarts, collapses and abandons are *outcomes*, reported as
   outcomes.
4. Abandoned attempts are excluded from the mastery update, not counted as
   failures.
5. Attempts that die before `min_start_run_steps` are invisible. Until
   boundaries come from the click, the observed failure rate is understated and
   the calibration instrument in design challenge 4 knows it.
6. Timing consistency (mean and spread) is reportable on Bluetooth; per-note
   verdicts are not, which is design decision 7 already established and now
   confirmed against a 17ms spread on the one clean repetition in the set (23ms
   and 27ms on the diverged and collapsed attempts) — the same order as BLE's
   own ±10–20ms jitter, which is exactly why the spread is reportable and a
   single note's offset is not.
