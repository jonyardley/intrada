# Intrada: The Practice Coach

Design document, v5. Captured 18 July 2026; revised same day after critical
review. v2 (2 Aug 2026): the measurement-validity decisions, the feedback
choreography, the fluency frame. v3 (3 Aug 2026): intent promoted to a
first-class mechanism alongside the graph and the pipelines, after the
question the earlier drafts couldn't answer — *if the app kills the choice,
how does it know what I want to work on right now?* See "Intent: goals,
campaigns and steering". v4 (3 Aug 2026): the non-failure states. A critical
review found the design managed failure well and everything else barely —
so fixation, time-orientation and the pull of grind are now designed rather
than assumed, and the click-always constraint is scoped to what it actually
governs. v5 (3 Aug 2026): mastery settled — measured-only, with self-report
moved to a separate judgement track that closes targets but never unlocks
prerequisites (decision 17), which unblocks the planner's engine.

## Vision

A practice companion that acts as your teacher between lessons: it decides what you practise, listens while you play, tells you when you are done, gets you unstuck, and adjusts your plan based on what it actually hears. You say where you're going — at whatever altitude you happen to know it, from "improvise confidently on jazz standards" down to "more transcription this week" — and it turns that into today's 20 minutes. **You own the destination; it owns the route.**

**One-line pitch:** sit down, press start, and practise like someone who knows exactly what they're doing is standing behind you, because functionally, one is.

**The honest scope of the claim:** the app measures the prerequisites of good jazz (right notes, in time, at tempo, in every key) and coaches the user's own judgement of the rest (touch, feel, phrasing). For jazz, the unmeasurable half matters at least as much as the measurable half; the app is authoritative about the former and a structured prompt for the latter. Gates and marketing both reflect this.

## Decisions (recorded, overrulable)

1. **Personal tool first, product decision deferred.** Built for user zero; placement, onboarding, content generalisation, and the audio path are deferred until three months of daily personal use argue for a product. Every scoping call below assumes this.
2. **Click-always.** Practice is always to a metronome or backing track. This is a pedagogical stance (time is the foundation) and a technical necessity (attempt segmentation and bar-level feedback need a shared clock). A stated constraint, not an accident. Sparse-click modes (2-and-4, click every other bar) are gate levels *within* click-always: they convert the internal-time objection (Galper, Longo) into a difficulty ladder rather than a rebuttal. **Scoped (3 Aug 2026): click-always governs *scored attempts*, not the learning rungs beneath a gate.** Singing a phrase, finding notes in a new key hands-alone, working out a shape slowly — the method packs already prescribe these without a click, and the constraint was never meant to forbid them. This is a clarification, not a weakening: playing against a steady pulse *is* swing (the lay-back, the ratio, playing ahead are all defined relative to a grid), which is why decision 6 scores consistency rather than deviation, and why rubato is correctly deferred until time is internalised rather than treated as a missing feature.
3. **Deploy-gates are self-confirmed in v1.** Detecting a transposed, varied device inside free improvisation is research-grade; v1 asks "did you land it?", with machine suggestion at best later.
4. **Horizons are self-trend-based, not population-based.** "Most people need ~10 sessions" requires population data that does not exist at launch; the honest version is "your trend suggests roughly N more sessions".
5. **Content scope for v1 is minimal:** 5 nodes, 2 method packs, 1 phrase, fully specified; the rest of the branch as stubs. The prove-the-loop principle applies to content as much as code.
6. **Score timing consistency and trend, never absolute deviation** (added 2 Aug 2026 after the measurement-validity review). Skilled jazz timing is systematically off the grid on purpose — a stable ~30ms lay-back reads as swinging *harder* — so only variance and drift are errors. "Late into bar 3" is a finding only when the lateness is inconsistent.
7. **Transport-tiered scoring** (added 2 Aug 2026). Wired MIDI (±1–3ms) earns fine timing feedback; Bluetooth MIDI (±10–20ms connection-interval jitter) gets note accuracy, swing, and trends only — and the app says why on screen. Never issue a precision verdict the input can't support: the never-bluff principle applied to the input path.
8. **Swing ratio is reported, never graded** (added 2 Aug 2026). Robustly measurable from beat-aligned MIDI, but the "correct" ratio varies legitimately by tempo (≈3.5:1 slow falling toward 1:1 fast), player, and phrase. Tempo-adjusted display ("3.1:1 — spacious for this tempo"), no pass/fail.
9. **Intent is declared at three altitudes, and every altitude is optional** (added 3 Aug 2026). Goal (months), campaign (a fortnight-ish destination — "restricted improv over Strasbourg"), today's steer ("more transcription", "hands only, I'm tired"). Each carries a working default, so a user who declares nothing still gets a session, and each is overrulable without ceremony. The rejected alternative was inferring intent from behaviour alone: it makes the app guess at the one thing the user reliably knows.
10. **Declared intent sets the destination; the planner owns the route** (added 3 Aug 2026). Intent resolves by back-chaining through prerequisites, never by literal obedience. Declare restricted improv on weak shells and the answer is "then we start with shells — roughly five sessions from here", honouring the destination while stating the distance (decision 4). A planner that simply does as it is told is a worse teacher than one that argues.
11. **No mode switch** (added 3 Aug 2026). There is no "guided vs my own way" toggle. Asking which mode reintroduces precisely the choose-at-the-piano problem the design exists to remove (see The problem, 3). The app always opens prescribed and yields in one tap: the deviation is available, never solicited.
12. **The LLM may interpret intent; it may never plan the session** (added 3 Aug 2026). Turning "I want to sound like Bill Evans on ballads" into candidate nodes, tunes and phrases is fuzzy linguistic work, cheap to check and visibly editable — good LLM territory. Choosing today's blocks stays deterministic: every block owes a testable why (challenge 5), and prescription trust cannot survive a hallucinated route.
13. **Targets the graph cannot express are accepted, not forced** (added 3 Aug 2026). A declared campaign may contain items that map to no node — a teacher's "make the bridge sing", "sort out your time in the last A". These are kept as opaque targets: prescribable, self-confirmed rather than scored (the deploy-gate stance, decision 3), and never crammed into an ill-fitting node to preserve the appearance of coverage. Recurring unmatched targets are the authoring queue — they say which node to write next on evidence of real demand rather than taxonomy completionism (challenge 3).
14. **Fixation is a first-class signal, met with permission rather than correction** (added 3 Aug 2026). The design managed failure thoroughly and the non-failure states not at all: the escalation ladder fires on three fails, the velocity trigger on three flat sessions, and neither notices a learner who is succeeding and cannot leave. Circling is detected from data the session already holds, surfaced at most once per session, and phrased as leave granted rather than a fault. Fixation usually arrives with shame attached; an app that adds to it gets closed.
15. **Orientation is never withheld; only judgement is rationed** (added 3 Aug 2026). "Done-criteria, not minutes" is right pedagogically and hostile to poor time perception when the clock is hidden too. Scores and verdicts are rationed by the feedback choreography. *Where you are* — time elapsed, blocks remaining, this block's ceiling — is never rationed, because withholding it turns a gate into an open-ended demand. This resolves the apparent conflict with UX principle 1's ban on mid-session dashboards: a bound is not a dashboard.
16. **Not everything is data** (added 3 Aug 2026). Off-piste is listened to and logged, which is right for diagnosis and wrong as the only option. A genuinely unmonitored mode exists — time logged, nothing scored, nothing inferred — because a practice tool that cannot be played without being measured has stopped being a practice room.
17. **Measured mastery is the only mastery; the judgement track closes but never unlocks** (added 3 Aug 2026, settling the question of whether self-report counts toward mastery). Both halves of musicianship matter — execution, and everything MIDI cannot hear — but they must not share one number, because two numbers competing to answer "how good am I at this?" is how the model becomes incoherent and which one the planner obeys becomes arbitrary. So: **mastery is measured-only**, and self-report lives in a separate **judgement track** recording completion and trend, never a level, named differently everywhere so the word stays unambiguous. Its power is deliberately asymmetric — a self-confirmed verdict may *retire* a target ("the last A sings now": stop prescribing it, bring it back on a spaced interval) but may never *satisfy a prerequisite*. If your own verdict could unlock the next thing, generosity would be quietly incentivised and the gate would stop meaning anything. Two consequences: the judgement track needs **no statistical model** to begin with (completion plus a trend line is the entire requirement), and self-judgement is already in the loop early via predict-then-reveal — so what is deferred is the model, not the practice. How much weight the track eventually earns is answered with evidence by the Phase 0 divergence log, not guessed now.

## The problem

Self-directed learners, especially in jazz, face four compounding failures:

1. **Slow feedback loop.** "Am I doing this right?" goes unanswered for weeks, so effort feels wasted and confidence erodes.
2. **Prescription overwhelm.** The full taxonomy of what to practise (technique, harmony, vocabulary, ear, repertoire, application, listening) is so daunting that most learners default to a narrow comfort zone. The documented failure mode: it's so overwhelming that "young improvisers just run blues scales over everything".
3. **Choosing at the piano.** Deciding what to do next, mid-session, is itself the mental block. Sessions stall, drift into playing what's already known, or die. The precise fault is *when* the choosing happens, not that it happens at all: seated at the instrument, tired, with the whole taxonomy in view, is the worst available moment to make a decision a calmer moment last week could have made well. This distinction is what makes prescription compatible with control (see Intent, below).
4. **Getting stuck means giving up.** Without a teacher's intervention (make it smaller, slower, different), a wall becomes an exit.

The insight that makes this buildable: the feedback loop does not need to be real-time. Feedback within seconds of finishing an attempt solves the actual problem. Post-exercise, not mid-phrase.

## Product principles

1. **Kill the choice at the piano; keep it at the right altitude.** The plan is made before the user sits down: open app, one screen, press start. This is not choice denied but choice *relocated* — to the altitude and the moment where the user has the information to choose well (a goal set once, a campaign named for the fortnight, a steer given at the door). Prescription is the rendering of intent already declared, which is why the default can be followed blindly and still be the user's own.
2. **Measure the prerequisites, coach the judgement.** Deterministic scoring of every attempt for what MIDI can hear; structured self-assessment for what it cannot. "Am I doing this right" gets an answer in seconds, honestly scoped.
3. **Make it smaller before they give up; make them stop before they burrow.** Two mirrored interventions, both automatic. The stuck ladder fires on failure patterns and shrinks the task. The circling check fires on fixation patterns — reps after the gate opened, attempts far above your own norm, one node eating the fortnight — and grants permission to leave. A teacher does both without being asked; an app that only does the first will happily watch you spend a month on one voicing.
4. **Honest about the grind — and structurally kind to it.** Grind blocks are labelled as grind, with the payoff, the horizon, and the trend attached. The app never pretends drudge is delight. But honesty is respect, not a mechanism, and a plan that relies on willpower to get through twelve keys of voicings will lose to any brain that runs on interest rather than importance. So three structural moves carry what labelling cannot: grind is **capped shorter** than a neurotypical plan would set it (two brief exposures beat one long one, and the planner already interleaves); grind is **always parameterised by the current tune**, never practised in the abstract, because the same voicings over changes you love are a different experience; and the user may **explicitly trade** it ("skip the grind today, two blocks tomorrow") as a logged deal, which beats the silent skip that becomes three weeks off.
5. **Every session touches real music.** Hard planner constraint: at least one block per session is new or applied. Effort in the middle, music at both ends.
6. **The taxonomy lives in the data model, never in the UI.** The app thinks about everything so the user thinks about nothing.
7. **Ends on competence, logs everything.** Sessions close with something the user can play. Early exits bank what was done. Quitting doesn't compound.
8. **Prescribe now, graduate later.** A good teacher transfers judgement; the endpoint of lessons is not needing them. The app surfaces its reasoning progressively so the user internalises the planning model, and off-piste grows from exception to peer mode as mastery climbs. Prescribed-forever is a failure state. The graduation ladder is concrete rather than atmospheric: the user declares at progressively finer altitudes over time (goal → campaign → today's steer → the block itself), and the planner's reasoning moves from trusted to visible to jointly authored.

## UX design principles

The context dictates everything: the user is seated at a piano, hands on keys, phone or iPad on the music stand, often mid-flow. Every interface decision follows from that.

1. **One screen, one action.** At any moment the app shows exactly what to do now and nothing else. No dashboards mid-session, no navigation. The home screen is a start button. It never asks which mode the user is in (decision 11): it opens prescribed and yields when pushed.
2. **Glanceable over readable — and orientation is not feedback.** Mid-drill feedback must land in under a second of eye contact: tick, cross, one fact, big type. If it needs reading, it waits for a block boundary. Separately and always available: where you are. Time elapsed, blocks left, this block's ceiling — ambient, unstyled, carrying no judgement (decision 15). The thing that must never appear mid-drill is a *verdict*; a bound is orientation, and hiding it is what makes an open-ended gate feel like a demand.
3. **Hands stay on the keys.** Minimal touch interaction during play: auto-advance on gate pass, auto-listen on count-in. The stuck button is the one big target. Voice or pedal-press interaction is a later win worth designing toward.
4. **Plain language, musician's language.** "3 clean passes at 120" not "criterion threshold met". Say "swing feel" and "left hand late", never "temporal deviation metric". Theory terms appear only at the point of use, with the sound attached. No gamification vocabulary (XP, gems, lives): this is a practice room, not an arcade.
5. **Frictionless start, frictionless stop, frictionless override.** Open to press-start in one tap, zero configuration per session. Ending early is one tap and celebrated as banked, never guarded by "are you sure?" guilt dialogues. Steering the session is the same cost: one tap to say "something else today", offered as the planner's own next candidates rather than a blank canvas, and never accompanied by a warning about breaking the plan.
6. **Calm, not gamified.** Warm manuscript-and-concert-hall aesthetic (the established Intrada identity), generous type, no badges raining down. The reward is the trend line and the sound of your own playing; the app's tone is a quiet colleague, not a cheerleader.
7. **The why is always one tap away, and quotes the user where it can.** Every prescription shows its one-line reason inline; deeper reasoning (graph state, trend) is a tap deeper, never forced. Progressive disclosure carries the graduation arc. Where a declared destination exists, the why cites it before it cites the graph: "shells, because you said you want to be improvising over Strasbourg by the 17th, and this is what's between here and there" outperforms any statement about node state.
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

### The fluency frame (added 2 Aug 2026)

From a lesson-notes sketch by user zero's teacher. What comes out as **music**
is the intersection of two circles: the **music in your head** (what you can
hear and imagine) and **technique** (what your hands can execute on demand).
Fluency is the growth of the overlap — either circle alone produces exercises
or frustration.

The frame organises the taxonomy along two axes the flat list doesn't show:

- **Head vs hands.** Listening, transcription, and the front half of the
  vocabulary pipeline grow the head circle; technique, voicings, and harmonic
  patterns grow the hands circle. Vocabulary is the bridge between them — a
  transcribed phrase (head) becomes a device drilled through the keys (hands).
  The lick pipeline is that bridge, built out.
- **At the keys vs away from them.** Listening needs no instrument (already
  the prescribable no-instrument session blocks). Transcription spans both
  modes, and the modes are a difficulty ladder: listen-and-repeat at the keys
  is the entry rung; working the phrase out away from the instrument, hearing
  it fully in the head before playing a note, is the advanced rung — pure
  audiation, no trial-and-error from the hands. That ladder belongs in the
  ear-training method pack, the same shape as sparse-click levels within
  click-always. Everything else is keyboard time making music or building
  the hands.

This is explicitly a portfolio model, not a session template: proficiency
comes from accumulated time across both circles over months, not from every
session touching everything. The planner's interleaving already rotates
categories, but it rotates by node due-ness — nothing detects a starved
circle. A user could pass gates for months with 90% of their time in the
hands circle, and v1's measurable-first content leans exactly that way. The
check this frame adds: a long-horizon time-by-circle view (Track pillar) and a
planner bias when one circle starves. Cheap, because every block is already
tagged with its taxonomy category. **Resequenced 3 Aug 2026:** the *tally* is
arithmetic over tagged blocks and ships in Phase 2, because an app that cannot
see the imbalance will optimise into it for months (challenge 2); only the
planner *bias* waits for Phase 4, when there is enough history to act on.

### Activity taxonomy (seven categories, engine room only)

1. **Technique and foundations.** Scales, modes, arpeggios, digital patterns (1-2-3-5 cells), finger independence.
2. **Harmony and voicings.** Diatonic 7ths, ii-V-I in all keys, extensions, rootless voicings (A and B positions). Pipeline: shells, rootless, upper structures.
3. **Vocabulary acquisition.** The full lick pipeline: learn, keys, analyse, extract, generate, integrate. Imitate, assimilate, innovate. The endpoint is not quoting; the essence sinks in and feeds your own voice. See "Theory and vocabulary" below for the back half of the pipeline.
4. **Ear training and transcription.** Interval and chord-quality recognition, hearing voicings (3rds and 7ths first), micro-transcription before whole solos. Note: lick transposition is micro-transcription digested; these are the same node family.
5. **Repertoire (tune pipeline).** Form, melody, changes from memory, comping, solo arrangement.
6. **Application and integration.** Restricted improv, guide-tone lines, applying the current lick over the current tune, rhythmic constraints.
7. **Listening.** Structured assignments (album per week, targeted listens). Needs no instrument: the commute slot, and the streak-saver.

### Three mechanisms underneath

The graph says what exists and in what order; the pipelines say what is in
flight; intent says where the user is pointing. The planner is the function
that reads all three.

- **Skill graph.** Jazz improvisation as a dependency graph, not a list. Each node: what it is, why it matters, prerequisites, 2-4 drills, machine-checkable done-criteria. Mastery is (estimate, confidence), never a boolean — and it is held per (node, parameter level), not per node: a single node-wide figure is derived for display only, never an input to the planner (see [`intrada-coach-engine.md`](intrada-coach-engine.md) §2).
- **Intent.** Declared destinations at three altitudes — goal, campaign, today's steer — each optional, each defaulted, none of them a route. Detailed below; the short version is that the graph and the pipelines already encode *ordering*, so the user only ever has to name a target.
- **Pipelines.** Tunes and phrases are instances moving through stages.
  - Tune pipeline: form, melody, shells, rootless, arpeggiate changes, guide tones. Clear "done" gate per stage.
  - Lick pipeline: learn, nearby keys, full cycle, analyse, extract, generate, integrate over the tune. One phrase in flight at a time, a few minutes per session over a fortnight.
  - Caps on in-flight items (one phrase, one or two tunes) are the anti-overwhelm feature, and they **outrank declared intent**: a campaign naming three tunes and two phrases is accepted in full and then *sequenced* against the caps, not run in parallel. The user is told which targets are in flight now and which are queued behind them. Silently dropping the excess would break the promise that a declared target is honoured; silently admitting it all would break the one feature that stops the taxonomy swamping the learner.
- **The intersection is the magic.** Skill drills are parameterised by the current tune: "enclosure drill over Strasbourg's changes". The tune is the vehicle; the skill is the cargo.

### Intent: goals, campaigns and steering (added 3 Aug 2026)

v2 left the obvious question unanswered: if the app kills the choice, how does
it know what the user wants to work on *now*? Its answer was thin — "you bring
the goal", plus off-piste for everything else. That leaves the altitude people
actually plan at (the week or fortnight: "learn this tune and get to restricted
improv over it") with nowhere to live, and frames every deviation as an escape
hatch rather than a designed path. Both are fixed by treating intent as a
mechanism instead of a precondition.

One mechanism, read at three altitudes. Each is a *destination*, never a route.

| Altitude | Horizon | Example | Default when unset |
|---|---|---|---|
| **Goal** | Months | "Improvise confidently on jazz standards" | The graph's own frontier |
| **Campaign** | 1–3 weeks | "Restricted improv over Strasbourg" — or a whole lesson's worth of targets at once | Back-chained from the goal |
| **Steer** | Today | "More transcription"; "hands only, I'm tired" | Whatever the campaign makes due |

Six properties make this a mechanism rather than a settings screen:

1. **Declaring a destination is not authoring a plan.** The ordering inside a
   campaign is knowledge the app already holds: the tune pipeline sequences
   form → melody → shells → rootless, and the graph knows chord-tone targeting
   needs shells first because you cannot aim at the 3rds and 7ths until you know
   where they live. So "learn the tune, transcribe, then restricted improv over
   it" requires no authoring — it is what back-chaining from that destination
   produces. The user names the target; prerequisites supply the route. This is
   the single most important consequence: it means granular session-building can
   be dropped without dropping control.
2. **Resolution back-chains, and it argues** (decision 10). Ambition that
   outruns the hands is met with the prerequisite and an honest distance drawn
   from the user's own trend, not with obedience and not with refusal.
3. **Campaigns expire and get reviewed; they do not linger.** A campaign
   carries a horizon, and the weekly re-derivation already in the design is its
   review point. The velocity trigger from the learning-science levers (three
   flat sessions forces a change) applies to a campaign as much as to a node: a
   stalled campaign is surfaced and re-scoped, never silently ground out.
4. **Off-piste is the lowest-commitment declaration, not the absence of one.**
   Wandering is intent expressed behaviourally rather than verbally — which is
   why it is still listened to and logged, and why "keep this as a drill?"
   exists. Some wanders are the graph revealing a gap; some are a campaign the
   user has not yet said out loud.
5. **A campaign holds a *set* of targets, and the canonical source is a
   lesson.** The common real case is not one destination but a handful handed
   over at once: work the rootless voicings, transcribe this Rollins phrase, get
   the bridge from memory. So a campaign accepts several targets, matches each to
   a node or pipeline stage where it can, keeps the rest as opaque targets
   (decision 13), then back-chains and interleaves across the whole set. Nothing
   here requires the teacher to exist in the system — the user writes the list
   and the app translates it. Same mechanism, several endpoints. This is also how
   a user "builds their own path" without hand-assembling blocks: they choose the
   *what*, the graph supplies the order, and the planner spreads it across the
   days they actually have.
6. **The gap read: a diff between the declared set and the current state.** Once
   a target set exists, "what's missing" becomes computable rather than
   rhetorical — in two tiers of confidence that must not be conflated.
   *Structural gaps* are reliable from day one and come straight from the graph:
   unmet prerequisites, and pipeline stages the targets sit behind. *Statistical
   gaps* need history and arrive later: nodes whose confidence has decayed under
   the declared targets, drills whose velocity has gone flat, and the
   starved-circle check from the fluency frame. Show the structural read
   immediately; withhold the statistical read until the data can carry it
   (decision 4 — never invent the numbers). Present either as a route, never a
   verdict: this many sessions, in this order, from here to what you asked for.

What this earns beyond flexibility is a better *why*. A prescription justified
by graph state is defensible; one justified in the user's own words is
persuasive (UX principle 7). Since prescription trust is the retention risk
(challenge 5), citing declared intent is the cheapest mitigation available.

Content implication: the goal and the current campaign are authored data like
everything else, so Phase 0 carries them, and the paper fortnight tests by hand
whether a declared destination actually produces a sane route.

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

The ceiling is **shown, not merely enforced** (decision 15). A gate with a hidden bound is an open-ended demand, and the block that silently ends at eight minutes teaches nothing about pacing. Ceiling and elapsed sit ambient on the drill screen, carrying no verdict — the one thing that never appears there is a score.

### The template

Something comfortable, then something hard (possibly grind, honestly labelled), then something new or applied, then something musical to close.

Worked example, 20 minutes on Strasbourg / St. Denis:

- 0-3: warm-up. Mastered pattern over the changes. Scored silently. Longer term this becomes the daily diagnostic (a scrappy warm-up drops today's frontier tempos before any failure is felt), but that flex needs weeks of baseline data first: a Phase 4 feature, plain warm-up until then.
- 3-9: frontier drill from whichever category is due (rootless voicings, A section). Gate: 3 clean passes. Per-rep ticks, escalation armed.
- 9-14: lick pipeline advance. Current phrase into its next key over the turnaround. Gate: 2 clean in both keys.
- 14-19: integration. Restricted improv over the form using the current lick, chord-tone targets. Softer gate, self-rated feel alongside metrics.
- 19-20: close. Play the head, sound good, stop. Summary and tomorrow's draft written before the lid goes down.

Every block carries: a one-line why (citing the campaign it serves where one is declared, and graph state otherwise), a gate, a stuck path, and state written back.

### Fixation and the circling check (added 3 Aug 2026)

The mirror of the stuck ladder, and the v4 review's headline gap: every
existing intervention triggers on *failure*, so a learner who is succeeding and
cannot leave is invisible. Grinding one voicing for a month, re-running a gate
that opened twenty reps ago, or declaring the same campaign four fortnights
running are all failure-free and all costly.

**Signals, all derivable from data the session already records:**

| Pattern | Signal |
|---|---|
| Can't leave a passing drill | Reps after the gate opened |
| Perfectionism loop | Attempts-to-pass far above your own norm for that gate |
| One node eating the fortnight | Share of session time per node, over sessions |
| Campaign that never lands | Sessions elapsed against the declared horizon |
| Always wandering to the same place | Repeat frequency of a wander target |
| Novelty churn (the inverse) | Campaigns declared and abandoned inside a week |

**Cadence and framing matter more than detection.** At most once per session,
at a block boundary or session end, **never mid-rep** — a fixation notice during
play is both a verdict and an interruption. It is phrased as an observation plus
permission, and it offers the exit rather than demanding it: *"You've run this
fourteen times and passed on the fourth. It's in. Bank it and move?"* Never
"you're stuck on this", never a count of wasted minutes. Fixation typically
arrives with shame already attached, and the anti-nag rules apply in full: say
it once, don't repeat the same observation in a later session, and let a
declined offer stand without a second ask.

The inverse case gets the same treatment in the other direction: churn is
surfaced as a question about the campaign, not a character note about the user.

Thresholds — reps-after-gate, the attempts-to-pass multiple, the time-share
fraction — are gate-criteria data like every other number here, calibrated from
the Phase 0 logs rather than guessed in code.

### Off-piano work is a queue, not a block (added 3 Aug 2026)

Listening, analysis, form-from-memory and the audiation rung need no instrument,
and v3 modelled them as prescribable *session blocks* — which means they get done
at the piano, wasting the one resource they don't need, or not at all. They get
their own queue and their own cue instead, decoupled from the piano session:
commute-sized items, tracked separately, and the streak-saver on a day the lid
never opens. The `away` mode tag already on every node is what populates it, and
the head-circle starvation the fluency frame predicts is far likelier if
off-piano work has to compete for piano minutes.

### Feedback choreography

Expanded 2 August 2026 from the earlier cadence sketch, after reviewing the
augmented-feedback and guidance-hypothesis literature and the shipped-scorer
failure modes (Melodics' praised early/late labels; Playground Sessions'
trust collapse from hidden criteria). The organising rule: **never correct in
real time**. The app is silent while the user plays, gives a one-glance
verdict after each repetition, and saves sentences for the natural rests.
Real-time correction fails on every axis: unreadable while playing, breeds
dependence, and is the naggy failure mode — the backseat driver.

Four layers, organised by *when*, not what:

- **Layer 0 — during play: silence.** The click and a passive beat/bar
  position indicator. No live wrong-note flashes, no score counting up. A
  mid-phrase flash pulls the eyes to the screen, which ruins the next phrase,
  which flashes again: the doom loop. The user's ears own this time.
- **Layer 1 — after each rep: one glance (~1 second).** Tick or cross plus
  at most one fact ("clean", "2 wrong notes", "rushing bars 3–4"). Timing
  facts always say early/late/rushing, never bare "wrong" — actionable
  without thought. Gate progress ticks visibly (● ● ○, "2 of 3"). The next
  count-in starts automatically; hands never leave the keys, nothing to tap
  between reps.
- **Layer 2 — block boundaries: the coaching voice, and it earns its slot.** The
  only place the app speaks in sentences, at the natural rest. Why this drill,
  the trend, and exactly one thing to listen for next. One thought, not a
  paragraph. **Not every boundary** (revised 3 Aug 2026): five blocks in twenty
  minutes would mean five coaching moments, which is a lot of talking from
  something designed not to nag. Default to one per session — where it will land
  hardest, usually the frontier block — plus on demand. **The voice holds no
  reserved slot, and neither does anything else** (settled 3 Aug 2026, closing
  the interruption-budget question): it competes for the session's single
  spoken-moment allowance against the stuck wall, the circling check, the grind
  trade and the gap read, and whatever loses is written into the session summary
  rather than fired late. An earlier draft of this bullet exempted the circling
  check; that exemption is withdrawn, because a cap with one exception is how
  five polite features become four interruptions. Arbitration lives in one place
  in the core — see [`intrada-coach-engine.md`](intrada-coach-engine.md) §7.
  Silence at a boundary is a valid choice and should be the common one.
- **Layer 3 — session end: the narrative.** Trends, the thread ("Monday's
  voicing drill is what your left hand just did in bar 5"), what's next,
  tomorrow's draft. Readable now or in the morning.

Anti-nag rules (where the naggy/useful balance is actually won):

- **A single failure gets a cross and a fact. Nothing else.** No "try
  again!", no tip. Next count-in. One bad rep is not an event.
- **Escalation on pattern, not on single failure — and it acts, not
  narrates.** Three fails triggers the ladder unprompted: drop tempo 20%,
  shrink scope (A section, one hand, one key), change mode (sing it, tap it,
  listen), swap the drill. Framed as the plan ("Let's take it to 100"),
  never as remediation. A teacher doesn't narrate your failure; they quietly
  make the next attempt winnable. Because it acts rather than speaks, the
  ladder **spends no part of the interruption allowance** — only the spoken
  name-the-wall moment does.
- **Name the wall — once.** At the quit-point, normalise plus a smaller
  step: "everyone's enclosures sound mechanical for the first three weeks;
  here's the smaller version." Never repeat the same encouragement twice;
  the second time is when it curdles into nagging.
- **Uncertainty asks instead of asserting.** Ambiguous attempt, low-fidelity
  input: "that sounded clean — agree?" One tap. Being asked is respectful;
  being wrongly failed is a trust-ender.

Feedback fades as mastery grows (the guidance-hypothesis tunable): a new
drill gets a verdict every rep (calibration); a solid drill shifts to
summary-every-few-reps; a mature drill goes predict-then-reveal — play,
self-rate with one tap, *then* see the machine's score. Predict-then-reveal
is the graduation arc in miniature: comparing self-judgement against
measurement is the skill a teacher hands over last, and it keeps mature
practice feeling like practising, not being invigilated. It is also the
**calibration instrument** for decision 17 — every predict-then-reveal rep is
a free measurement of how well this user's own verdict tracks the machine's, so
self-judgement stays exercised and evidenced from early on even though the
judgement track's model is deferred. What is postponed is the model, never the
practice.

All thresholds here — the fail-count trigger, the fade-out schedule, the
verdict cadence — are gate-criteria data, not code, calibrated against the
Phase 0 logs.

### Anti-abandonment

- First block is always a warm, low-stakes win (90%+ success rate). Momentum before difficulty.
- The comfort loop (playing what you know) is legitimised: it is the opener and closer, time-boxed. Frontier work sits in the middle, sandwiched by things that feel good.
- "End early" banks what was done and keeps the streak. A shortened session recorded beats an abandoned one; abandoning must never be what the app teaches.
- Off-piste mode: wandering is honoured, still listened to and logged, feeding diagnosis without a plan. End-of-wander prompt: "keep this as a drill?" Some wanders are the graph revealing a gap.
- Unmonitored play, one level below off-piste (decision 16): time logged, nothing scored, nothing inferred, no end-of-wander prompt. The distinction is consent — off-piste is "no plan, still listening"; this is "not today". A tool that can only be played while being measured is not a practice room, and on the bad day the difference decides whether the lid opens at all.

## Learning science (the optimisation levers)

A note on epistemic honesty: these mechanisms are directionally well-supported but the specific numbers are softer than app-copy usually admits. The design uses them as tunable setpoints, not laws.

1. **Interleaving over blocking — but not during acquisition.** Rotating drills in shorter blocks beats long single-drill blocks on retention, though it feels worse in session and effect sizes vary by task. The planner interleaves by default: the genuine edge, because self-directed learners never do it. The refinement (added 3 Aug 2026): blocked practice is better while material is being *acquired*, and interleaving wins for *retention*, so brand-new material earns a longer uninterrupted block and interleaving arrives as the node matures. Blocking-versus-interleaving is a function of node maturity, not a global default — which also means the four-minute block that suits a maintenance pass is the wrong shape for a first encounter.
2. **Spacing.** Expanding-interval review per node; reviews are quick gated checks, not full blocks. Keys, tunes, and drills all decay. SM-2 is a declarative-memory algorithm and motor-skill decay behaves differently, so intervals are a starting heuristic to be recalibrated against observed decay, not gospel.
3. **Desirable difficulty.** Target roughly 80-85% success per drill as the initial setpoint. The oft-cited 85% figure comes from perceptual and machine-learning tasks; its transfer to complex motor skill is plausible, not established. What is solid: with scored attempts the success rate is measurable, so tempo, key count, and constraints auto-adjust to hold whatever band proves right. The closed loop is the feature; the number is a parameter.
4. **Velocity, not just position.** Rate of improvement per node. Three flat sessions triggers a change (level down, different drill, attack a prerequisite). Also the honest progress view: "enclosures 90 to 140bpm this month." Note the blind spot velocity alone leaves: a node improving *slowly but steadily* never trips this trigger and can still be eating the fortnight, which is why the circling check watches time-share rather than only rate.
5. **Horizon planning.** Declared intent back-propagates through the graph: the goal into a weekly rhythm, a campaign into the next handful of sessions, both by back-chaining through prerequisites rather than by literal obedience. Plans re-derive weekly from current state *plus* current intent; a bad fortnight adjusts the forecast and may re-scope the campaign, never breaks the plan.
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
| Audio transcription | On-device, CoreML | A piano-specialist transcription model (ByteDance's open-source high-resolution model or Transkun; 95–98% note F1 on piano, vs ~71% for the originally-pencilled Basic Pitch — below the trust floor for per-note feedback) converts acoustic audio to note events, feeding the same Rust analysis. Used, not trained. Mic mode is a coarser scoring tier by construction (~±20ms onset precision at best; dense mid-register voicings are the weak case). |
| Coaching voice | Cloud LLM via Axum proxy | Claude/GPT. Input: metrics plus session, graph and intent state. Output: the teacher's voice. May also translate a stated goal into candidate graph targets — a suggestion the user edits, never a plan it executes (decision 12). Prompt engineering, cached, degrades gracefully offline. Never in the critical path. |
| Planning and prescription | On-device, Crux core (Rust) | Intent resolution (back-chaining from declared destinations), graph traversal, spacing, difficulty setpoint, session weave. Deliberately deterministic: testable, explainable, no hallucinated pedagogy. **Corrected from "backend Rust, state in Turso"** after the codebase review (`docs/rebuild-review.md` §5): the practice room is the worst place to need a server round-trip, and the device store is already the source of truth. |

Stack fit: Crux core holds the session state machine, MIDI event stream, analysis functions, and the planner. Swift shell handles CoreMIDI capture and AVAudioEngine plus the transcription model. Axum proxies the LLM only. The device store holds graph state, declared intent, and session history; Turso becomes sync and backup later, never the live read path. MIDI events are just events at the data-model level — but not at the wire level: note events cross the FFI bridge in batches, not individually (see the review's §3, which also corrects this doc's earlier "the event-sourced model extends naturally" — the core is command-pattern, not event-sourced).

Data model sketch: `Tune` (pipeline stage), `Skill` (graph node, mastery as estimate plus confidence), `Phrase` (per-key mastery, pipeline stage), `Device` (reusable musical device, referenced by many phrases, with theory annotation and its own mastery via the generate ladder), `Drill` = (skill x tune x parameters) referencing a `MethodPack` (decomposition stages, traversal orderings, cues by stage, escalation overrides), `Gate` criteria as tunable data (JSON/TOML), `Judgement` (per target: completion state, spaced-return interval, feel-rating series — never a level, never a prerequisite satisfier), `Goal` and `Campaign` (declared destinations — a campaign names graph nodes and/or a pipeline stage as its target, plus a horizon derived from the user's own trend), session planner as a pure function of (tune stages, skill mastery, pipeline states, declared intent, available minutes).

Mastery update function — **now specified in [`intrada-coach-engine.md`](intrada-coach-engine.md) §2**, which is the one answer; this paragraph is the summary. Per (node, parameter level), **measured** attempt outcomes update a Beta distribution (pass/fail as evidence), giving estimate and confidence for free; confidence decays with time-since-practice to drive spacing; level-ups shift the distribution rather than resetting it. The engine spec settles the three things this sentence left open: decay pulls the pseudo-counts back toward the prior (never toward failure), the review interval derives from that same state rather than a second schedule, and a level-up inherits a discounted share of the level below. Attempts-to-pass and time-share are recorded but feed the circling check only, never mastery.

Its inputs are now settled (decision 17): measured attempts only. Self-report does not enter this function at all. The **judgement track** sits beside it as a deliberately simpler structure — per target, a completion state (open / retired-by-you, with a spaced return interval) plus a time series of feel ratings for trend display. No distribution, no level, no contribution to prerequisite satisfaction. Keeping it structurally smaller is the point: a parallel mastery model would create two numbers competing to answer one question, and the planner would have to arbitrate between them on every block.

Custom model test: fine-tuning makes sense only when off-the-shelf fails the task. Frontier LLMs know jazz pedagogy; transcription models handle piano. The proprietary asset is structure (graph, criteria, method packs), injected as context. Revisit on-device coaching models at v3, not before.

Cost: a few LLM calls per session is pence per user per month.

## Design challenges and mitigations

1. **Attempt segmentation and time alignment (the hardest technical problem).** "Late into bar 3" requires a shared clock and knowing which notes constituted the attempt: count-in handling, restart detection, separating noodling from attempts, mid-attempt collapses. This, not MIDI capture, is the real Phase 1 risk, and it is why click-always is a recorded decision. Mitigation: a named spike before any scoring UI is built.
2. **Pedagogical authority without a teacher's judgement.** MIDI hears notes and milliseconds, not touch, feel, or phrasing, and for jazz the unmeasurable half matters at least as much. Risk: gating on the measurable trains mechanical playing, while the app hands the more important half back to the user who was the unreliable judge to begin with. Mitigation: the measured-prerequisites framing throughout; gates check the measurable, the voice structures self-assessment for the rest ("did that sound like the record? Rate the feel"). Self-report is a legitimate signal — with a deliberately asymmetric power (decision 17): it may close a target but never unlock a prerequisite, so the half the app cannot hear still gets practised, tracked and trended without the gates quietly becoming self-graded. What the framing does *not* fix is **proportion** (added 3 Aug 2026): the goal is confident improvising, the deploy gate that actually tests it is self-confirmed by decision 3, and everything rigorously scored is a prerequisite. An app optimises what it can see, so if nine-tenths of scored effort is mechanics, its gravity pulls there regardless of the stated destination. The time-by-circle tally is the cheap instrument that makes the imbalance visible, which is why it moves forward to Phase 2 rather than waiting for Phase 4 analytics.
3. **The content burden.** Nodes x drills x criteria x method packs x cues x micro-lessons is a textbook's worth of authored material, validated against one learner. "The moat" is also a euphemism for hundreds of authoring hours. Mitigation: the minimal-content decision (5 nodes, 2 packs, 1 phrase for v1); author breadth only after the loop is proven — and let the unmatched-target queue (decision 13) choose the authoring order, so content follows demonstrated demand instead of filling in the taxonomy. One known hole jumps that queue on merit: **rhythm has no node.** It appears only as a modifier inside other things — "rhythmic constraints", "rhythmic displacement" — which is a strange omission for a design whose foundational claim is that time is the foundation. Time and feel are measured thoroughly and taught nowhere. A subdivision / displacement / rhythmic-vocabulary family is first in the authoring queue, ahead of a fifth voicing variant.
4. **Done-criteria that don't lie.** Too strict: grind and quit. Too loose: mastery is fiction. Mitigation: criteria as tunable data, failure-to-pass rates instrumented from day one, permanent calibration expected.
5. **Prescription trust.** One arbitrary-feeling prescription and users revert to noodling. Mitigation: every block carries its why, citing the declared destination it serves and the node state behind it — a reason in the user's own words is the strongest form available. Explainability is a feature requirement, the argument for deterministic planning, and the reason the LLM is kept out of the route (decision 12).
6. **Rigidity vs the texture of practice.** Practice has moods, and a plan that cannot bend gets abandoned rather than adjusted. Mitigation, in ascending order of commitment: today's steer (one tap, re-weaves the session around your choice), re-scoping the campaign, off-piste (no plan, still logged), and the graduation arc (principle 8). Structure as default, never as cage — and never as a mode switch (decision 11).
7. **Cold start for user two.** Mitigation: placement session, aggressive early re-planning, surfaced humility ("early days, still learning your level"), uncertainty in the mastery model from the start. Parked behind the personal-tool-first decision.
8. **The audio measurement gap.** A wrong "wrong note" is instant trust damage. Mitigation: confidence-weighted feedback; when uncertain, ask ("that sounded clean, agree?"). Never bluff.
9. **Motivation mechanics that curdle.** Mitigation: design from failure stories; streaks count any engagement; soft landings; horizons framed from the user's own trend, never invented population statistics.
10. **The mundane one: MIDI setup UX.** Which cable, which piano, iOS quirks. For any user beyond user zero, this is where people actually fall off, before pedagogy ever matters. Mitigation: parked with the product decision, but not forgotten.

Pattern: almost none of these are engineering problems. They are judgement encoded as data: tolerances, criteria, phrasing, when to bend.

## Build plan

**Phase 0: paper teacher (1-2 weeks, no code).** Fully specify the minimal content set: 5 nodes, 2 method packs, 1 phrase (with stage and per-key state), Strasbourg's tune-pipeline position, gate criteria as data, and one declared goal plus one declared campaign (destination and horizon) so the fortnight tests intent resolution by hand — back-chaining a route from a named target is the planner's core move, and doing it on paper first is the cheapest possible test of whether the graph supports it. Sketch the rest of the improvisation branch as stubs only. Run own practice from it for a fortnight. Deliverables are the content set plus four logs:

- Divergence log: machine-score vs felt-score per drill, one line where they differ (specs the self-assessment boundary). Promoted in importance by decision 17: this log is the **evidence base for how much weight the judgement track eventually earns**. If felt-score tracks measured score closely over a fortnight, self-report is more trustworthy than assumed; if it drifts, and in which direction, that is a number rather than an opinion. Designing the judgement track before this log exists would be guessing.
- Gate-attempt log: **attempts-to-pass**, pass/fail, plus felt difficulty (first tolerance numbers, criteria schema shape). The attempts count is the highest-value number in the whole fortnight and v3 omitted it: it calibrates whether "3 clean passes" costs four tries or forty, and its distribution is what the circling check later compares against to spot a perfectionism loop.
- Why log: every block's one-line why citing node state and the campaign it serves (validates the graph and the back-chaining; a why that can't be written is a missing node, and a why that can't name the destination it serves is a missing campaign).
- Wander log: off-plan time, triggers, "keep as drill?" (sizes off-piste mode).

Plus: write the failure stories and desired app responses; note what the app should have done on the inevitable bad day; write the placement session as a thought experiment and check the criteria aren't jon-shaped. And one cheap, high-value test of the intent mechanism: take a real lesson's worth of targets, resolve it by hand into a week of blocks, and record which items matched a node, which stayed opaque (decision 13), and what the structural gap read produced. If that resolution is hard to do on paper, it will not be easier in Rust.

**Phase 1: the listening gate (3-4 weeks).** Opens with the named spike: attempt segmentation and click alignment (count-in, restart detection, noodling vs attempts, collapse handling) proven against real playing before any scoring UI exists. The same capture harness measures BLE-MIDI timestamp fidelity on the real piano (sender-side timestamps vs arrival times, USB as ground truth) — one afternoon that decides how much Bluetooth scoring can honestly claim (decision 7). Prior art to read before building segmentation: the Rach3 rehearsal-MIDI dataset tooling (ISMIR 2025). Then: CoreMIDI capture into the Crux core; deterministic analysis; one screen (drill, live scoring, gate progress, and the ambient elapsed/ceiling per decision 15). The session state machine records what the circling check will need from the start — reps after the gate opened, attempts-to-pass, time per node — because these are cheap to capture now and impossible to reconstruct later. First drill type: lick transposition (target exactly known, scoring unambiguous, exercises the Phrase model, user-zero motivation built in). Shells second, proving generalisation. Alongside: an honest hour surveying current competitors (Yousician-class products have quietly added more adaptivity than "content libraries" suggests). The moment to reach: play, see the tick, feel the gate release you.

**Phase 2 was split in two (3 Aug 2026)** after it accumulated twelve deliverables under a single three-to-four week estimate. A phase that cannot land is worse than a phase that admits its size, and 2a is independently useful.

**Phase 2a: prescribe and run (3-4 weeks).** The loop working end to end. Planner as a pure function of state plus declared intent; press-start flow; gated blocks with the visible ceiling; stuck button with the mechanical ladder; soft-landing exit; state on device (sync deferred). Prerequisite, not optional: the mastery update function specified first, with its inputs settled by decision 17 (measured attempts only). This is also the phase that deletes the notebook's session builder — the granular builder is not ported, because declaring a destination replaces it (see Intent, property 1). Exit criterion: a real session runs end to end, gates and all.

**Phase 2b: steer and guard (3-4 weeks).** Everything that makes the loop kind rather than merely correct. The three declaration surfaces (goal, campaign, today's steer) with their defaults, so the app is usable having declared nothing; multi-target campaigns with opaque targets accepted (decision 13); back-chaining with honest horizons and the structural gap read; **the circling check** and the grind trade; the off-piano queue with its own cue (not blocks inside a piano session); unmonitored play alongside off-piste; the time-by-circle tally (arithmetic over already-tagged blocks, pulled forward from Phase 4); the judgement track, now designed against a fortnight of divergence-log evidence rather than guessed (decision 17). Exit criterion: two weeks of daily use without reverting to self-directed sessions.

**Phase 3: the voice (2-3 weeks).** LLM behind Axum: summaries, whys, stuck-moment coaching (normalise plus smaller step), and goal interpretation — turning a sentence about how the user wants to sound into candidate graph targets they then edit (decision 12; the deterministic planner still owns the route). Deliberately late: coaching prose over unmeasured practice is a chatbot, not a teacher.

**Phase 4: widen (ongoing).** More drill types, spaced repetition, difficulty auto-adjustment, velocity tracking, the statistical gap read (decayed confidence and flat velocity under the declared targets) and the planner bias that acts on a starved circle once the Phase 2 tally has history behind it, ear-training interaction loop, audio path via a piano-specialist transcription model (preceded by two spikes: model-vs-MIDI ground truth from the same performance, yielding real error rates per drill type; and click-bleed — the metronome is a broadband transient landing exactly on the beats being scored, so headphone-click or click-subtraction must be proven before mic mode ships), placement diagnostics, second user (the graph needs a second data point before it is a curriculum).

**Sequencing calls (overrule as needed):** MIDI before audio; LLM last, not first; no placement until Phase 4.

**Effort:** evenings and weekends alongside COAST's September push. Phases 0-2 by late autumn, voice by year end. Phase 0 is also the actual practice regime: research time is piano time. Strasbourg is the test fixture.

## Competitive position

Content libraries with linear courses (Tonebase, Piano Marvel, iReal) are not mastery-based graph traversal (Duolingo, Math Academy). Nobody has done the latter properly for jazz piano. Yousician can hear notes; nobody has built the thing that gets you through minute eleven when you want to walk away. The moat is the pedagogy structure (graph, criteria, method packs), authored from inside the exact problem. One further gap worth naming: the graph-traversal products prescribe a fixed destination — you may progress through their tree but not aim it. Letting the learner declare a destination and having the system back-chain to it is a differentiator against both camps, and it costs almost nothing once the graph exists.

## Next step

The content set is authored (`content/`), so the remaining Phase 0 action is
the fortnight itself: declare the goal and the current campaign, correct the
seeded mastery values at the piano, and practise from the content for two weeks
while keeping the four logs. No code and no design are required to do that.
The doc is not "done" — v3 added the intent mechanism because using the design
in anger exposed the hole — but it is ahead of the practice, which is the right
order.
