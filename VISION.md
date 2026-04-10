# Intrada

**Your Intentional Practice Companion**

*Product Vision — April 2026*

---

## The Problem

Musicians practise but don't always progress. Hours pass at the instrument, yet the same passages stumble, the same keys feel unfamiliar, and the sense of forward motion stalls. This isn't a talent problem — it's a practice design problem with three layers.

**Material gets lost.** A teacher introduces a new voicing pattern, assigns ii-V-I progressions across all keys, suggests a tune to transcribe. The musician scrawls it in a notebook. A week later, the notation is cryptic, the context is gone, and half the assignment is forgotten. Over a year, dozens of rich, multi-key exercises accumulate — and most of them disappear. Even self-directed musicians face this: ideas, exercises, and repertoire scatter across apps, sheets, and memory.

**Deciding what to practise is paralysing.** The musician sits down with 30 minutes. They could work on voicings, scales over changes, a new tune, that passage from last week, or revisit something from a month ago. The material is effectively infinite — twelve keys, multiple voicing types, dozens of exercises — and without a plan, the decision cost is high enough to either stall the session entirely or push the musician into grabbing whatever's on the music stand. Research confirms the intuition: choice is only motivating when the learner has the competence and knowledge to choose meaningfully. When they don't, excessive choice becomes a burden rather than a freedom.

**Progress is invisible for too long.** Music has one of the longest feedback loops of any skill. A software developer writes code and sees whether it works in seconds. A musician practises a passage and it sounds… slightly better? Maybe? Improvement is often invisible for weeks — motor skills consolidate between sessions, not within them. Without external evidence that practice is working, anxiety fills the gap: "Am I doing the right thing? Is this the right voicing? Am I wasting my time?" The musician needs to trust the process before they can see the results, but trust without evidence is hard to sustain.

---

## What Intrada Does

Intrada is where a musician's practice material lives and where their daily practice plan comes from. It captures what you need to work on, organises it so you can find it, schedules it so you revisit things at the right time, and tracks your progress so you can see that the work is paying off.

### Five Layers

These build on each other. You can't schedule what you haven't captured, you can't space what you haven't scheduled, and you can't show progress on what you haven't tracked.

**1. Capture and remember.** Every exercise, piece, voicing pattern, scale, lick, and technique lives in Intrada — structured, searchable, and connected to its context. When your teacher introduces ii-V-I progressions in a new voicing, you capture it once and the app knows it's an exercise that spans twelve keys. Your practice notes are searchable across your entire history. Nothing gets lost in a notebook.

**2. Plan what to do today.** Given your available time and your library of material, Intrada generates a practice session. You sit down, tap start, and play. The app absorbs the decision-making burden — you always *can* choose what to work on, but you never *have to*. A 15-minute session gets a focused selection; a 60-minute session gets a broader one. The path from "I want to practise" to "I am practising" requires one decision: start.

**3. Space it for retention.** Intrada schedules returns to material at intervals optimised for long-term retention, not just coverage. Items you're learning surface frequently; items you've consolidated appear less often but before you've forgotten them. The app manages the spacing so you don't have to — and so exercises from six weeks ago don't silently decay while you focus on this week's assignment. This is grounded in decades of research on spaced repetition and interleaved practice.

**4. Show that it's working.** Progress in music is real but gradual. Intrada makes it visible. Mastery ratings per item, per key, and per tempo accumulate over weeks and months into evidence you can see: "Your Db mastery went from 2 to 4 over the last three weeks." This isn't gamification — it's the external confirmation that the process is working, delivered before you can feel it yourself. It turns weeks of invisible improvement into daily visible signals.

**5. Identify gaps and guide what's next.** Over time, Intrada gets smarter. It surfaces patterns you can't see yourself — "You consistently struggle with keys that have four or more flats" or "You spend most of your time on pieces and almost none on the keys you rated lowest." Initially this comes from your practice data alone. Later, it extends to suggesting specific exercises to address weaknesses and generating practice pathways toward goals — the kind of curriculum guidance that a teacher provides instinctively.

---

## Who It's For

Intrada is for self-directed musicians who are past the beginner stage and serious about improvement. The common thread is a desire to practise more effectively, not just more.

This includes musicians working with a teacher who need a structured place to capture assignments and manage the growing library of material between lessons. It includes self-taught musicians who lack the external structure a teacher provides. It includes adult returners who are rebuilding skills after a gap and need evidence that progress is real. It includes anyone preparing for graded exams, auditions, or performance.

Intrada is instrument-agnostic but designed from a keyboard and jazz perspective first — where exercises multiply across twelve keys and the volume of material is especially overwhelming.

### Musician Tracks — Entry Points, Not Boxes

Musicians come to Intrada with different motivations: some want mastery through deliberate practice, some want to learn one piece to play at a party, some are returning to an instrument after years away. Intrada recognises these different starting points through *tracks* — not rigid categories, but lenses that shape onboarding, default suggestions, analytics framing, and encouragement tone.

Tracks are fluid. A Late Starter who gains confidence becomes a Jammer. A Virtuoso who burns out becomes a Soul Player. The app never frames a change as failure — it adapts, and the musician's complete history travels with them.

> **Note:** The specific tracks (Entertainer, Jammer, Virtuoso Classical/Jazz, Soul Player, Late Starter) are a design hypothesis, not validated categories. The underlying principle — that different motivations should shape the experience — is well-supported. The specific categories will be refined against real user data. See the [Research Foundation](docs/research-foundation.md) for the SDT basis.

---

## Core Principles

### Progress Is the Product

The primary value Intrada delivers is the feeling of making progress. Every feature should either enable progress or make it visible. If a feature doesn't serve one of those functions, it doesn't belong.

### Simplicity Over Features

Every interaction should feel lightweight. If logging a practice session takes more than 30 seconds, it's too slow. Musicians will abandon tools that feel like work. The app stays out of the way during practice and does its thinking before and after.

### One Tap to Start

Every decision point between "I want to practise" and "I am practising" is a potential session-ending barrier — particularly for musicians with executive function challenges. The default path requires one decision: tap start. Everything else is optional.

### Celebrate Comeback, Not Streak

Intrada never shows a broken streak counter or a zero. It celebrates returns: "Welcome back — your scheduling has adapted and we've got a session ready." Consistency matters, but the framing determines whether it motivates or shames.

### No Journey Is Linear

Musicians plateau, change direction, take breaks, restart, and redefine what they want from their instrument. Intrada normalises all of this. Taking a break is not quitting. Choosing enjoyment over mastery is not giving up. Coming back after six months is not starting over.

### Designed for Every Mind

Designing for neurodivergent musicians isn't a separate workstream — it's embedded in Intrada's core design. Reducing decisions to start, externalising time, celebrating comeback over streak, and supporting variable session lengths all stem from this principle. These features benefit every musician, but they're essential for musicians with ADHD, who are likely a meaningful portion of the audience.

Specific commitments: sensory-considerate defaults (no auto-playing sounds, calm palette, configurable contrast), accessible typography (clean fonts, adequate spacing, dark grey on off-white), predictable navigation (same layout, same flows, stable across versions), and configurable feedback (frequency, tone, and delivery all adjustable).

---

## How Practice Material Works

### Key-Aware Grouping

When a musician adds an exercise to be practised across keys, Intrada generates sub-items for each key, tracking mastery independently. The smart scheduler treats each key as a separate unit — weaker keys surface more frequently. This is essential for jazz musicians working through voicings, progressions, and patterns, and for classical musicians working through technical exercises.

The app also recognises when a musician always practises keys as a sequential loop and can prompt independent key practice to ensure genuine fluency rather than motor pattern reinforcement.

### Item Types

Pieces (repertoire with sections), exercises (scales, arpeggios, technical studies), licks and vocabulary (jazz patterns, typically across keys), and techniques (specific technical goals like left-hand independence or pedalling).

### Routines

Reusable sequences of items — a warm-up routine, a technical block, a cool-down set — that can be inserted into any session. This reduces daily decision-making, supports consistency, and is particularly valuable for musicians who struggle with session initiation.

---

## The Scheduling Intelligence

The scheduler combines spaced repetition urgency (what's overdue for review), interleaved ordering (mixing material types for better retention), goal alignment (what serves the musician's current objectives), and time fitting (what fits in the available session length).

This isn't AI — it's an algorithm grounded in well-established learning science. The research basis is documented in the [Research Foundation](docs/research-foundation.md).

---

## Future Vision

The five layers describe what Intrada is building toward now. Beyond them lie capabilities that extend the product significantly:

**AI-assisted curriculum design.** When a musician says "I want to play Claire de Lune" or "I want to improvise like Bill Evans," the app decomposes the goal into prerequisite skills, maps them against the musician's current mastery data, and generates a sequenced pathway of exercises and studies. This is the "illuminate the critical path" promise at its most ambitious — and it depends on the data foundation that layers 1–4 build.

**Correctness validation.** Helping musicians answer "am I doing this right?" — whether a voicing choice is appropriate, whether a scale fits the harmony. This requires either AI musical knowledge or teacher integration, and is the hardest problem in the stack.

**Teacher integration.** Teachers can suggest items, set target tempos, view progress (with permission), and share routines. This bridges the gap between lesson and practice room — the teacher's assignments flow directly into the student's practice plan.

**Audio recording and playback.** Record a run-through, play it back, compare to weeks ago. Objective evidence of improvement that goes beyond numerical ratings.

**Adaptive intelligence.** The scheduling algorithm learns from the musician's response patterns — adjusting interleaving intensity, detecting plateaus, and suggesting strategy changes when progress stalls.

These are real ambitions, not aspirational padding. But they depend on the foundation being solid first. See the [roadmap](docs/roadmap.md) for current priorities and timeline.

---

## Research Foundation

Intrada's design decisions are grounded in evidence from learning science, motivational psychology, and music education research. The detailed research basis — including spaced repetition, interleaved practice, deliberate practice, self-determination theory, growth mindset, choice overload, and goal-specific pathways — is documented separately in the [Research Foundation](docs/research-foundation.md).

---

## Competitive Position

Existing practice apps either track time (measuring attendance, not progress), overwhelm with features that feel like work, or optimise for social engagement over practice quality. Intrada's unique position is the combination of structured capture, evidence-based scheduling, key-aware tracking, fine-grained progress metrics, and neurodiversity-informed design — in a clean, instrument-agnostic package.

A detailed competitive analysis is maintained in the [Research Foundation](docs/research-foundation.md).

---

*The vision is a living document. It evolves as the product matures and as real user data validates or challenges its assumptions.*
