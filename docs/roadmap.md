# intrada — Revised Product Roadmap

*Draft for review — 2026-02-21*

This roadmap replaces the four-phase structure in VISION.md with six smaller, more
realistic phases. Each phase delivers a complete, valuable product — not a stepping
stone. The changes are driven by a critical review of the user journey from product,
teacher, and student perspectives.

**Key changes from the original roadmap:**

1. The original Phase 2 ("Intelligence") was overloaded — it contained the SRS engine,
   interleaved scheduling, one-tap start, mastery decay, difficulty balancing, goals,
   encouragement, and rest awareness. That's been split across Phases 3 and 4.
2. Practice quality improvements (focus mode, rep counter, visual timer) move earlier —
   they enhance the *existing* session system without needing the scheduling engine.
3. Tempo-building is promoted to a first-class practice flow, not just a data field.
4. A first-time user onboarding flow is added (it was completely absent).
5. "Free practice" and "deep dive" modes are added to prevent the app feeling like it
   *only* supports structured interleaved practice.
6. The streak counter is reworked to comeback framing immediately, not deferred to Phase 3.
7. Scoring captures tempo alongside mastery rating.

---

## What's Built Today

| Area | Status |
|------|--------|
| Auth (Clerk/Google, JWT, JWKS refresh, 401 retry) | ✅ Complete |
| Library CRUD (piece + exercise, tags, key, tempo, notes) | ✅ Complete |
| Sessions (build → active → summary, timer, crash recovery) | ✅ Complete |
| Routines (full CRUD, load into builder, save from summary) | ✅ Complete |
| Analytics (weekly stats, streak, 28-day chart, top items, score trends) | ✅ Complete |
| Scoring (1-5 per item) and per-item/session notes | ✅ Complete |
| Search/filter infrastructure in core (ListQuery) | ✅ Backend only (no UI) |

---

## Phase 1: Practice Quality (4 weeks)

**Theme: "Make every session count"**

Enhance the existing session experience. No new scheduling intelligence — just make
the act of practising meaningfully better with what we already have.

### Features

- **Focus mode** — strip the UI to essentials during active practice. Current item +
  timer only. Navigation and library hidden but accessible via swipe/tap.
- **Visual progress timer** — replace the numeric countdown with a progress ring/bar.
  Show both per-item and total session elapsed. Gentle chime at suggested transition.
- **Repetition counter** — optional per-item tool. Tap ✓ to increment, ✗ to decrement
  (never below 0). Configurable target (3–10, default 5). When reached, auto-prompt
  transition to next item.
- **Lightweight micro-cycle** — before each item, a quick-tap focus selector (accuracy /
  evenness / dynamics / tempo / custom). After each item, a 1–5 rating tap + optional
  short note. **Not** a mandatory reflection essay — one tap is the minimum interaction.
- **Light path for maintenance items** — items at mastery ≥ 4 get a streamlined flow:
  focus mode → timer → quick rating → auto-transition. No goal prompt or extended
  reflection. Configurable threshold.
- **Search/filter UI** — expose the existing ListQuery infrastructure as a search bar +
  filter chips on the library list. The backend is already built.
- **Rework streak → comeback framing** — replace the current streak counter in analytics
  with "X of the last 7 days" language. Celebrate returns, never shame gaps. This
  aligns the existing analytics with the vision's inclusive design principles.
- **Scoring captures tempo** — when rating an item, optionally record the tempo achieved
  alongside the 1–5 mastery score. This is essential for tracking progress on items
  where mastery is tempo-dependent.

### Delivers

A meaningfully better practice experience using the existing library and session system.
Musicians feel the difference immediately — the app respects their time and attention
during practice, not just before and after.

### Dependencies

None — all features enhance the existing system.

---

## Phase 2: Library Depth (4 weeks)

**Theme: "A library that understands music"**

Make the library genuinely useful for structured music study. This phase is prerequisite
for the scheduling engine — the scheduler needs richer data to make intelligent decisions.

### Features

- **Key-aware exercise generation** — mark an exercise as "all 12 keys" and auto-generate
  12 sub-items. Each tracks mastery independently. Parent exercise shows aggregate view.
  Musician sees one library item but can drill into per-key progress.
- **Section/passage management** — break pieces into practisable sections. Each section
  has its own mastery score and tempo target. The scheduler (Phase 3) can surface
  specific sections rather than whole pieces.
- **Expanded item types** — add Licks/Vocabulary and Technique as first-class types
  alongside Piece and Exercise. Each type can have type-specific metadata.
- **Effort/difficulty tags** — per-item difficulty level and effort rating. These feed
  the session difficulty balancer in Phase 3.
- **Lesson capture flow** — "What did your teacher assign this week?" A quick-add
  interface optimised for translating lesson notes into library items. Available from
  onboarding and as a recurring entry point.
- **First-time user onboarding** — guided flow for empty libraries: "How do you
  practise?" → lesson capture or quick-add wizard → first session prompt. Addresses
  the Day 1 cold-start problem.
- **Free practice mode** — a timer-only mode that logs time without requiring items,
  scores, or structure. Legitimises unstructured practice time. Logged in analytics
  as "free practice" so it counts toward consistency.

### Delivers

The library becomes genuinely useful for structured music study. Jazz students can track
all 12 keys. Classical students can break sonatas into sections. The cold-start problem
is solved.

### Dependencies

Phase 1 (scoring captures tempo — needed for section-level tempo targets).

---

## Phase 3: Scheduling Intelligence (6 weeks)

**Theme: "Practise what matters most"**

The signature feature. This is the largest phase because the SRS engine, interleaved
generator, and session planning all need to ship together to deliver the one-tap start
experience.

### Features

- **Mastery decay model** — scores decrease over time if items aren't reviewed. Rate of
  decay is configurable. Creates natural urgency in the scheduling algorithm.
- **Spaced repetition engine** — modified SM-2 algorithm. Intervals based on mastery
  scores + time since last practice. High-effort items get shorter intervals (informed
  by Donovan & Radosevich's finding that task complexity moderates the spacing effect).
  Parameters should be tuneable and validated against real user data over time.
- **Interleaved setlist generator** — create mixed-type practice sessions. Session begins
  with more similar items and increases variety as the session progresses (Mathias &
  Goldman, 2025). Configurable interleaving intensity: gentle mixing → full interleave.
- **Tempo-building practice flow** — a first-class sub-loop within any practice item.
  Set current tempo + target tempo. Play → judge (solid?) → bump or hold → repeat.
  Iterative tempo-building is how musicians actually work — it's a session within a
  session. Metronome integration (built-in or external — **open question**).
- **One-tap session start** — the landing screen shows today's algorithmically generated
  session with a prominent Start button. Musician picks a duration, taps Start, and plays.
  Zero other decisions.
- **Short session options** — 10, 15, 20, 30, 45, 60 minutes prominently offered.
  Framing: short sessions are legitimate and effective.
- **Difficulty balancing** — front-load demanding work when focus is fresh. No back-to-back
  high-effort items. Taper toward lighter review material.
- **Deep dive mode** — pick ONE item, open-ended time, intensive work. The interleaved
  approach is the default, but sometimes a musician needs to shed. This respects that
  interleaving isn't always the right answer.
- **Transition prompts** — gentle cues between items. Preview of what's coming next.
  Optional 30-second micro-break. Reduces "what do I do now?" cognitive load.

### Delivers

Open app → tap Start → get a personalised, science-backed practice session. The tempo
loop makes practice feel like real musicianship, not admin. Deep dive mode prevents the
app from feeling prescriptive.

### Dependencies

Phase 2 (key-aware exercises, sections, effort tags — the scheduler needs this richer
data to make good decisions).

---

## Phase 4: Goals & Encouragement (4 weeks)

**Theme: "See the process working"**

Add the motivational layer. Goals give direction; encouragement provides evidence that
the direction is working.

### Features

- **Goal framework** — three tiers:
  - Session goals: "Focus on left hand today" (set at session start)
  - Weekly goals: "Practice 5 days this week" (set from goals screen)
  - Milestone goals: "All keys to mastery 4 by March" (linked to specific items)
- **Goal-aligned scheduling** — items tagged for active goals get a priority boost in
  the scheduling algorithm. Active goals influence session composition.
- **Encouragement messages** — data-tied, process-focused. "Your Db mastery went from
  2 to 4 over three weeks — that's the spacing effect at work." Not empty praise.
  Tied to observable improvements in the data.
- **Rest & recovery awareness** — flag when practice volume significantly exceeds
  historical average. "Your body and brain need recovery time to consolidate what
  you've learned."
- **Time vs planned comparison** — post-session summary shows planned duration vs actual.
  Helps musicians calibrate their internal time sense. Particularly valuable for
  musicians with ADHD-related time blindness.
- **Quick close option** — save scores captured during practice, skip the detailed
  summary. Respects that after 45 minutes of practice, some musicians just want to stop.

### Delivers

Musicians can set goals and see evidence that their practice is working. The
encouragement is specific, data-grounded, and tied to the scheduling methodology —
reinforcing trust in the process.

### Dependencies

Phase 3 (scheduling engine — goal-aligned scheduling needs the SRS engine).

---

## Phase 5: Visualisation & Insights (6 weeks)

**Theme: "Trust the evidence"**

The full visualisation suite. These features transform raw practice data into actionable
insight and motivational fuel.

### Features

- **Mastery timeline charts** — per-item and aggregate line charts showing mastery
  improvement over weeks and months.
- **Key coverage heatmap** — circle-of-fifths view showing mastery level per key for any
  exercise. Immediately reveals weak keys.
- **Consistency calendar** — comeback framing: "4 of the last 7 days — great spacing for
  retention." Not streak-focused. Celebrate returns after gaps.
- **Tempo progress charts** — showing tempo increase over time toward target BPM per item.
- **Goal progress dashboard** — percentage completion for active goals.
- **Weekly summary with insights** — comparison to prior week. Shown on next app open
  or via notification.
- **Encouragement configuration** — frequency (every item / session / weekly / off),
  tone (data-focused / warm / minimal), delivery (inline / summary / notification).
  Respects that not all musicians want the same encouragement.
- **Audio recording & playback** — record a run-through, play it back, rate it. Compare
  to a recording from weeks ago. Provides an objective anchor for subjective mastery
  ratings. (**Open question:** browser audio API viability on mobile.)

### Delivers

A musician can look at their dashboard and see concrete evidence of progress. Weak spots
are visible. The sense of competence (SDT) is actively reinforced. The app becomes a
motivational engine, not just a tracking tool.

### Dependencies

Phase 4 (goals — the goal dashboard needs the goal framework).

---

## Phase 6: Collaboration & Intelligence (6+ weeks)

**Theme: "A practice partner who knows you"**

AI-powered features and teacher collaboration. These leverage all the data accumulated
in earlier phases.

### Features

- **Teacher integration (basic)** — teacher shares routines, suggests items for student's
  library, sets target tempos or milestone goals, views progress dashboards (with
  permission). Musician retains full control. This is a *collaboration* feature, not an
  AI feature.
- **Import/export** — share routines with other musicians. Import items from external
  sources.
- **AI setlist generation** — "I have a gig playing 8 standards in 3 weeks — build me a
  practice plan."
- **Pattern recognition** — identify systematic weaknesses (e.g. flat keys, left hand
  passages) from accumulated practice data.
- **Adaptive difficulty** — auto-adjust scheduling aggressiveness based on progress
  patterns over time.
- **Adaptive interleaving** — AI adjusts mixing intensity based on how the musician
  responds to different levels of contextual interference.
- **AI session review** — post-session analysis with rebalancing suggestions.
- **Goal coaching** — help set realistic goals and break them into actionable daily
  practice plans.

### Delivers

Intrada becomes a knowledgeable practice partner — one that understands the musician's
history, patterns, and goals.

### Dependencies

Phase 5 (visualisations — the AI needs the data patterns that the visualisations reveal).

---

## Timeline Summary

| Phase | Theme | Duration | Cumulative |
|-------|-------|----------|------------|
| 1. Practice Quality | Make every session count | 4 weeks | 4 weeks |
| 2. Library Depth | A library that understands music | 4 weeks | 8 weeks |
| 3. Scheduling Intelligence | Practise what matters most | 6 weeks | 14 weeks |
| 4. Goals & Encouragement | See the process working | 4 weeks | 18 weeks |
| 5. Visualisation & Insights | Trust the evidence | 6 weeks | 24 weeks |
| 6. Collaboration & Intelligence | A practice partner who knows you | 6+ weeks | 30+ weeks |

Total: ~30 weeks vs the original 24. More realistic given the additions (onboarding,
tempo-building flow, free practice mode, deep dive mode, lesson capture).

---

## Open Questions

These need decisions before or during implementation:

1. **Metronome: built-in or external?** The tempo-building loop (Phase 3) needs a
   metronome. Building one is non-trivial (accurate timing in WASM). Alternative:
   design the flow to work alongside an external metronome app.

2. **Offline-first architecture.** Vision Principle 6 says "offline-first." The current
   architecture is API-dependent. What syncs? What doesn't? When does sync happen? This
   is an architectural decision that gets harder to retrofit the longer we wait.

3. **iOS native vs web-first.** The vision doc raises this question. For Phases 1–3,
   the web shell is fine. For the tempo loop and audio recording (Phases 3 and 5),
   native APIs would be significantly better. Decision point: before Phase 3.

4. **Audio recording viability.** Browser MediaRecorder API works on desktop but is
   inconsistent on mobile Safari. If audio recording is important, this pushes toward
   native.

5. **Teacher integration timing.** Currently Phase 6. Basic sharing (routines, item
   suggestions) could come earlier without AI. Consider splitting: basic teacher
   features in Phase 4, AI-powered teacher features in Phase 6.

6. **Scoring + tempo coupling.** Should every mastery rating require a tempo? Or is
   tempo optional (only relevant for items with tempo targets)? The student critique
   suggests mastery is tempo-dependent, but not all items have tempo targets.

---

## Critique Integration Log

Changes made based on the product/teacher/student review:

| # | Critique | Resolution |
|---|----------|------------|
| 1 | Feature inventory, not journey | Added onboarding lane, emotional annotations, open questions |
| 2 | First-time user invisible | Added Day 1 onboarding flow with empty-library handling |
| 3 | No lesson → practice bridge | Added lesson capture as onboarding and recurring flow |
| 4 | Phase 2 overloaded | Split into Phases 3 + 4; moved practice quality to Phase 1 |
| 5 | Goals incoherent across diagram | Gave goals their own lane with set → link → track → achieve |
| 6 | Tempo work absent | Tempo-building loop is first-class in Phase 3 practice flow |
| 7 | No deep dive mode | Added alongside interleaved as a session type |
| 8 | Micro-cycle too rigid | Added light path for maintenance items |
| 9 | Sections need more depth | Sections get own mastery + tempo; schedulable independently |
| 10 | Teacher integration too late | Noted as open question; basic sharing could move to Phase 4 |
| 11 | Reflection is homework | Quick close option; rating is 1 tap, notes are optional |
| 12 | Scoring too coarse (no tempo) | Scoring now captures tempo alongside mastery |
| 13 | No free practice mode | Added: timer only, no structure, counts toward consistency |
| 14 | Streak counter contradicts comeback | Rework streak → comeback in Phase 1, not deferred |
| 15 | Offline absent | Flagged as cross-cutting principle + open question |
| 16 | Too linear / not messy enough | Added cross-flow connections, lesson capture as re-entry |
| 17 | No import/share | Added to Phase 6 (teacher integration) |
