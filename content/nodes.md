# Skill-graph nodes — the first five

Schema per node: what it is, why it matters, prerequisites, 2–4 drills,
machine-checkable done-criteria (referenced from [`gates.toml`](gates.toml)),
and mastery as **(estimate, confidence)** — never a boolean. Circle and mode
tags per the fluency frame (see [`README.md`](README.md)).

In Phase 0 "machine-checkable" means *a machine could check it*: the
criterion is stated in notes, beats, tempos, and counts, with no judgement
words. Until the listening gate exists, you check it yourself — honestly.

Mastery seeds are day-one guesses. Correct them at the piano (README
checklist), then let the gate-attempt log move them.

---

## 1. Shell voicings through ii–V–I — `shells-ii-v-i`

**Circle:** hands · **Mode:** keys

**What.** Two-note left-hand shells (root+3rd or root+7th), chosen per chord
so the 3rds and 7ths resolve by step through a ii–V–I. The minimum viable
statement of the harmony.

**Why.** Shells are the floor under everything: enough harmony to support
any melody or solo line, the hand shapes rootless voicings are built from,
and a physical map of where the 3rds and 7ths live — which is exactly the
raw material `chord-tone-targeting` needs. Mastered shells are also the
designated warm-up material: comfortable, scoreable, low-stakes.

**Prerequisites:** `diatonic-7ths` (stub — name the 7th chords in any major key).

**Drills.**

| Id | Drill | Gate |
|---|---|---|
| S1 | ii–V–I shells around the cycle of fourths, LH alone, sustained whole-bar hits | `shells-cycle` |
| S2 | Shells under the melody, Strasbourg A section, hands together | `shells-under-melody` |
| S3 | Random-key retrieval: shuffle 12 key cards, land the ii–V–I shells cold | `shells-random-key` |

**Done-criteria.** All three gates in `gates.toml` passed within the same
week: full cycle clean at 100 bpm on click level L2, tune application clean,
and 12/12 random-key retrieval within one bar of thinking time.

**Mastery seed:** estimate 0.7, confidence medium.

---

## 2. Rootless voicings, A and B positions — `rootless-a-b`

**Circle:** hands · **Mode:** keys

**What.** Four-note rootless left-hand voicings for the ii–V–I: A-form built
up from the 3rd (for Dm7: 3-5-7-9 → F–A–C–E), B-form built up from the 7th
(for G7: 7-9-3-13 → F–A–B–E), alternating forms so the voice leading moves
by step or not at all. The bass player owns the root; you own the colour.

**Why.** The modern LH comping sound, and the current frontier. Minimal-
motion voice leading between these forms is the engine of comping; every
hour here pays out in every tune. Honest label: the middle keys are grind.

**Prerequisites:** `shells-ii-v-i`.

**Drills.**

| Id | Drill | Gate |
|---|---|---|
| R1 | One key, LH alone, slow loop: ii–V–I with a full bar on each chord | `rootless-one-key` |
| R2 | The change, not the chords: loop just ii→V, then just V→I, minimal motion | `rootless-transition` |
| R3 | Key traversal per the method pack: 2–3 new keys per session, whole-step order | `rootless-traversal` |
| R4 | Application: rootless LH under the melody, Strasbourg A section | `rootless-under-melody` |

**Done-criteria.** `rootless-cycle` gate: full cycle of fourths, both forms
alternating, clean at 80 bpm, click L1 — reached over weeks via the
traversal schedule, never in one session.

**Mastery seed:** estimate 0.3, confidence low.

---

## 3. Phrase transposition — `phrase-transposition`

**Circle:** bridge (head → hands) · **Mode:** keys; advanced rung away

**What.** The front half of the lick pipeline: take the current phrase
([`p001`](phrases/p001-flat9-turnaround.md)) from its home key through all
twelve, per the
[phrase-transposition method pack](method-packs/phrase-transposition.md).
One phrase in flight at a time.

**Why.** Transposition is micro-transcription digested: it converts a lick
you can quote into language you own in any key, and proves you learned the
*device* (intervals against the chord), not a fingering. This is the bridge
between the circles, built out — and Phase 1's first scored drill type.

**Prerequisites:** `micro-transcription` (entry rung), `swing-click-time` (stub).

**Drills.**

| Id | Drill | Gate |
|---|---|---|
| P1 | Home-key consolidation: phrase over its ii–V–I loop with click | `phrase-home-key` |
| P2 | Nearby keys: whole step up and down from home | `phrase-nearby-keys` |
| P3 | Cycle chain: phrase through consecutive cycle-of-fourths keys, ii–V motion linking them | `phrase-cycle` |
| P4 | Random-key retrieval: deal a key card, one count-in, play it | `phrase-random-key` |

**Done-criteria.** Per-key state in the phrase file reaches `solid` in all
12 keys and `phrase-random-key` passes — then the phrase advances to the
analyse/extract stages of its pipeline.

**Mastery seed** (node-level — running the pipeline as a skill; per-key
state lives in the phrase file): estimate 0.4, confidence low.

---

## 4. Chord-tone targeting — `chord-tone-targeting`

**Circle:** bridge (head + hands intersection) · **Mode:** keys

**What.** Restricted improvisation over the tune form with one rule: land
the 3rd or 7th of the new chord on beat 1 of every change. Everything else
is free.

**Why.** The smallest unit of "making the changes". It forces the ear to
hear the next chord before the hands leave this one, and it's where scale
knowledge stops being weather and starts being intent. The worked-example
session's integration block is this node parameterised by the tune.

**Prerequisites:** `shells-ii-v-i` (you must know where the 3rds and 7ths
live before you can aim at them), `swing-click-time` (stub).

**Drills.**

| Id | Drill | Gate |
|---|---|---|
| C1 | Guide-tone skeleton: whole notes only, 3rds and 7ths connected by step through the full form | `targeting-skeleton` |
| C2 | Approach and land: one free bar, then land the target on beat 1 of the next change; loop two-bar cells | `targeting-approach` |
| C3 | Full form, restricted improv, click L2; self-rate the feel 1–5 after each pass | `targeting-full-form` |

**Done-criteria.** `targeting-full-form` is deliberately the softer gate the
spec prescribes for integration work: the landing rule is machine-checkable
(target chord tone on beat 1, every change, 2 consecutive passes), the feel
self-rating is logged alongside, not gated on.

**Mastery seed:** estimate 0.35, confidence low.

---

## 5. Micro-transcription — `micro-transcription`

**Circle:** head · **Mode:** ladder from keys to away

**What.** Lifting 1–2 bar phrases from records by ear — never whole solos.
The modes are a difficulty ladder (per the fluency frame): listen-and-repeat
at the keys is the entry rung; the advanced rung is working the phrase out
*away* from the instrument, hearing it fully before playing a note — pure
audiation, no trial-and-error from the hands.

**Why.** This node grows the head circle, and it is the intake valve for the
whole vocabulary pipeline: every `p00N` phrase enters through here. Skipping
it is how the content set collapses into hands-only exercises.

**Prerequisites:** none. (`interval-recognition` stub accelerates it but
doesn't gate it.)

**Drills.**

| Id | Drill | Gate |
|---|---|---|
| M1 | Entry rung: pick a 1–2 bar phrase from the current listening record; ≤5 listens, match it at the keys | `transcription-entry` |
| M2 | Sing first: sing the phrase accurately (pitch contour + rhythm) before touching the keys | `transcription-sing` |
| M3 | Audiation rung: work the phrase out entirely away from the piano (commute slot); first attempt at the keys is the test | `transcription-audiation` |

**Done-criteria.** Paper-tier honesty: these gates are self-confirmed
against the record in Phase 0 (`transport = paper` in `gates.toml`), the
same stance the spec takes on deploy-gates. They become machine-suggestable
only once capture exists.

**Mastery seed:** estimate 0.4, confidence low.

---

## Stubs — the rest of the improvisation branch

Named so gates and whys can reference them; deliberately unspecified until
the loop is proven. Circle tag on each so the branch-level balance stays
visible.

| Stub | Circle | One line |
|---|---|---|
| `diatonic-7ths` | hands | Name and play the seven 7th chords of any major key |
| `swing-click-time` | hands | Swing eighths held against the sparse-click ladder |
| `interval-recognition` | head | Hear and name intervals, then 3rds-and-7ths inside voicings |
| `guide-tone-lines` | bridge | Compose/play continuous guide-tone lines through full forms |
| `device-generate-ladder` | bridge | The back half of the lick pipeline: vary, transplant, compose, deploy |
| `comping-rhythms` | hands | Charleston and its displacements under a recorded soloist |
| `upper-structures` | hands | Triads over rootless voicings; the pipeline after A/B forms |
| `listening-assignments` | head | Album-per-week structured listening; the no-instrument block |
| `tune-form-memory` | head | Form and changes from memory, away from the keys |
