# Intrada

**A practice notebook for musicians**

*Product vision, April 2026; revised July 2026; v3 September 2026, after the
coach reversal (#1344) and the releases v0.10.0 to v0.14.0.*

*Where the work stands and what comes next: [`docs/roadmap.md`](docs/roadmap.md).*

---

## The Problem

Musicians practise but don't always progress. Hours pass at the instrument, yet the same passages stumble, the same keys feel unfamiliar, and the sense of forward motion stalls. This isn't a talent problem: it's a practice design problem with three layers.

**Material gets lost.** A teacher introduces a new voicing pattern, assigns ii-V-I progressions across all keys, suggests a tune to transcribe. The musician scrawls it in a notebook. A week later, the notation is cryptic, the context is gone, and half the assignment is forgotten. Over a year, dozens of rich, multi-key exercises accumulate, and most of them disappear. Self-directed musicians face the same thing: ideas, exercises and repertoire scatter across apps, sheets and memory.

**Deciding what to practise is costly.** The musician sits down with 30 minutes. They could work on voicings, scales over changes, a new tune, that passage from last week, or something from a month ago. Without a plan made in advance, the decision either stalls the session or pushes the musician into playing whatever is on the music stand.

**Progress is invisible for too long.** Music has one of the longest feedback loops of any skill. Improvement is often invisible for weeks, because motor skills consolidate between sessions, not within them. Without evidence that practice is working, anxiety fills the gap: "Am I doing the right thing? Am I wasting my time?"

---

## What Intrada Is

Intrada is a practice notebook: the place a musician's material lives, where today's practice starts, and where the record of what they actually played builds up. **The musician decides what matters; the app decides where today starts, keeps the record honest, and can always be overruled.**

In July 2026 intrada became a practice coach that chose what you practised, gated each block on evidence and left the musician no way round it; it went too far too fast and was removed on 2026-08-14 (#1344). What broke it was the loss of control, not the ready-made plan. What survived, and what every release since has built on, is the notebook.

### Five layers

The layers explain what the notebook does and how the parts depend on each other. They are not a ranking rule; the roadmap ranks work directly.

**1. Capture.** Every piece, exercise, scale and pattern lives in intrada, with its keys, tempo, chord chart and the exercises that build it. Adding one takes one pass, typed or from a photo of the page. Nothing gets lost in a paper notebook.

**2. Plan.** The musician builds a session from the library: pick, group, reorder and set its length, or start from the starred set or the Up next suggestion. The path from "I want to practise" to "I am practising" stays short: practice defaults and the preferred session length are remembered. Routines return as a saved starting point (#1348).

**3. Nudge.** The musician says what matters over the week: the starred items they want to improve and what the teacher set (#1926). The app adds what they would lose track of: what is going cold. From those it offers today's plan, filled to the session length, so starting takes one tap and no decision (#57). The plan is always shown before it starts and can always be changed or set aside for one built by hand. How it weighs the three sources, and whether it shows why each item is there, is still being explored (#2185).

**4. Show.** Progress is real but gradual, so intrada makes it visible: a score and a tempo per item, per key, recorded only when there is evidence behind them (a tempo is kept when the musician set it or the click was sounding). The session record keeps what the musician wrote beside the numbers.

**5. Guide.** Later, and only on the foundation the first four build: patterns the musician can't see for themselves ("your flat keys lag your sharp ones"), then suggested exercises. See Future vision.

---

## Who It's For

Self-directed musicians past the beginner stage who want to practise more effectively, not just more: people working with a teacher who need a structured place for assignments between lessons, self-taught musicians who lack that structure, adult returners rebuilding skills, and anyone preparing for exams, auditions or performance.

Intrada is instrument-agnostic but designed from a keyboard and jazz perspective first, where exercises multiply across twelve keys and the volume of material is especially overwhelming.

**So far the only regular user is the person building it.** The next step is to watch new musicians get from a fresh install to their first marked session (#2121), and to let what they do test everything in this section.

### Musician tracks: an unbuilt hypothesis

Musicians come with different motivations: mastery, one piece for a party, a return after years away. The earlier vision named tracks for these (Entertainer, Jammer, Virtuoso, Soul Player, Late Starter) to shape onboarding and tone. None is built, and none will be until real users show the difference matters. The principle stands: different motivations can want a different experience. See the [Research Foundation](docs/research-foundation.md) for its basis in self-determination theory.

---

## Core Principles

### Progress Is the Product

The primary value intrada delivers is the feeling of making progress. Every feature should either enable progress or make it visible. If it does neither, it doesn't belong.

### The Musician Decides

The app proposes and remembers; the musician chooses. A suggestion can be dismissed, a default can be changed, and nothing blocks the next step on the app's judgement. This is the principle the coach broke.

### Simplicity Over Features

Every interaction should feel lightweight. If logging a session takes more than 30 seconds, it's too slow. The app stays out of the way during practice and does its work before and after.

### Short Path to Start

Every decision between "I want to practise" and "I am practising" is a chance for the session not to happen, particularly for musicians with executive function challenges. Today's plan makes it one tap (#57); remembered defaults and the starred set keep building your own session short, and routines will join them (#1348).

### Reflection Closes the Loop

Practice without reflection is repetition. A short note per item at the end of a session, skippable but always offered, is where the musician says what improved and what still breaks. Those words stay in the session record; showing them on the item is planned with the week's intent (#1926).

### The Record Tells the Truth

A number shown as measurement must be one. Tempo, score and time are recorded when there is evidence for them and left empty otherwise, so a chart never draws a default as if it were played.

### Celebrate Comeback, Not Streak

Intrada never shows a broken streak or a zero. A return is welcomed, and what went cold while you were away is shown as information, never as blame.

### No Journey Is Linear

Musicians plateau, change direction, take breaks and restart. Taking a break is not quitting; coming back after six months is not starting over.

### Designed for Every Mind

Designing for neurodivergent musicians runs through every screen: a short path to start, time made visible, comeback over streak, variable session lengths. Specific commitments: no auto-playing sounds, a calm palette, readable type at every text size, predictable navigation that stays stable across versions, and feedback whose frequency and tone the musician can adjust.

---

## How Practice Material Works

### Keys and variations

An exercise practised across keys tracks each key on its own, so the weak ones are visible rather than averaged away. A variation belongs to the exercise, and a session records the ones you played (#1739, [`specs/exercise-variations.md`](specs/exercise-variations.md)); the remaining wording and tidy-ups are in #1970.

### Item types

Pieces (repertoire, with an optional chord chart) and exercises (scales, arpeggios, patterns, technical studies).

### Piece scaffolding

A piece is built, not only practised. A jazz standard accumulates the exercises that construct it: the melody, shells in each inversion, scales on every change. A piece anchors its related exercises, each tracked separately, practised together and reachable in both directions.

### Routines

Reusable sequences of items (a warm-up, a technical block) that start or join a session, so the musician doesn't rebuild the same plan every day. The first version was removed in #1747; the rebuild is planned under #1974 (#1348).

---

## The Week's Intent

A musician usually knows what this week is for: the tune for Thursday's gig, the voicing the teacher set. Intrada will hold that as **one sentence on an item**, dated and labelled with its source ("From Tuesday's lesson", "Mine"). The current sentence shows while the item plays and pre-fills its aim in the builder; earlier ones stay on the item as history, and the week is read from the dates (#1926). There is no lesson or goal record to manage.

This replaces the separate goals feature ruled in July 2026, which was never built. Goals were built and removed twice before (#213, #769), and each time they became an admin surface of their own. The per-item priority star stays as the zero-ceremony layer beneath.

---

## Future Vision

Real ambitions, each depending on the foundation above and none scheduled:

- **Patterns and gaps.** Weak keys, neglected material and stalls surfaced from the musician's own record.
- **Teacher sharing.** A teacher suggests items, sets target tempos and shares routines, with the student's permission.
- **Audio recording.** Record a run-through and compare it with weeks ago.
- **Curriculum help.** Turning "I want to play Clair de lune" into a sequence of exercises, measured against the musician's record.
- **Correctness help.** Answering "am I doing this right?": whether a voicing or a scale fits the harmony. It needs musical knowledge or a teacher, and is the hardest problem here.
- **Adaptive plans.** Today's plan learning from how the musician changes it, and noticing a plateau.

Machine listening (the app hearing whether a passage was right) is deferred, with the spike's findings in [`docs/segmentation-findings.md`](docs/segmentation-findings.md); MIDI input comes first if listening returns.

---

## Research Foundation

Intrada's design is grounded in learning science, motivational psychology and music education research: spaced repetition, interleaved practice, deliberate practice, self-determination theory, and choice overload. The detail is in the [Research Foundation](docs/research-foundation.md).

---

## Competitive Position

Existing practice apps either track time (measuring attendance, not progress), overwhelm with features that feel like work, or optimise for social engagement over practice quality. Intrada combines fast capture, key-aware tracking, an honest record and neurodiversity-informed design, in a clean, instrument-agnostic app that works entirely on the phone.

A detailed competitive analysis is in the [Research Foundation](docs/research-foundation.md).

---

*The vision is a living document. It changes when the product teaches us something.*
