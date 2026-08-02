# Phrase p001 — the ♭9 turnaround

**Status:** in flight (the one allowed in-flight phrase) ·
**Pipeline stage:** 2 of 7 (nearby keys) ·
**Method pack:** [`phrase-transposition`](../method-packs/phrase-transposition.md)

**Source.** Generic bebop language, authored as the template instance. If
the phrase currently under your fingers from the record is a different one,
replace the notation and devices below and keep the structure — the
structure is the schema (README, day-one checklist).

## The phrase

Two bars over a ii–V–I (two beats each on the ii and V), shown in the home
key of C. Eight consecutive swing eighths from beat 1; the A♭ falls on the
and-of-4, and the G lands on beat 1 of bar 2:

```text
   Dm7        G7           Cmaj7
| F E D C    B D F A♭    | G — — — |
```

Descending scalar line from the 3rd of Dm7; up the G7♭9 arpeggio from its
3rd; the A♭ (♭9 of G7) resolves down a semitone to G, the 5th of Cmaj7,
landing on beat 1. Target tempo: 120 bpm, swing eighths, click L2.

**Generate-ladder rung 1 (preview, not yet in flight):** same line, enclosed
ending — replace the landing with F–D♯–E so it resolves to the 3rd instead
of the 5th. Stays parked until extract (stage 5) is done and generate
(stage 6) opens.

## Analysis (stage 4 preview — the annotation the app will one day write)

The crunch-then-release you like is the ♭9: maximally tense against the V,
resolving by the smallest possible move onto the most stable colour tone of
the I. The first bar is plain vocabulary; the second bar is the device.

## Devices carried (extract stage, 5, will formalise these)

| Device | What it is |
|---|---|
| `d-flat9-to-5` | ♭9 over the V resolving down a semitone to the 5th of the I |
| `d-v7b9-arpeggio` | Ascending V7♭9 arpeggio from the 3rd |
| `d-scalar-from-3rd` | Scalar descent starting on the 3rd of the ii |

## Pipeline position

| Stage | Name | State |
|---|---|---|
| 1 | Learn (home key) | done |
| 2 | Nearby keys | **in progress** |
| 3 | Full cycle | — |
| 4 | Analyse | — |
| 5 | Extract | — |
| 6 | Generate | — |
| 7 | Integrate (over Strasbourg) | — |

## Per-key state

Key = key of the resolution chord. Status ladder: `untouched` → `learning`
(found it, not clean) → `solid` (passed `phrase-nearby-keys`/`phrase-cycle`
criterion in that key) → `proven` (passed cold in a `phrase-random-key`
deal). Solid keys get a maintenance pass each session; a failed maintenance
pass demotes to `learning`. Seeds below are day-one guesses — correct them.

| Key | Status | Top clean tempo | Last practised | Notes |
|---|---|---|---|---|
| C | solid | 120 | seed | home key |
| D | learning | 90 | seed | whole step up |
| B♭ | learning | 90 | seed | whole step down |
| E♭ | untouched | — | — | |
| E | untouched | — | — | |
| F | untouched | — | — | |
| F♯ | untouched | — | — | |
| G | untouched | — | — | |
| A♭ | untouched | — | — | |
| A | untouched | — | — | |
| B | untouched | — | — | |
| D♭ | untouched | — | — | |
