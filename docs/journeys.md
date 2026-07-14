# The Ideal Journey

> Canonical walkthrough of what Intrada is for, written from the musician's
> side of the screen. Every feature is judged against this document: if a
> change doesn't move one of these steps forward, ask why it's being built.
> Status column last audited 2026-07-14 (vision/journey audit). Use this as
> the walking-skeleton checklist: before starting new feature work, walk the
> ten steps on a device; the first place the journey breaks is the next
> priority.

## The journey, told as one story

I'm learning a jazz standard. The tune lives in my **library** alongside
everything else I'm working on. To build it up (and build the ability to
improvise over it) I create the **exercises and activities that scaffold it**:
learn the melody, shells in each inversion, scales down to each chord tone on
every change, improvising on limited chord tones, improvising in rhythm only.
Each of those is **tracked separately**: its own score, its own tempo, its own
keys, its own reps.

Intrada knows my **goals**, so when I sit down to practise it **recommends a
session**: drawn from what I'm working towards, what's going stale, and where
I'm weak. **Focus mode guides me through it** one exercise at a time and stays
out of the way. When something clicks (or breaks) mid-exercise, I can **jot it
in the moment** without stopping. At the end I **reflect properly**: what
improved, what's still broken, what to target next time.

Later, my **progress report speaks my language**: not just charts, but what I
actually said, played back against the numbers. And when an exercise stops
serving me, Intrada **suggests alternatives** that work the same skill.

## The ten steps

| # | Step | What "done" means | Status (2026-07) |
|---|------|-------------------|------------------|
| 1 | Library of pieces | Add a piece in seconds, enrich progressively: composer, key(s), tempo, difficulty, named sections. Find it by any facet. | Partial – multi-key (#46), difficulty (#372), sections (#50) missing |
| 2 | Exercises scaffold a piece | Build and maintain an ordered set of related exercises per piece; create inline; navigate both directions; practise as a block. | Built |
| 3 | Track each exercise separately | An exercise's detail answers "how is this going?" across score, tempo, reps, and keys. | Partial – score only; tempo has no iOS input, reps not aggregated, keys absent |
| 4 | Goals drive planning | Capture an outcome ("learn Body and Soul") linked to items; planning consumes it. | Missing – rebuilt small per the 2026-07 ruling (roadmap Q5) |
| 5 | Recommended sessions | The Practice tab proposes one concrete session with per-item reasons; one tap starts it. | Missing – signals exist in core, nothing composes them |
| 6 | Guided focus mode | One item at a time, chrome-free, survives interruption and relaunch. | Built – crash recovery (#962) and pause (#1000) outstanding |
| 7 | Reflection in the moment | One-tap capture mid-item that saves and returns to the timer; never forces advance. | Partial – capture only at item hand-off, and it advances |
| 8 | End-of-session reflection | Structured prompts (improved / still broken / next target) against the session's intention; skippable, never a gate. | Partial – single unlabelled note box |
| 9 | Narrative progress | Past sessions re-readable; item note history visible; Progress quotes your words next to the deltas. | Missing on iOS – notes persist but are write-only |
| 10 | Suggest alternative exercises | 2–3 quiet, dismissible suggestions that work the same skill; one tap to adopt. | Missing |

## Priority within the journey

The front half (1–3, 6) is the logbook; the back half (4, 5, 7–10) is the
companion. The companion loop is what differentiates Intrada: reflection feeds
narrative progress, narrative progress and goals feed recommendation,
recommendation feeds the next session. Sequence work to close that loop:
reflection first (cheapest, highest leverage), then narrative surfaces, then
tracking completeness, then goals, then recommendation, then suggestions.
