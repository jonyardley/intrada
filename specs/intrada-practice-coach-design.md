# Intrada: The Practice Coach

Design document, v2. Captured 18 July 2026. Revised same day after critical review.

## Vision

A practice companion that acts as your teacher between lessons: it decides what you practise, listens while you play, tells you when you are done, gets you unstuck, and adjusts your plan based on what it actually hears. You bring the goal ("improvise confidently on jazz standards"); it handles everything between the goal and today's 20 minutes.

**One-line pitch:** sit down, press start, and practise like someone who knows exactly what they're doing is standing behind you, because functionally, one is.

**The honest scope of the claim:** the app measures the prerequisites of good jazz (right notes, in time, at tempo, in every key) and coaches the user's own judgement of the rest (touch, feel, phrasing). For jazz, the unmeasurable half matters at least as much as the measurable half; the app is authoritative about the former and a structured prompt for the latter. Gates and marketing both reflect this.

## Decisions (recorded, overrulable)

1. **Personal tool first, product decision deferred.** Built for user zero; placement, onboarding, content generalisation, and the audio path are deferred until three months of daily personal use argue for a product. Every scoping call below assumes this.
2. **Click-always.** Practice is always to a metronome or backing track. This is a pedagogical stance (time is the foundation) and a technical necessity (attempt segmentation and bar-level feedback need a shared clock). A stated constraint, not an accident.
3. **Deploy-gates are self-confirmed in v1.** Detecting a transposed, varied device inside free improvisation is research-grade; v1 asks "did you land it?", with machine suggestion at best later.
4. **Horizons are self-trend-based, not population-based.** "Most people need ~10 sessions" requires population data that does not exist at launch; the honest version is "your trend suggests roughly N more sessions".
5. **Content scope for v1 is minimal:** 5 nodes, 2 method packs, 1 phrase, fully specified; the rest of the branch as stubs. The prove-the-loop principle applies to content as much as code.

## The problem

Self-directed learners, especially in jazz, face four compounding failures:

1. **Slow feedback loop.** "Am I doing this right?" goes unanswered for weeks, so effort feels wasted and confidence erodes.
2. **Prescription overwhelm.** The full taxonomy of what to practise (technique, harmony, vocabulary, ear, repertoire, application, listening) is so daunting that most learners default to a narrow comfort zone. The documented failure mode: it's so overwhelming that "young improvisers just run blues scales over everything".
3. **Choosing at the piano.** Deciding what to do next, mid-session, is itself the mental block. Sessions stall, drift into playing what's already known, or die.
4. **Getting stuck means giving up.** Without a teacher's intervention (make it smaller, slower, different), a wall becomes an exit.

The insight that makes this buildable: the feedback loop does not need to be real-time. Feedback within seconds of finishing an attempt solves the actual problem. Post-exercise, not mid-phrase.

## Product principles

1. **Kill the choice.** The plan is made before the user sits down. Open app, one screen, press start. An escape hatch exists; the default is prescribed.
2. **Measure the prerequisites, coach the judgement.** Deterministic scoring of every attempt for what MIDI can hear; structured self-assessment for what it cannot. "Am I doing this right" gets an answer in seconds, honestly scoped.
3. **Make it smaller before they give up.** The stuck ladder is the teacher's core intervention, and it fires automatically on failure patterns.
4. **Honest about the grind.** Grind blocks are labelled as grind, with the payoff, the horizon, and the trend attached. The app never pretends drudge is delight. Trust in its honesty is the retention mechanic.
5. **Every session touches real music.** Hard planner constraint: at least one block per session is new or applied. Effort in the middle, music at both ends.
6. **The taxonomy lives in the data model, never in the UI.** The app thinks about everything so the user thinks about nothing.
7. **Ends on competence, logs everything.** Sessions close with something the user can play. Early exits bank what was done. Quitting doesn't compound.
8. **Prescribe now, graduate later.** A good teacher transfers judgement; the endpoint of lessons is not needing them. The app surfaces its reasoning progressively so the user internalises the planning model, and off-piste grows from exception to peer mode as mastery climbs. Prescribed-forever is a failure state.

## UX design principles

The context dictates everything: the user is seated at a piano, hands on keys, phone or iPad on the music stand, often mid-flow. Every interface decision follows from that.

1. **One screen, one action.** At any moment the app shows exactly what to do now and nothing else. No dashboards mid-session, no navigation. The home screen is a start button.
2. **Glanceable over readable.** Mid-drill feedback must land in under a second of eye contact: tick, cross, one fact, big type. If it needs reading, it waits for a block boundary.
3. **Hands stay on the keys.** Minimal touch interaction during play: auto-advance on gate pass, auto-listen on count-in. The stuck button is the one big target. Voice or pedal-press interaction is a later win worth designing toward.
4. **Plain language, musician's language.** "3 clean passes at 120" not "criterion threshold met". Say "swing feel" and "left hand late", never "temporal deviation metric". Theory terms appear only at the point of use, with the sound attached. No gamification vocabulary (XP, gems, lives): this is a practice room, not an arcade.
5. **Frictionless start, frictionless stop.** Open to press-start in one tap, zero configuration per session. Ending early is one tap and celebrated as banked, never guarded by "are you sure?" guilt dialogues.
6. **Calm, not gamified.** Warm manuscript-and-concert-hall aesthetic (the established Intrada identity), generous type, no badges raining down. The reward is the trend line and the sound of your own playing; the app's tone is a quiet colleague, not a cheerleader.
7. **The why is always one tap away.** Every prescription shows its one-line reason inline; deeper reasoning (graph state, trend) is a tap deeper, never forced. Progressive disclosure carries the graduation arc.
8. **Never bluff, visibly.** Confidence is part of the interface: certain feedback is stated, uncertain feedback asks. Trust is the product; the UI must never spend it.
9. **Design for the bad day.** Tired, low-motivation, ten minutes: the interface must feel lighter on those days, not heavier. Shorter session offered by default, wins front-loaded, zero admin.
10. **Music-stand ergonomics.** Readable at arm's length in practice-room lighting, works in both orientations, count-ins audible over the piano, no interactions that require precision tapping while seated at an instrument.

### Behavioural research alignment

Each design choice above sits on an established finding. The mapping, so the principles survive future design debates:

- **Fogg Behaviour Model (B = MAP) and choice overload (Iyengar & Lepper).** When motivation fluctuates, reducing ability barriers is the strongest lever, and more options reduce action. Grounds: one-tap start, zero per-session configuration, kill-the-choice, the single stuck button.
- **Cognitive load theory (Sweller).** Extraneous load steals working memory from learning; at an instrument, every word on screen competes with the ears. Grounds: glanceable feedback, one screen one action, one thinking cue per rep.
- **Flow (Csikszentmihalyi).** Clear goals, immediate feedback, challenge matched to skill. The session design is a flow machine: gates are the clear goals, per-rep ticks the immediate feedback, the difficulty setpoint the challenge-skill balance.
- **The guidance hypothesis (Salmoni, Schmidt & Walter).** In motor learning, feedback after every attempt can breed dependence and hurt retention; reduced-frequency feedback often retains better. Grounds a tunable: per-rep ticks may thin out as a drill matures (summary feedback every few reps), another reason feedback cadence is data, not code.
- **Peak-end rule (Kahneman & Redelmeier).** Experiences are remembered by their peak and their end, not their average. Grounds: every session closes on real music the user can play; the memory of practice is competence, so returning feels good.
- **Progress principle (Amabile & Kramer) and small wins.** Visible progress on meaningful work is the single strongest daily motivator. Grounds: trend lines, per-rep ticks, banked early exits, the narrated thread from drill to music.
- **Endowed progress effect (Nunes & Dreze).** People persist more when they can see progress already made. Grounds: the warm-up counts, gate progress shows "2 of 3" not "1 to go from zero".
- **Goal gradient (Hull; Kivetz).** Effort accelerates as the goal nears. Grounds: visible within-block gate progress and short horizons ("2 more keys this week") over distant ones.
- **Self-determination theory (Deci & Ryan).** Intrinsic motivation runs on competence, autonomy, and relatedness; extrinsic rewards can crowd it out (overjustification effect). Grounds: gates and trends (competence), off-piste mode, the overrulable plan and the graduation arc (autonomy), the coach's voice and normalisation (relatedness), and the outright ban on XP, gems, and badge showers.
- **Implementation intentions (Gollwitzer).** When-then plans roughly double follow-through versus intentions alone. Grounds: tomorrow's session is drafted before the lid closes; the app's real product is a standing when-then ("when I sit down, I press start").
- **Habit formation (Lally; Wood).** Habits form through context-stable cues and low-friction repetition, over weeks to months, and missing one day does not derail formation. Grounds: same-slot session prompts, streaks that count any engagement, and no catastrophising a missed day.
- **Loss aversion, handled with care (Kahneman & Tversky).** Loss framing is powerful and curdles: a broken streak framed as loss becomes a quit event. Grounds: progress framing over loss framing, soft-landing exits, streaks that bend rather than break.
- **Self-efficacy (Bandura).** Belief in one's capability is built by mastery experiences, vicarious experience, and credible verbal persuasion. Grounds: front-loaded wins, "everyone's enclosures sound mechanical for three weeks" (normalisation is vicarious experience), and feedback that cites evidence rather than cheerleads.
- **Growth mindset and attribution (Dweck).** Praise for process and strategy sustains effort; ability labels create fragility. Grounds: the voice praises trends and tactics ("the tempo drop worked"), never talent, and frames walls as stages, not verdicts.
- **Zeigarnik effect.** Open loops pull attention back. Grounds: the end-of-session summary always names what is next and in flight, so the phrase mid-pipeline is an itch the user returns to scratch.

The pattern worth noticing: the gamification playbook (points, streak-loss, badges) borrows the shallow half of this literature and skips the half that matters for learning (competence, autonomy, flow, self-efficacy). Building on the deeper half is itself a differentiator.

## The pedagogy model

### Activity taxonomy (seven categories, engine room only)

1. **Technique and foundations.** Scales, modes, arpeggios, digital patterns (1-2-3-5 cells), finger independence.
2. **Harmony and voicings.** Diatonic 7ths, ii-V-I in all keys, extensions, rootless voicings (A and B positions). Pipeline: shells, rootless, upper structures.
3. **Vocabulary acquisition.** The full lick pipeline: learn, keys, analyse, extract, generate, integrate. Imitate, assimilate, innovate. The endpoint is not quoting; the essence sinks in and feeds your own voice. See "Theory and vocabulary" below for the back half of the pipeline.
4. **Ear training and transcription.** Interval and chord-quality recognition, hearing voicings (3rds and 7ths first), micro-transcription before whole solos. Note: lick transposition is micro-transcription digested; these are the same node family.
5. **Repertoire (tune pipeline).** Form, melody, changes from memory, comping, solo arrangement.
6. **Application and integration.** Restricted improv, guide-tone lines, applying the current lick over the current tune, rhythmic constraints.
7. **Listening.** Structured assignments (album per week, targeted listens). Needs no instrument: the commute slot, and the streak-saver.

### Two mechanisms underneath

- **Skill graph.** Jazz improvisation as a dependency graph, not a list. Each node: what it is, why it matters, prerequisites, 2-4 drills, machine-checkable done-criteria. Mastery is (estimate, confidence), never a boolean.
- **Pipelines.** Tunes and phrases are instances moving through stages.
  - Tune pipeline: form, melody, shells, rootless, arpeggiate changes, guide tones. Clear "done" gate per stage.
  - Lick pipeline: learn, nearby keys, full cycle, analyse, extract, generate, integrate over the tune. One phrase in flight at a time, a few minutes per session over a fortnight.
  - Caps on in-flight items (one phrase, one or two tunes) are the anti-overwhelm feature.
- **The intersection is the magic.** Skill drills are parameterised by the current tune: "enclosure drill over Strasbourg's changes". The tune is the vehicle; the skill is the cargo.

### Method packs (how to attack, not just what)

Every drill type ships with:

1. **Decomposition recipe.** The ladder up: hands separate, one key slow, then the transitions ("practise the change, not the chords"), then add the right hand, then tempo. The stuck ladder is this recipe read in reverse.
2. **Key-traversal strategy.** Whole steps while learning; around the cycle of fourths for fluency (the ii-V motion chains keys into one continuous exercise); random-key retrieval as proof. The ordering progression is itself a gate sequence.
3. **Thinking cues.** One per stage, not a lecture. Shape thinking ("see the A voicing as a shape off the 3rd"), minimal motion ("find the fingers that don't move first"), anticipation ("hear the next chord before your hands leave this one").
4. **The all-in-one-go schedule.** "All 12 keys" is never a session goal: 2-3 new keys per session, prior keys as maintenance passes, full-cycle-clean as a node gate reached over weeks. The app carries the schedule.

Method packs are authored content (Markdown-friendly) and, with the graph, they are the moat. Anyone can list drills; the packs are a teacher's craft, encoded.

### Theory and vocabulary (just-in-time, never a curriculum)

Principle: theory is taught at the point of use, driven by the phrases the user already loves. Nobody needs a modes course; they need to know why this phrase works, right now, while it is under their fingers. The theory graph builds itself lazily; motivation is pre-solved because the app is explaining a sound the user already wants.

The back half of the lick pipeline:

1. **Analyse: why does it sound like that?** The app knows the notes and the chord context, so it annotates automatically ("b9 over the V, resolving down a semitone to the 5th; that crunch-then-release is the sound you like"). Interval-against-chord labelling is deterministic Rust; the prose is LLM. Each annotation links to an optional 90-second micro-lesson, shown only if tapped.
2. **Extract: the device, not the lick.** Every lick decomposes into reusable devices (an enclosure shape, a b9-to-5 resolution, a rhythmic displacement). The device is the actual vocabulary item; the lick was its carrier. The user's device inventory is their musical vocabulary, visible and growing: a strong progress surface.
3. **Generate: make it yours.** A graded constraint ladder, each rung gateable:
   - Vary one parameter: same device, change the rhythm; same rhythm, change the target note.
   - Transplant: apply the device over a different chord type, then a different tune.
   - Compose: write or improvise-and-capture two original phrases using the device.
   - Deploy: improvise over the form with one instruction, "land this device once per chorus, anywhere". Self-confirmed in v1 (see Decisions); transposition-and-variation-tolerant motif detection in free improvisation is research-grade, so machine checking is a later, confidence-weighted suggestion at best. Deploy-on-demand is the real test of vocabulary: retrieval under improvisational load, not recitation.

Session placement: analyse and extract are light, even instrument-free (a two-minute "why this works" read is a listening-slot item, prepping tomorrow's generate drill). Generate drills take the integration block's slot on lick-pipeline days. And the narrated thread gains its strongest line: "you just used the b9 resolution from the Rollins lick, unprompted, in bar 12" is evidence of acquired language, the whole point.

## Session design

### Done-criteria, not minutes

Each block has an exit condition ("3 clean passes at 120", "resolves on beat 1 in all 4 keys, twice") with a time ceiling so nothing spirals. When the gate passes, the app moves on: no lingering on what's already fine.

### The template

Something comfortable, then something hard (possibly grind, honestly labelled), then something new or applied, then something musical to close.

Worked example, 20 minutes on Strasbourg / St. Denis:

- 0-3: warm-up. Mastered pattern over the changes. Scored silently. Longer term this becomes the daily diagnostic (a scrappy warm-up drops today's frontier tempos before any failure is felt), but that flex needs weeks of baseline data first: a Phase 4 feature, plain warm-up until then.
- 3-9: frontier drill from whichever category is due (rootless voicings, A section). Gate: 3 clean passes. Per-rep ticks, escalation armed.
- 9-14: lick pipeline advance. Current phrase into its next key over the turnaround. Gate: 2 clean in both keys.
- 14-19: integration. Restricted improv over the form using the current lick, chord-tone targets. Softer gate, self-rated feel alongside metrics.
- 19-20: close. Play the head, sound good, stop. Summary and tomorrow's draft written before the lid goes down.

Every block carries: a one-line why (citing graph state), a gate, a stuck path, and state written back.

### Feedback cadence

- **Per rep:** glanceable only. Tick, cross, one fact ("late into bar 3"). Hands stay on keys.
- **Block boundaries:** the coaching voice. Why this drill, what to listen for, the trend.
- **Escalation on pattern, not on single failure.** Three fails triggers the ladder unprompted: drop tempo 20%, shrink scope (A section, one hand, one key), change mode (sing it, tap it, listen), swap the drill. No ceremony, no guilt.
- **Name the wall.** At the quit-point, normalise plus a smaller step: "everyone's enclosures sound mechanical for the first three weeks; here's the smaller version."

### Anti-abandonment

- First block is always a warm, low-stakes win (90%+ success rate). Momentum before difficulty.
- The comfort loop (playing what you know) is legitimised: it is the opener and closer, time-boxed. Frontier work sits in the middle, sandwiched by things that feel good.
- "End early" banks what was done and keeps the streak. A shortened session recorded beats an abandoned one; abandoning must never be what the app teaches.
- Off-piste mode: wandering is honoured, still listened to and logged, feeding diagnosis without a plan. End-of-wander prompt: "keep this as a drill?" Some wanders are the graph revealing a gap.

## Learning science (the optimisation levers)

A note on epistemic honesty: these mechanisms are directionally well-supported but the specific numbers are softer than app-copy usually admits. The design uses them as tunable setpoints, not laws.

1. **Interleaving over blocking.** Rotating drills in shorter blocks beats long single-drill blocks on retention, though it feels worse in session and effect sizes vary by task. The planner interleaves by default: the genuine edge, because self-directed learners never do it.
2. **Spacing.** Expanding-interval review per node; reviews are quick gated checks, not full blocks. Keys, tunes, and drills all decay. SM-2 is a declarative-memory algorithm and motor-skill decay behaves differently, so intervals are a starting heuristic to be recalibrated against observed decay, not gospel.
3. **Desirable difficulty.** Target roughly 80-85% success per drill as the initial setpoint. The oft-cited 85% figure comes from perceptual and machine-learning tasks; its transfer to complex motor skill is plausible, not established. What is solid: with scored attempts the success rate is measurable, so tempo, key count, and constraints auto-adjust to hold whatever band proves right. The closed loop is the feature; the number is a parameter.
4. **Velocity, not just position.** Rate of improvement per node. Three flat sessions triggers a change (level down, different drill, attack a prerequisite). Also the honest progress view: "enclosures 90 to 140bpm this month."
5. **Horizon planning.** Goal back-propagates through the graph into a weekly rhythm, then daily sessions. Plans re-derive weekly from current state; a bad fortnight adjusts the forecast, never breaks the plan.
6. **Consistency beats intensity.** Twenty minutes daily beats two hours on Sunday. The planner optimises for streak-friendliness: sessions that fit the time actually available.

## Motivation model

People return to things where they can feel themselves getting better at something they care about.

- **Feel:** visible trends, per-rep ticks, weekly velocity.
- **Getting better:** gates, the difficulty setpoint, the placement of frontier work.
- **Care about:** applied to real tunes, payoff framing, the narrated thread ("that voicing drill from Monday is what your left hand just did in bar 5").

Streak mechanics, defanged: any engagement counts (a listening block, five minutes), progress framing over loss framing, soft-landing exits as a first-class flow. Mechanics are designed from the failure stories (missed three days, failed a gate five times, quit at minute eight), not the success stories.

## Technical architecture

No model training required. No AI engineering required. Orchestration and pedagogy are the work.

| Layer | Where | What |
|---|---|---|
| MIDI analysis | On-device, Crux core (Rust) | Deterministic: wrong notes vs target, timing, swing ratio, tempo drift, gate detection. No AI. ~60% of the "listening teacher". |
| Audio transcription | On-device, CoreML | Basic Pitch (Spotify, open source) converts acoustic audio to note events, feeding the same Rust analysis. Used, not trained. |
| Coaching voice | Cloud LLM via Axum proxy | Claude/GPT. Input: metrics plus session and graph state. Output: the teacher's voice. Prompt engineering, cached, degrades gracefully offline. Never in the critical path. |
| Planning and prescription | Backend Rust, state in Turso | Graph traversal, spacing, difficulty setpoint, session weave. Deliberately deterministic: testable, explainable, no hallucinated pedagogy. |

Stack fit: Crux core holds the session state machine, MIDI event stream, and analysis functions. Swift shell handles CoreMIDI capture and AVAudioEngine plus Basic Pitch. Axum proxies the LLM and serves planner endpoints. Turso holds graph state and session history. MIDI events are just events; the event-sourced model extends naturally.

Data model sketch: `Tune` (pipeline stage), `Skill` (graph node, mastery as estimate plus confidence), `Phrase` (per-key mastery, pipeline stage), `Device` (reusable musical device, referenced by many phrases, with theory annotation and its own mastery via the generate ladder), `Drill` = (skill x tune x parameters) referencing a `MethodPack` (decomposition stages, traversal orderings, cues by stage, escalation overrides), `Gate` criteria as tunable data (JSON/TOML), session planner as a pure function of (tune stages, skill mastery, pipeline states, available minutes).

Mastery update function (first crude version, to be refined): per (node, parameter level), attempt outcomes update a Beta distribution (pass/fail as evidence), giving estimate and confidence for free; confidence decays with time-since-practice to drive spacing; level-ups shift the distribution rather than resetting it. Whatever the final form, this function is the planner's engine and must be specified before Phase 2, not discovered inside it.

Custom model test: fine-tuning makes sense only when off-the-shelf fails the task. Frontier LLMs know jazz pedagogy; transcription models handle piano. The proprietary asset is structure (graph, criteria, method packs), injected as context. Revisit on-device coaching models at v3, not before.

Cost: a few LLM calls per session is pence per user per month.

## Design challenges and mitigations

1. **Attempt segmentation and time alignment (the hardest technical problem).** "Late into bar 3" requires a shared clock and knowing which notes constituted the attempt: count-in handling, restart detection, separating noodling from attempts, mid-attempt collapses. This, not MIDI capture, is the real Phase 1 risk, and it is why click-always is a recorded decision. Mitigation: a named spike before any scoring UI is built.
2. **Pedagogical authority without a teacher's judgement.** MIDI hears notes and milliseconds, not touch, feel, or phrasing, and for jazz the unmeasurable half matters at least as much. Risk: gating on the measurable trains mechanical playing, while the app hands the more important half back to the user who was the unreliable judge to begin with. Mitigation: the measured-prerequisites framing throughout; gates check the measurable, the voice structures self-assessment for the rest ("did that sound like the record? Rate the feel"). Self-report is a legitimate signal.
3. **The content burden.** Nodes x drills x criteria x method packs x cues x micro-lessons is a textbook's worth of authored material, validated against one learner. "The moat" is also a euphemism for hundreds of authoring hours. Mitigation: the minimal-content decision (5 nodes, 2 packs, 1 phrase for v1); author breadth only after the loop is proven.
4. **Done-criteria that don't lie.** Too strict: grind and quit. Too loose: mastery is fiction. Mitigation: criteria as tunable data, failure-to-pass rates instrumented from day one, permanent calibration expected.
5. **Prescription trust.** One arbitrary-feeling prescription and users revert to noodling. Mitigation: every block carries its why, citing node state. Explainability is a feature requirement and the argument for deterministic planning.
6. **Rigidity vs the texture of practice.** Mitigation: off-piste mode, and the graduation arc (principle 8). Structure as default, never as cage.
7. **Cold start for user two.** Mitigation: placement session, aggressive early re-planning, surfaced humility ("early days, still learning your level"), uncertainty in the mastery model from the start. Parked behind the personal-tool-first decision.
8. **The audio measurement gap.** A wrong "wrong note" is instant trust damage. Mitigation: confidence-weighted feedback; when uncertain, ask ("that sounded clean, agree?"). Never bluff.
9. **Motivation mechanics that curdle.** Mitigation: design from failure stories; streaks count any engagement; soft landings; horizons framed from the user's own trend, never invented population statistics.
10. **The mundane one: MIDI setup UX.** Which cable, which piano, iOS quirks. For any user beyond user zero, this is where people actually fall off, before pedagogy ever matters. Mitigation: parked with the product decision, but not forgotten.

Pattern: almost none of these are engineering problems. They are judgement encoded as data: tolerances, criteria, phrasing, when to bend.

## Build plan

**Phase 0: paper teacher (1-2 weeks, no code).** Fully specify the minimal content set: 5 nodes, 2 method packs, 1 phrase (with stage and per-key state), Strasbourg's tune-pipeline position, and gate criteria as data. Sketch the rest of the improvisation branch as stubs only. Run own practice from it for a fortnight. Deliverables are the content set plus four logs:

- Divergence log: machine-score vs felt-score per drill, one line where they differ (specs the self-assessment boundary).
- Gate-attempt log: pass/fail plus felt difficulty (first tolerance numbers, criteria schema shape).
- Why log: every block's one-line why citing node state (validates the graph; a why that can't be written is a missing node).
- Wander log: off-plan time, triggers, "keep as drill?" (sizes off-piste mode).

Plus: write the failure stories and desired app responses; note what the app should have done on the inevitable bad day; write the placement session as a thought experiment and check the criteria aren't jon-shaped.

**Phase 1: the listening gate (3-4 weeks).** Opens with the named spike: attempt segmentation and click alignment (count-in, restart detection, noodling vs attempts, collapse handling) proven against real playing before any scoring UI exists. Then: CoreMIDI capture into the Crux core; deterministic analysis; one screen (drill, live scoring, gate progress). First drill type: lick transposition (target exactly known, scoring unambiguous, exercises the Phrase model, user-zero motivation built in). Shells second, proving generalisation. Alongside: an honest hour surveying current competitors (Yousician-class products have quietly added more adaptivity than "content libraries" suggests). The moment to reach: play, see the tick, feel the gate release you.

**Phase 2: the prescribed session (3-4 weeks).** Planner as a pure function; press-start flow; stuck button with the mechanical ladder; soft-landing exit; listening blocks as prescribable no-instrument items; state into Turso. Exit criterion: two weeks of daily use without reverting to self-directed sessions.

**Phase 3: the voice (2-3 weeks).** LLM behind Axum: summaries, whys, stuck-moment coaching (normalise plus smaller step). Deliberately late: coaching prose over unmeasured practice is a chatbot, not a teacher.

**Phase 4: widen (ongoing).** More drill types, spaced repetition, difficulty auto-adjustment, velocity tracking, ear-training interaction loop, Basic Pitch audio path (preceded by a one-afternoon spike: Basic Pitch vs MIDI ground truth from the same performance, yielding real error rates per drill type), placement diagnostics, second user (the graph needs a second data point before it is a curriculum).

**Sequencing calls (overrule as needed):** MIDI before audio; LLM last, not first; no placement until Phase 4.

**Effort:** evenings and weekends alongside COAST's September push. Phases 0-2 by late autumn, voice by year end. Phase 0 is also the actual practice regime: research time is piano time. Strasbourg is the test fixture.

## Competitive position

Content libraries with linear courses (Tonebase, Piano Marvel, iReal) are not mastery-based graph traversal (Duolingo, Math Academy). Nobody has done the latter properly for jazz piano. Yousician can hear notes; nobody has built the thing that gets you through minute eleven when you want to walk away. The moat is the pedagogy structure (graph, criteria, method packs), authored from inside the exact problem.

## Next step

One action starts everything: open a Markdown file, write the first 5 nodes and the gate criteria for the current phrase, and practise from it tomorrow. Phase 0 needs no code, no design, and no further revisions to this document. The doc is done; the fortnight is the work.
