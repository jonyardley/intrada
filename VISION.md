# Intrada

**Your Intentional Practice Companion**

*Product Vision & Research Foundation — February 2026*

---

## 1. Product Vision

### 1.1 The Problem

Musicians at every level share a common frustration: they practise, but they don't always progress. Hours pass at the instrument, yet weeks later the same passages stumble, the same keys feel unfamiliar, and the sense of forward motion stalls. The gap between effort and improvement is not a talent problem — it's a practice design problem.

Most practice is unstructured. Musicians repeat what feels comfortable, avoid what feels hard, and lack visibility into whether they're actually improving. Existing tools either track time (which measures attendance, not progress) or overwhelm with features that feel like work rather than music.

### 1.2 The Vision

**Intrada is an intentional practice companion that helps musicians practise smarter, see their progress, and trust the process.**

It sits at the intersection of three evidence-based principles: the science of learning (spaced repetition, interleaved practice, deliberate practice), the psychology of motivation (self-determination theory, growth mindset), and the practical realities of musical skill acquisition (technique across all keys, repertoire management, goal-directed study).

Intrada doesn't replace your teacher or your musicianship. It gives you the structure, visibility, and encouragement to make every practice session count.

### 1.3 Core Value Proposition

Where other apps track minutes or provide generic tools, Intrada offers something different:

- **Fine-grained metric-based progression** — track mastery at the level of individual keys, passages, and techniques, not just "time spent"
- **Smart scheduling** — algorithmically surface what you need to practise based on spaced repetition and interleaved practice research
- **Mindful and intentional practice** — encourage focus, reflection, and deliberate engagement rather than mindless repetition
- **Visible progress** — visualise improvement over time so musicians can see that the process works
- **Trust the process** — positive encouragement grounded in evidence that sustained, structured effort leads to real growth
- **Accessible by design** — designed with neurodivergent musicians in mind, reducing friction and supporting diverse cognitive styles

### 1.4 Target Users

Intrada is designed for self-directed musicians who are past the beginner stage and serious about improvement. This includes jazz and classical pianists, instrumental students at intermediate to advanced level, adult returners to music, and anyone preparing for graded exams, auditions, or performance. The common thread is a desire to practise more effectively, not just more.

An estimated 15–20% of the population is neurodivergent, and the intersection of ADHD with music is particularly significant — music provides the kind of stimulation, flow, and immediate feedback that ADHD brains thrive on (Wilde & Welch, 2022). Designing for neurodivergent musicians isn't niche — it's likely a meaningful portion of Intrada's audience, and features that support executive function challenges benefit every user.

---

## 2. Research Foundation

Intrada's design is grounded in well-established findings from cognitive psychology, motor learning, and music education research. This section summarises the key principles that inform every feature decision, including honest acknowledgement of where the evidence is strong and where it is more nuanced.

### 2.1 Spaced Repetition

The spacing effect is one of the most robust findings in learning science. Material reviewed at increasing intervals is retained far more effectively than material crammed in a single session. A meta-analysis by Donovan & Radosevich (1999) across 63 studies with 112 effect sizes found an overall mean weighted effect size of d=0.46, indicating that spaced practice conditions significantly outperform massed practice conditions. A more recent meta-analysis by Cepeda et al. (2006), reviewing 317 experiments across 184 articles, confirmed the spacing benefit and found that the optimal spacing interval increases as the desired retention interval increases.

**Important nuance for music:** Donovan & Radosevich also found that the spacing effect is moderated by task complexity — more complex tasks benefit less from distributed practice than simpler ones. Musical skill acquisition involves complex motor coordination, and Simmons (2012) found no spacing effect when teaching a 17-note piano sequence to novices, suggesting that the effect "may not always be demonstrable for complex motor skills." However, Moss (1995), reviewing 120 articles, found spacing improved learning of motor skills in over 80% of studies reviewed. The weight of evidence supports spacing for music, but Intrada's scheduling algorithm should be understood as leveraging a *general principle with strong support* rather than a precisely calibrated system proven specifically for instrument practice.

Intrada applies this by tracking when each item was last practised and scheduling reviews at increasing intervals based on the musician's self-rated mastery, while remaining tuneable as we learn more about how spacing interacts with musical skill retention.

### 2.2 Interleaved Practice

Carter & Grahn (2016) compared blocked and interleaved practice schedules with advanced clarinetists (n=10) in an ecologically valid practice environment. Whenever there was a ratings difference between conditions, pieces practised in the interleaved schedule were rated higher by expert assessors, though results varied across raters. Participant questionnaires revealed that interleaved practice had positive effects on goal setting, focus, and mistake identification. Notably, while 6 of 10 participants found the interleaved schedule more useful, the majority still preferred the blocked schedule — consistent with a well-documented phenomenon where blocked practice feels more productive despite producing inferior retention.

This "contextual interference effect" has broader support in motor skill learning research (Shea & Morgan, 1979; Magill & Hall, 1990), though results in music specifically are mixed. Stambaugh (2009) found interleaving benefits for beginner clarinetists, but Stambaugh & Demorest (2010) found no effect for technical accuracy in intermediate students. A more recent study by Mathias & Goldman (2025) with violinists explored increasing contextual interference within sessions, suggesting that a gradual shift from blocked to interleaved practice within a session may be optimal.

**Design implication:** Because musicians tend to resist interleaving despite its benefits, Intrada provides a user-facing interleaving intensity preference — from "gentle mixing" (similar items grouped loosely) through to "full interleave" (maximum variety between consecutive items). This respects autonomy (SDT) while nudging toward evidence-based scheduling. The Mathias & Goldman finding also informs session structure: sessions can begin with more similar items and increase variety as the session progresses, easing the transition for musicians unfamiliar with interleaved practice.

### 2.3 Deliberate Practice

Ericsson, Krampe & Tesch-Römer (1993) introduced the deliberate practice framework, proposing that expert performance is the result of prolonged, focused, effortful practice designed to improve specific aspects of performance. Their original study with violinists at a Berlin music academy found that the best violinists had accumulated significantly more solitary practice hours than less accomplished peers by age 20. This work has been enormously influential across domains.

However, the evidence base has been refined considerably since 1993. A meta-analysis by Macnamara, Hambrick & Oswald (2014) found that deliberate practice explained only about 21% of variance in music performance — substantial and practically meaningful, but far from the "sufficient account" Ericsson originally claimed. A direct replication by Macnamara & Maitra (2019), using improved double-blind methodology, found the effect was "substantial, but considerably smaller than the original study's effect size," explaining about 26% of performance variance. They also found that practice designed by the performers themselves was perceived as more relevant to improvement than teacher-designed practice.

What remains well-supported is the *qualitative* aspect of deliberate practice: that focused attention on specific goals, working at the edge of one's ability, and incorporating feedback leads to better outcomes than mindless repetition. Ambrose et al. (2010) synthesise this broader evidence, emphasising that a feedback-correction cycle — practising, evaluating, adjusting — is central to effective skill development.

Ericsson's original framework also emphasised that deliberate practice is effortful and requires recovery. This has implications for Intrada's design: the app should not only structure practice but also support appropriate rest, flagging when a musician's practice volume significantly exceeds their historical average or recommended thresholds.

Intrada supports deliberate practice by prompting musicians to set specific goals for each practice item, rate their performance honestly, and reflect on what improved and what didn't. It does not claim that practice quantity alone determines outcomes, but that practice *quality* matters enormously.

### 2.4 Self-Determination Theory (SDT)

Ryan & Deci's SDT framework identifies three basic psychological needs that sustain motivation: autonomy (feeling in control of one's actions), competence (feeling effective and capable), and relatedness (feeling connected to others). Evans & Bonneville-Roussy (2016) found that university music students' autonomous motivation predicted both the frequency of practice and the proportion of practice sessions reported as highly productive.

Crucially for Intrada's design, Valenzuela, Codina & Pestana (2018) investigated SDT in the context of conservatoire instrument practice (N=162) and found that perceived competence was the strongest predictor of flow variations during practice. Autonomous motivation had a direct positive effect on flow, while controlled motivation had an inverse effect. This has direct implications: visible progress and evidence of improvement feed the sense of competence that sustains motivation and flow states.

More recent work by Bonneville-Roussy & Evans (2024), studying 213 university music students across the UK and Canada, found that teacher autonomy support predicted autonomous motivation, which in turn predicted both practice time and practice quality. This reinforces the importance of autonomy-supportive design — Intrada should suggest and support, not dictate.

Intrada supports autonomy through user-driven goal setting and flexible practice design; competence through granular progress tracking and evidence of growth; and encouragement through positive, process-focused messaging.

### 2.5 Growth Mindset & Process Trust

Dweck's implicit theories framework (Dweck, 2006) distinguishes between a growth mindset (the belief that abilities can be developed through effort and learning) and a fixed mindset (the belief that abilities are innate and unchangeable). In educational contexts, Blackwell, Trzesniewski & Dweck (2007) found that students with a growth mindset outperformed fixed-mindset peers over time, even when controlling for initial achievement levels. Music is a domain where fixed-mindset beliefs about "talent" are particularly prevalent (O'Neill, 2002).

The growth mindset literature has faced legitimate scrutiny. A meta-analysis by Sisk et al. (2018) found that mindset interventions had only weak effects on academic achievement overall, though effects were larger for academically at-risk students. Dweck & Yeager (2019) responded by clarifying that mindset interventions are most effective when they are sustained and embedded in supportive contexts, rather than delivered as one-off treatments.

For Intrada, the relevant takeaway is not that a brief mindset intervention will transform practice, but that the *design of the tool itself* can reinforce growth-oriented beliefs. Progress visualisations provide objective evidence that effort leads to improvement. Process-focused encouragement tied to specific data points ("Your Db mastery went from 2 to 4 over three weeks") provides the kind of effort-linked feedback that Dweck's research associates with adaptive motivation.

**Design implication for consistency tracking:** Growth mindset research also informs how Intrada handles practice gaps. Rather than streak-based tracking that creates anxiety around breaking a streak (and can trigger shame-driven avoidance, particularly in musicians with ADHD), Intrada uses "comeback" framing that emphasises the return rather than the gap. "You've practised 4 of the last 7 days — that's great spacing for retention" rather than "5 day streak — don't break it!"

### 2.6 Retrieval Practice

An emerging area of relevance is retrieval practice — the finding that actively recalling information produces better long-term retention than passive review. While extensively studied in verbal learning, Wellmann & Skillicorn (2024) recently proposed the first systematic application of retrieval practice to jazz performance education, noting that its benefits should extend to music learning given that the underlying memory mechanisms are domain-general. They recommend spaced retrieval schedules with intervals of at least 24 hours between practice sessions of the same material for optimal retention.

This supports Intrada's scheduling approach: rather than letting musicians repeatedly drill the same material in a single session, the app encourages returning to material across sessions, leveraging both spacing and retrieval effects.

### 2.7 Neurodiversity & Music Practice

Research at the intersection of ADHD and music reveals both specific challenges and important strengths that inform Intrada's design.

**Executive function and task initiation.** ADHD involves deficits in task initiation, planning, organisation, and working memory (Barkley, 2015). Starting a practice session is often harder than sustaining one — the ADHD brain struggles with the transition from "thinking about practising" to "actually practising." Every decision point before playing begins is an opportunity for the ADHD brain to stall. This has direct implications for interaction design: the path from opening the app to playing should involve as few decisions as possible.

**Time blindness.** ADHD involves a disrupted sense of time passing (Ptacek et al., 2019). Musicians with ADHD may vastly underestimate or overestimate how long they've been practising, lose track of time during hyperfocused work on one item (neglecting others), or struggle to pace a practice session. This is distinct from poor time management — it's a perceptual deficit that external time cues can help mitigate.

**Hyperfocus paradox.** ADHD brains can enter hyperfocus states on engaging tasks, which for musicians might mean spending 45 minutes on one satisfying piece while neglecting the scale work that actually needs attention. The interleaved scheduling in Intrada is potentially therapeutic here — it externally structures what the ADHD brain struggles to self-regulate.

**Emotional dysregulation.** Frustration tolerance is often lower in ADHD. A practice session that repeatedly surfaces difficult material without any sense of progress can trigger emotional shutdown. Encouragement and visible progress features are not motivational niceties — for ADHD musicians, they may be the difference between sustained engagement and abandonment.

**Music-specific findings.** Research identifies timing deficits in ADHD, including difficulties with beat tracking and processing short time intervals (Puyjarinet et al., 2017; Serrallach et al., 2022). However, these deficits do not extend to improvisation and musical expression (Grob et al., 2022), and Wilde & Welch (2022) found that ADHD behaviours were often *absent* during active music-making. Raz (2025) found that musicians with ADHD showed enhanced cognitive abilities compared to non-musician ADHD peers, including better sustained attention and impulse control. This suggests that the practice session itself may be less affected than the surrounding executive function demands — deciding what to practise, starting, transitioning between items, and stopping.

**Broader neurodiversity.** Beyond ADHD, sensory processing differences (common in autism), dyslexia (affecting 10–15% of the population), and other cognitive variations all influence how musicians interact with a practice tool. Design principles that support neurodivergent users — reduced visual clutter, predictable navigation, configurable feedback, accessible typography — benefit every user.

---

## 3. Product Pillars

These six pillars organise Intrada's feature set. Each pillar maps directly to the research foundation and addresses a specific user need.

| Pillar | What It Does | Research Basis |
|--------|-------------|----------------|
| **Library & Organisation** | Manage pieces, exercises, and techniques. Group items by key, genre, or goal. Auto-generate key variations for exercises. | Reduces cognitive load so musicians focus on playing, not admin |
| **Smart Scheduling** | Algorithmically generate practice setlists using spaced repetition intervals and interleaved ordering. | Spacing effect (Donovan & Radosevich, 1999; Cepeda et al., 2006); Contextual interference (Carter & Grahn, 2016; Shea & Morgan, 1979) |
| **Intentional Practice** | Prompt goal-setting per item, encourage reflection after each session, support deliberate engagement. | Deliberate practice (Ericsson et al., 1993; Macnamara et al., 2014); Feedback-correction cycle (Ambrose et al., 2010) |
| **Metric-Based Progression** | Track mastery scores per key/passage/tempo. Log improvement over time with fine granularity. | Perceived competence predicts flow (Valenzuela et al., 2018); SDT competence need (Ryan & Deci, 2000) |
| **Progress Visualisation** | Charts, heatmaps, and dashboards showing improvement trends, consistency, and coverage gaps. | Growth mindset (Dweck, 2006; Blackwell et al., 2007); SDT competence; Process trust |
| **Encouragement & Goals** | Goal-setting framework. Positive, process-focused messaging. Celebration of consistency over perfection. | SDT autonomy and competence (Evans & Bonneville-Roussy, 2016; Bonneville-Roussy & Evans, 2024); Growth mindset |

---

## 4. Feature Detail

### 4.1 Library & Organisation

The library is the foundation of Intrada. Musicians add pieces, exercises, scales, licks, etudes, and any other practice material. Each item has metadata including instrument, genre/style tags, difficulty level, effort level, and tempo targets.

#### Key-Aware Grouping

A distinguishing feature of Intrada is key-aware exercise management. When a musician adds a jazz lick or chord progression exercise, they can specify that it should be practised in all 12 keys (or a subset). Intrada automatically generates 12 sub-items grouped under the parent exercise, each tracking mastery independently. This is essential for jazz musicians working on ii-V-I progressions, blues heads, or chord voicings, and for classical musicians working through technical exercises in all keys.

The musician sees the parent exercise as a single library item, but can drill into per-key progress. The smart scheduler treats each key as a separate scheduling unit, so weaker keys surface more frequently.

#### Item Types

- **Pieces** — repertoire with sections/passages as sub-items
- **Exercises** — technical studies, scales, arpeggios, patterns
- **Licks/Vocabulary** — jazz vocabulary items, typically practised across keys
- **Technique** — specific technical goals (e.g. left hand independence, pedalling)

#### Warm-Up Routines & Templates

Many musicians have consistent warm-up routines (long tones, scales in a particular pattern, Hanon exercises, etc.). Intrada supports reusable "routines" — a saved sequence of items that can be inserted as a block into any practice session. This reduces daily decision-making, supports consistency, and is particularly valuable for musicians who struggle with session initiation.

#### Searchable Practice Notes

All practice notes and reflections are full-text searchable across the entire history. A musician who can search "left hand" across all their practice notes and see every session where they noted left-hand difficulties gets a longitudinal view of a persistent challenge. This supports the self-awareness and metacognition that characterise effective practicers (Hallam, 2001).

### 4.2 Smart Practice Scheduling

The scheduler is Intrada's core intelligence. Given a musician's available practice time and their library, it generates an optimal practice setlist.

#### Scheduling Algorithm

The algorithm combines four factors. First, **spaced repetition urgency**: items whose review is overdue based on their mastery level and time since last practice are prioritised. The spacing intervals follow a modified SM-2 algorithm adapted for musical skill learning, with parameters that should be validated against user data over time (see Section 2.1 on the nuances of spacing for motor skills). Second, **interleaving**: the generated setlist alternates between different types of material (e.g. scale work, then a piece passage, then a lick in a new key) rather than grouping similar items together, with the degree of interleaving adjustable by the musician. Third, **goal alignment**: items tagged as relevant to the musician's current goals receive a priority boost. Fourth, **difficulty balancing**: the session balances effort across its duration, front-loading demanding work when focus is fresh and tapering toward lighter review material, avoiding sequences of multiple high-effort items back-to-back.

The Donovan & Radosevich (1999) finding that task complexity moderates the spacing effect informs the algorithm: items tagged as "high effort" may receive different scheduling parameters (shorter intervals) than "maintenance" items.

#### Session Structure

A generated practice session includes warm-up items (drawn from saved routines or algorithmically selected), focused work items (the core of the session), and review/consolidation items. Each item has a suggested time allocation. The musician can accept the suggested session, swap items, or adjust time allocations — maintaining autonomy while benefiting from intelligent scheduling. This flexibility is a deliberate design choice informed by SDT: the musician stays in control (Bonneville-Roussy & Evans, 2024).

#### One-Tap Session Start

The default interaction when opening Intrada is a ready-to-go session with a prominent "Start" button. No configuration required. The app has already decided what to practise based on the scheduling algorithm — the musician just has to begin. This directly addresses task initiation difficulty (Barkley, 2015) and reduces cognitive overhead for all users. The musician can always choose to modify the session, but the default path is zero-decision start.

#### Short Session Support

Intrada prominently offers short session options: 10, 15, 20, and 30 minutes alongside longer options. A 15-minute practice session using spaced repetition and interleaving is genuinely valuable — the spacing research shows that shorter, more frequent sessions often outperform longer, less frequent ones. Framing short sessions as legitimate and effective, rather than as "not enough," is both evidence-based and inclusive.

### 4.3 Intentional Practice Mode

When a musician starts practising an item, Intrada prompts them to set a micro-goal: what specifically will they focus on? This could be accuracy, evenness, dynamics, tempo, or any user-defined focus area. After the practice block, they rate their performance and note what improved and what needs further work.

This micro-cycle of goal → practice → reflect mirrors the deliberate practice feedback loop. It's lightweight enough to not feel burdensome, but structured enough to prevent mindless repetition. The qualitative value of focused, goal-directed practice is well-supported even where quantitative claims about practice hours have been moderated (Macnamara & Maitra, 2019).

**Note:** This micro-cycle is introduced early to establish the intentional practice habit from day one. Research on habit formation suggests that early interaction patterns tend to persist — if musicians learn to use Intrada without intentional practice prompts, adding them later may feel like friction rather than enhancement.

#### Focus Mode

During active practice, the screen shows only what's immediately relevant: the current item, the timer, and a simple rating control. No menu bars, no other library items visible, no notification badges. This "focus mode" minimises visual distraction, benefiting all users and particularly those with ADHD or sensory processing differences. The full UI with navigation, library, and dashboards is available but not competing for attention during the practice session itself.

#### Visible Timers with Progress Indication

A visible, always-present timer during practice shows both elapsed time per item and elapsed time for the session. The timer is visual rather than purely numerical — a progress ring or bar that fills is easier to perceive peripherally than digits. Gentle alerts at transition points (a soft chime or haptic when the suggested time for an item is reached) help with pacing without feeling punitive.

At the end of each session, a "time spent vs. planned" summary helps musicians calibrate their internal time sense over time — particularly valuable for musicians with ADHD-related time blindness (Ptacek et al., 2019).

#### Transition Prompts

Intrada provides explicit, gentle transition cues between practice items: a visual and/or haptic prompt when the suggested time has elapsed, a brief preview of what's coming next (reducing the cognitive load of "what do I do now?"), and an optional micro-break (even 30 seconds between items helps reset attention). Sessions begin with more similar items and increase variety as the session progresses, easing transitions (informed by Mathias & Goldman, 2025).

#### Repetition Counter

An optional tool available during practice on any item: a simple counter that tracks consecutive correct repetitions toward a target. The musician taps "got it" to increment or "missed" to decrement (never below zero). When the counter reaches the target, the item is marked as achieved for that session and the app prompts a transition to the next item.

The target is configurable per item (default 5, adjustable from 3 to 10). The decrement-on-error mechanic is the key design choice: it prevents musicians from counting sloppy repetitions toward their goal and ensures the final sequence of attempts represents genuine consistency rather than accumulated luck.

**Research basis:** The overlearning literature supports continuing correct repetitions beyond initial success to improve retention. Driskell, Willis & Copper (1992) reviewed 11 studies and found that overlearning (50–100% additional correct trials beyond the criterion of first success) has a positive influence on retention, with a moderate overall effect size (d = 0.753). Practically, this means if it takes a few attempts to get a passage right, doing 3–5 more correct repetitions is beneficial, with diminishing returns beyond that. Experienced music pedagogues converge on a similar range — enough to confirm consistency, not so many that habituation and careless errors set in (the "Practising the Piano" approach emphasises reflection between repetitions to prevent mindless drilling).

The 85% Rule (Wilson et al., 2019) found that learning is optimised at roughly 85% accuracy — about 1 error per 6 attempts. This has two implications for the repetition counter. First, it validates the decrement mechanic: errors during the process are expected and healthy, not failures. Second, it suggests the target shouldn't be too high — requiring 10 consecutive perfect repetitions may indicate the material is already too easy for maximal learning, or may frustrate musicians working at the productive edge of their ability. A default of 5 balances overlearning benefit against the 85% principle: a musician working at appropriate difficulty will typically experience a few decrements on the way to 5, resulting in a total attempt count where the error rate falls naturally in a productive range.

This feature is entirely optional — musicians can practise without it. But when used, it provides an objective "done" signal for an item, which is particularly valuable for musicians who either move on too quickly (a single success doesn't confirm learning) or stay too long (perfectionist drilling past the point of diminishing returns). For musicians with ADHD, the counter provides a concrete, visible goal that supports task focus and provides a clear endpoint for transitioning to the next item.

### 4.4 Metric-Based Progression

Intrada tracks multiple dimensions of mastery for each item:

- **Mastery score** — a 1–5 self-rating after each practice session, decaying over time if not practised
- **Tempo progression** — track comfortable tempo over time, working up to target
- **Per-key mastery** — for key-aware items, independent tracking per key
- **Consistency** — how regularly the item is practised
- **Notes & reflections** — qualitative log of what's working and what isn't, full-text searchable

### 4.5 Progress Visualisation

Visualisations serve two purposes: they provide actionable insight (where are the gaps?) and they reinforce the sense of competence that sustains motivation. Since perceived competence is the strongest predictor of flow during instrument practice (Valenzuela et al., 2018), making progress visible is not a cosmetic feature — it is motivationally central. Planned visualisations include:

- **Key coverage heatmap** — a circle-of-fifths view showing mastery level per key for any exercise, immediately revealing weak keys
- **Mastery timeline** — line charts showing how mastery scores have improved over weeks and months
- **Practice consistency calendar** — using "comeback" framing that emphasises frequency over streaks (e.g. "4 of the last 7 days" rather than "5 day streak"), celebrating returns after gaps rather than penalising them
- **Tempo progress chart** — showing tempo increase over time toward target BPM
- **Goal progress dashboard** — percentage completion toward active goals

**Basic encouragement messaging** (comparing current ratings to recent history) can be delivered before the full visualisation suite. Simple messages like "Your Db mastery went from 2 to 4 over the last three weeks" require only comparison of existing data, not charts or dashboards, and provide the competence feedback that sustains motivation early on.

### 4.6 Goals & Encouragement

Musicians set goals at multiple levels: session goals (what will I focus on today?), weekly goals (what do I want to accomplish this week?), and milestone goals (learn this piece by March, get all keys to mastery level 4). Intrada tracks progress toward each and provides process-focused encouragement.

The encouragement philosophy avoids empty praise — Dweck's own clarifications emphasise that process praise only works when tied to learning outcomes (Dweck & Yeager, 2019). Instead, Intrada highlights objective evidence: "Your Db mastery went from 2 to 4 over the last three weeks — that's the spacing effect at work." This ties progress back to the process, reinforcing trust that the method works.

**Customisable encouragement:** Encouragement messaging is configurable in frequency (every item, once per session, weekly summary only, off), tone (data-focused, warm, minimal), and delivery (inline during practice, end-of-session summary, notification). Some musicians find unsolicited encouragement patronising or distracting; others depend on it for motivation. This respects the reality that musicians — particularly neurodivergent musicians — are not a monolith.

### 4.7 Rest & Recovery Awareness

The deliberate practice literature emphasises that effortful practice requires recovery. Intrada tracks total practice time per day and week and gently flags when a musician is practising significantly more than their historical average or exceeding recommended thresholds. The messaging is encouraging: "You've been putting in serious work this week — your body and brain need recovery time to consolidate what you've learned." This protects against burnout and repetitive strain.

### 4.8 Practice Session Recording & Playback (Future)

A simple audio recording feature — record a run-through, play it back, then rate — supports the feedback-correction cycle central to deliberate practice. The recording doesn't need to be high-fidelity or stored long-term. Even a "record this attempt → listen back → delete or keep" flow is valuable.

This also addresses a limitation of self-rated mastery: ratings are subjective and can drift over time. Occasional recordings create an objective anchor. A musician might *feel* like their Bb major scale hasn't improved, but comparing a recording from three weeks ago with today's tells a different story — directly reinforcing perceived competence.

### 4.9 Teacher Integration (Future)

Macnamara & Maitra (2019) found that self-designed practice was perceived as more relevant than teacher-designed practice, but Bonneville-Roussy & Evans (2024) found that teacher autonomy support predicted better outcomes. These aren't contradictory: the ideal is a teacher who helps the musician design their own practice, not one who dictates it.

A teacher integration feature could allow a teacher to suggest items for the student's library, set target tempos or milestone goals, and view (with permission) progress dashboards. The musician retains full control over their daily practice — the teacher provides guidance at the goal and repertoire level. This positions Intrada as a bridge between lessons rather than a replacement for them.

### 4.10 AI Practice Assistant (Future)

A future AI layer could help musicians in several ways: suggesting practice setlists tailored to specific goals (e.g. "I have a gig in three weeks playing these standards"), identifying patterns in practice data ("You consistently struggle with keys that have four or more flats — here's a targeted exercise plan"), and helping navigate technique and musical understanding by explaining concepts in context.

An additional AI feature is post-session review: after a session, the AI analyses the pattern of ratings, time allocations, and reflections and offers a brief synthesis — "You spent 40% of your time on pieces and only 10% on the keys you rated lowest. Want me to rebalance tomorrow's session?" This connects the AI to the scheduling engine in a way that's immediately actionable.

The AI layer could also adapt interleaving intensity based on user response patterns, automatically adjusting mixing aggressiveness based on how the musician responds to different levels of contextual interference over time.

This feature builds on the existing library and data layer — the AI has access to the musician's practice history, mastery data, and goals, making its suggestions specific and actionable rather than generic.

---

## 5. Competitive Landscape

Understanding existing solutions helps position Intrada's unique contribution.

| App | Strengths | Weaknesses | Intrada Differentiator |
|-----|-----------|------------|----------------------|
| **Modacity** | Deliberate practice guidance, recording, mastery rating, good UX | No spaced repetition scheduling, no per-key tracking, expensive subscription, basic statistics | Algorithmic scheduling, key-aware tracking, fine-grained metrics, research-backed process |
| **Tonic** | Social/community features, gamification, practice streaks | Social-first rather than practice-quality-first, no deep progress metrics, no scheduling intelligence | Practice quality over social metrics, deep per-item analytics, privacy-first |
| **Piano Practice Assistant** | True spaced repetition, section-level tracking, interleaved practice, research-grounded | Android-only, piano-only, minimal UX polish, no goal framework | Cross-platform, instrument-agnostic, polished UX, goal-setting, encouragement layer |
| **Instrumentive** | Practice habit tracking, recording, metronome, similar to Modacity | Focuses on habit building over practice quality, limited analytics | Both habit and quality, research-backed scheduling, richer data model |

**Intrada's unique position**: the only app that combines spaced repetition scheduling, interleaved practice generation, key-aware exercise management, fine-grained metric tracking, neurodiversity-informed design, and a research-backed encouragement framework — all in a clean, instrument-agnostic package.

---

## 6. Product Roadmap

The detailed roadmap lives in [`docs/roadmap.md`](docs/roadmap.md) and is the single
source of truth for what's built, what's next, and what's planned.

The roadmap is organised around three activity pillars — **Plan** (decide what to
practise), **Practice** (play with intention), and **Track** (see the process working).
Each pillar advances independently; a musician benefits from progress in any of them
without waiting for the others. Within each pillar, features are placed on a rolling
horizon: **Now** (next 4 weeks), **Next** (4–12 weeks), and **Later** (12+ weeks).

See the roadmap for current status, feature list, and prioritisation model.

---

## 7. Inclusive Design Principles

Designing for neurodivergent musicians isn't a separate workstream — it's embedded in Intrada's core design. These principles apply across all phases and benefit every user.

### 7.1 Reduce Decisions to Start

Every decision point between "I want to practise" and "I am practising" is an opportunity for any musician to stall — and for a musician with executive function challenges, it's a potential session-ending barrier. The default path through Intrada requires one decision: tap "Start." Everything else is optional.

### 7.2 Externalise Time

Time blindness affects ADHD musicians (Ptacek et al., 2019), but all musicians benefit from external time structure. Visual progress indicators, gentle transition prompts, and post-session time summaries make time concrete and manageable rather than abstract and anxiety-inducing.

### 7.3 Celebrate Comeback, Not Streak

Consistency matters, but the framing of consistency determines whether it motivates or shames. Intrada never shows a broken streak counter or a zero. It celebrates returns: "Welcome back — your scheduling has adapted and we've got a session ready for you." This aligns with growth mindset research (Dweck & Yeager, 2019) and protects against the emotional dysregulation that can turn a missed day into an abandoned practice habit.

### 7.4 Sensory Sensitivity

Avoid auto-playing sounds, sudden animations, or bright flashing elements. Offer colour themes including high-contrast and low-contrast modes, with a calm, muted palette as the default. All haptic feedback and sound cues are optional and configurable. Music notation rendering (if added) avoids overly dense visual presentation, allowing the musician to control how much is visible at once.

### 7.5 Accessible Typography

Use a clean sans-serif font as the default. Ensure sufficient contrast between text and background — dark grey on off-white rather than pure black on pure white to reduce visual stress. Adequate line spacing and letter spacing support readability for the estimated 10–15% of the population with some degree of dyslexia, and benefit every user.

### 7.6 Predictable Navigation

Navigation elements are always in the same place. The flow through a practice session follows the same pattern every time. Core workflows remain stable across versions. Consistency is a form of accessibility — particularly for autistic users who benefit from predictable interface behaviour, but for all users who build muscle memory around their tools.

### 7.7 Configurable Feedback

Not all musicians want the same encouragement. Messaging is configurable in frequency (every item, once per session, weekly summary only, off), tone (data-focused, warm, minimal), and delivery (inline during practice, end-of-session summary, notification). What motivates one musician may irritate another — particularly across neurodivergent profiles.

---

## 8. General Design Principles

These principles guide every feature and design decision in Intrada.

**1. Simplicity over features.** Every interaction should feel lightweight. If logging a practice session takes more than 30 seconds, it's too slow. Musicians will abandon tools that feel like work.

**2. Progress is the product.** The primary value Intrada delivers is the feeling of making progress. Every feature should either enable progress or make it visible. This is grounded in the finding that perceived competence is the strongest predictor of flow during practice (Valenzuela et al., 2018).

**3. Trust the musician.** Intrada provides structure and intelligence, but the musician stays in control. Suggested sessions can always be modified. Self-ratings are trusted. Autonomy is preserved — consistent with SDT research showing that autonomy support predicts better practice outcomes (Bonneville-Roussy & Evans, 2024).

**4. Evidence over opinion.** Feature decisions are grounded in learning science research. Encouragement is grounded in observable data, not empty platitudes. Where the evidence is nuanced (see Section 2), we acknowledge the limitations honestly.

**5. Music first, tech second.** The app should feel like a musical tool, not a productivity app. Terminology, aesthetics, and workflows should reflect how musicians think about practice.

**6. Offline-first.** Musicians practise wherever their instrument is, not necessarily where there's internet. The app must work fully offline and sync when connected.

**7. Designing for the edges improves the middle.** Features that support neurodivergent musicians — reduced friction, external time structure, configurable feedback, sensory consideration — make the experience better for everyone. Accessibility is not a separate feature; it's a design philosophy embedded from the foundation.

---

## 9. Technical Considerations

### 9.1 Current Architecture

The existing Intrada codebase uses a Crux pure-core pattern in Rust with a Leptos/WASM web frontend, an Axum API server, and Turso (libsql/SQLite) for persistence. This architecture is solid for a cross-platform approach, with the pure core being portable to iOS, Android, and desktop shells.

### 9.2 Platform Decision: iOS-First vs Web-First

Given the rewrite context, a key decision is whether to target iOS native (Swift/SwiftUI) or continue with the Rust/WASM/Crux approach. Considerations include:

- **iOS-first (SwiftUI):** Best UX for the primary use case (practising at an instrument with a phone/tablet nearby). Native haptics, notifications, and offline-first with SwiftData/Core Data. Faster iteration for a solo developer.
- **Crux/Rust (current):** True cross-platform potential, but higher complexity. Web shell already exists. The pure-core pattern is architecturally elegant but adds development overhead.

Recommendation: for Phase 1–3, optimise for speed and UX quality. If iOS is the primary target, SwiftUI with a clean data layer will get to a usable product faster. The Crux core can be retained for the business logic if cross-platform is a near-term goal.

### 9.3 Data Model

The core data model needs to support the fine-grained tracking that differentiates Intrada. Key entities include: Library Items (with type, metadata, key-awareness flag, target tempo, effort level), Sub-Items (per-key instances, sections/passages), Practice Sessions (timestamp, duration, items practised), Practice Logs (per-item within a session: mastery rating, tempo achieved, notes, focus area, goal, reflection), Routines (saved sequences of items for warm-ups or recurring patterns), Goals (type, target, deadline, linked items), and Scheduling State (per sub-item: last practised, interval, ease factor).

### 9.4 Scheduling Algorithm

The spaced repetition engine should be adapted from SM-2 but with modifications for musical skill learning. As discussed in Section 2.1, musical skills involve complex motor coordination with different retention characteristics to verbal learning. The algorithm parameters should be tuneable and should be validated against user data over time. Initial parameters should err on the side of shorter intervals than typical flashcard systems, with the expectation that they will be refined as real usage data accumulates. Items with higher effort tags may receive different scheduling parameters (shorter intervals, less aggressive interval growth) than maintenance items.

---

## 10. References

Ambrose, S. A., Bridges, M. W., DiPietro, M., Lovett, M. C., & Norman, M. K. (2010). *How Learning Works: Seven Research-Based Principles for Smart Teaching*. San Francisco: Jossey-Bass. ISBN: 978-0-470-48410-4.

Barkley, R. A. (2015). *Attention-Deficit Hyperactivity Disorder: A Handbook for Diagnosis and Treatment* (4th ed.). New York: Guilford Press.

Blackwell, L. S., Trzesniewski, K. H., & Dweck, C. S. (2007). Implicit theories of intelligence predict achievement across an adolescent transition: A longitudinal study and an intervention. *Child Development*, 78(1), 246–263. https://doi.org/10.1111/j.1467-8624.2007.00995.x

Bonneville-Roussy, A., & Evans, P. (2024). The support of autonomy, motivation, and music practice in university music students: A self-determination theory perspective. *Psychology of Music*. https://doi.org/10.1177/03057356241296109

Carter, C. E., & Grahn, J. A. (2016). Optimizing music learning: Exploring how blocked and interleaved practice schedules affect advanced performance. *Frontiers in Psychology*, 7, 1251. https://doi.org/10.3389/fpsyg.2016.01251

Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006). Distributed practice in verbal recall tasks: A review and quantitative synthesis. *Psychological Bulletin*, 132(3), 354–380. https://doi.org/10.1037/0033-2909.132.3.354

Donovan, J. J., & Radosevich, D. J. (1999). A meta-analytic review of the distribution of practice effect: Now you see it, now you don't. *Journal of Applied Psychology*, 84(5), 795–805. https://doi.org/10.1037/0021-9010.84.5.795

Driskell, J. E., Willis, R. P., & Copper, C. (1992). Effect of overlearning on retention. *Journal of Applied Psychology*, 77(5), 615–622. https://doi.org/10.1037/0021-9010.77.5.615

Dweck, C. S. (2006). *Mindset: The New Psychology of Success*. New York: Random House.

Dweck, C. S., & Yeager, D. S. (2019). Mindsets: A view from two eras. *Perspectives on Psychological Science*, 14(3), 481–496. https://doi.org/10.1177/1745691618804166

Ericsson, K. A., Krampe, R. T., & Tesch-Römer, C. (1993). The role of deliberate practice in the acquisition of expert performance. *Psychological Review*, 100(3), 363–406. https://doi.org/10.1037/0033-295X.100.3.363

Evans, P., & Bonneville-Roussy, A. (2016). Self-determined motivation for practice in university music students. *Psychology of Music*, 44(5), 1095–1110. https://doi.org/10.1177/0305735615610926

Grob, C. M., Biasutti, M., & Schacter, E. N. (2022). Musical improvisation and expression in individuals with ADHD. *Frontiers in Psychology*, 13, 895780.

Hallam, S. (2001). The development of metacognition in musicians: Implications for education. *British Journal of Music Education*, 18(1), 27–39.

Macnamara, B. N., Hambrick, D. Z., & Oswald, F. L. (2014). Deliberate practice and performance in music, games, sports, education, and professions: A meta-analysis. *Psychological Science*, 25(8), 1608–1618. https://doi.org/10.1177/0956797614535810

Macnamara, B. N., & Maitra, M. (2019). The role of deliberate practice in expert performance: Revisiting Ericsson, Krampe & Tesch-Römer (1993). *Royal Society Open Science*, 6(8), 190327. https://doi.org/10.1098/rsos.190327

Mathias, T., & Goldman, A. (2025). How does increasing contextual interference in a musical practice session affect acquisition and retention? *Journal of Research in Music Education*. https://doi.org/10.1177/00224294231222801

Moss, S. L. (1995). The distribution of practice effect: A review of the literature. Unpublished review, cited in Firth et al. (2023).

O'Neill, S. A. (2002). The self-identity of young musicians. In R. A. R. MacDonald, D. J. Hargreaves, & D. Miell (Eds.), *Musical Identities* (pp. 79–96). Oxford University Press.

Ptacek, R., Weissenberger, S., Braaten, E., Klicperova-Baker, M., Goetz, M., Raboch, J., & Stefano, G. B. (2019). Clinical implications of the perception of time in attention deficit hyperactivity disorder (ADHD): A review. *Medical Science Monitor*, 25, 3918–3924.

Puyjarinet, F., Bégel, V., Lopez, R., Dellacherie, D., & Dalla Bella, S. (2017). Children and adults with Attention-Deficit/Hyperactivity Disorder cannot move to the beat. *Scientific Reports*, 7, 11550.

Raz, S. (2025). Enhancing cognitive abilities in young adults with ADHD through instrumental music training. *Psychological Research*, 89, 9. https://doi.org/10.1007/s00426-024-02048-2

Ryan, R. M., & Deci, E. L. (2000). Self-determination theory and the facilitation of intrinsic motivation, social development, and well-being. *American Psychologist*, 55(1), 68–78. https://doi.org/10.1037/0003-066X.55.1.68

Serrallach, B., Groß, C., Christiner, M., Wildermuth, S., & Schneider, P. (2022). Musical performance in adolescents with ADHD, ADD and dyslexia — Behavioral and neurophysiological aspects. *Brain Sciences*, 12(2), 127.

Shea, J. B., & Morgan, R. L. (1979). Contextual interference effects on the acquisition, retention, and transfer of a motor skill. *Journal of Experimental Psychology: Human Learning and Memory*, 5(2), 179–187. https://doi.org/10.1037/0278-7393.5.2.179

Simmons, A. L. (2012). Distributed practice and procedural memory consolidation in musicians' skill learning. *Journal of Research in Music Education*, 59(4), 357–368. https://doi.org/10.1177/0022429411424798

Sisk, V. F., Burgoyne, A. P., Sun, J., Butler, J. L., & Macnamara, B. N. (2018). To what extent and under what circumstances are growth mind-sets important to academic achievement? Two meta-analyses. *Psychological Science*, 29(4), 549–571. https://doi.org/10.1177/0956797617739704

Stambaugh, L. A. (2009). Effects of practice schedule on wind instrument performance: A preliminary application of a motor learning principle. *Update: Applications of Research in Music Education*, 27(2), 20–28.

Stambaugh, L. A., & Demorest, S. M. (2010). Effects of practice schedule on the acquisition and retention of wind instrument skills. *Journal of Research in Music Education*, 58(4), 357–367.

Valenzuela, R., Codina, N., & Pestana, J. V. (2018). Self-determination theory applied to flow in conservatoire music practice: The roles of perceived autonomy and competence, and autonomous and controlled motivation. *Psychology of Music*, 46(1), 33–48. https://doi.org/10.1177/0305735617694502

Wellmann, M., & Skillicorn, A. T. (2024). Research-to-resource: Introducing retrieval practice in jazz pedagogy. *Journal of Research in Music Education*. https://doi.org/10.1177/87551233221146282

Wilde, E. M., & Welch, G. F. (2022). Attention deficit hyperactivity disorder (ADHD) and musical behaviour: The significance of context. *Psychology of Music*, 50(6), 1903–1920.

Wilson, R. C., Shenhav, A., Straccia, M., & Cohen, J. D. (2019). The Eighty Five Percent Rule for optimal learning. *Nature Communications*, 10, 4646. https://doi.org/10.1038/s41467-019-12552-4

---

*Intrada: Practise with intention. See the progress. Trust the process.*
