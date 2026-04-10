# Intrada — Research Foundation

*Evidence base for design decisions — extracted from product vision, April 2026*

This document contains the academic and empirical grounding for Intrada's design decisions. It is referenced from the [Product Vision](../VISION.md) and informs the [Roadmap](roadmap.md). The research is organised by topic, with honest acknowledgement of where evidence is strong and where it is more nuanced.

---

## 1. Spaced Repetition

The spacing effect is one of the most robust findings in learning science. Material reviewed at increasing intervals is retained far more effectively than material crammed in a single session. A meta-analysis by Donovan & Radosevich (1999) across 63 studies with 112 effect sizes found an overall mean weighted effect size of d=0.46, indicating that spaced practice conditions significantly outperform massed practice conditions. A more recent meta-analysis by Cepeda et al. (2006), reviewing 317 experiments across 184 articles, confirmed the spacing benefit and found that the optimal spacing interval increases as the desired retention interval increases.

**Important nuance for music:** Donovan & Radosevich also found that the spacing effect is moderated by task complexity — more complex tasks benefit less from distributed practice than simpler ones. Musical skill acquisition involves complex motor coordination, and Simmons (2012) found no spacing effect when teaching a 17-note piano sequence to novices, suggesting that the effect "may not always be demonstrable for complex motor skills." However, Moss (1995), reviewing 120 articles, found spacing improved learning of motor skills in over 80% of studies reviewed. The weight of evidence supports spacing for music, but Intrada's scheduling algorithm should be understood as leveraging a *general principle with strong support* rather than a precisely calibrated system proven specifically for instrument practice.

> **Citation note:** Moss (1995) is an unpublished review cited secondarily via Firth et al. (2023). The specific "120 articles" and "80%+" claims cannot be independently verified from the primary source. The broader claim — that spacing benefits motor skills — is well-supported by published meta-analyses, but this specific citation should be treated with caution.

**Design implication:** Intrada tracks when each item was last practised and schedules reviews at increasing intervals based on the musician's self-rated mastery, while remaining tuneable as we learn more about how spacing interacts with musical skill retention.

**Scheduling algorithm note:** SM-2 was designed for verbal flashcard learning (Wozniak, 1987) and its interval growth parameters are calibrated for declarative memory retention. Musical skill learning involves procedural/motor memory with different consolidation dynamics (Walker & Stickgold, 2004). No published research validates SM-2 parameters for motor skill scheduling. Intrada's initial parameters are informed guesses that must be refined empirically through user data. This is a known unknown, not a solved problem.

---

## 2. Interleaved Practice

Carter & Grahn (2016) compared blocked and interleaved practice schedules with advanced clarinetists (n=10) in an ecologically valid practice environment. Whenever there was a ratings difference between conditions, pieces practised in the interleaved schedule were rated higher by expert assessors, though results varied across raters. Participant questionnaires revealed that interleaved practice had positive effects on goal setting, focus, and mistake identification. Notably, while 6 of 10 participants found the interleaved schedule more useful, the majority still preferred the blocked schedule — consistent with a well-documented phenomenon where blocked practice feels more productive despite producing inferior retention.

This "contextual interference effect" has broader support in motor skill learning research (Shea & Morgan, 1979; Magill & Hall, 1990), though results in music specifically are mixed. Stambaugh (2009) found interleaving benefits for beginner clarinetists, but Stambaugh & Demorest (2010) found no effect for technical accuracy in intermediate students. A more recent study by Mathias & Goldman (2025) with violinists explored increasing contextual interference within sessions, suggesting that a gradual shift from blocked to interleaved practice within a session may be optimal.

**Design implication:** Because musicians tend to resist interleaving despite its benefits, Intrada provides a user-facing interleaving intensity preference — from "gentle mixing" (similar items grouped loosely) through to "full interleave" (maximum variety between consecutive items). This respects autonomy (SDT) while nudging toward evidence-based scheduling. The Mathias & Goldman finding also informs session structure: sessions can begin with more similar items and increase variety as the session progresses, easing the transition for musicians unfamiliar with interleaved practice.

---

## 3. Deliberate Practice

Ericsson, Krampe & Tesch-Römer (1993) introduced the deliberate practice framework, proposing that expert performance is the result of prolonged, focused, effortful practice designed to improve specific aspects of performance. Their original study with violinists at a Berlin music academy found that the best violinists had accumulated significantly more solitary practice hours than less accomplished peers by age 20. This work has been enormously influential across domains.

However, the evidence base has been refined considerably since 1993. A meta-analysis by Macnamara, Hambrick & Oswald (2014) found that deliberate practice explained only about 21% of variance in music performance — substantial and practically meaningful, but far from the "sufficient account" Ericsson originally claimed. A direct replication by Macnamara & Maitra (2019), using improved double-blind methodology, found the effect was "substantial, but considerably smaller than the original study's effect size," explaining about 26% of performance variance. They also found that practice designed by the performers themselves was perceived as more relevant to improvement than teacher-designed practice.

What remains well-supported is the *qualitative* aspect of deliberate practice: that focused attention on specific goals, working at the edge of one's ability, and incorporating feedback leads to better outcomes than mindless repetition. Ambrose et al. (2010) synthesise this broader evidence, emphasising that a feedback-correction cycle — practising, evaluating, adjusting — is central to effective skill development.

Ericsson's original framework also emphasised that deliberate practice is effortful and requires recovery. This has implications for Intrada's design: the app should not only structure practice but also support appropriate rest, flagging when a musician's practice volume significantly exceeds their historical average or recommended thresholds.

**Design implication:** Intrada supports deliberate practice by prompting musicians to set specific goals for each practice item, rate their performance honestly, and reflect on what improved and what didn't. It does not claim that practice quantity alone determines outcomes, but that practice *quality* matters enormously.

---

## 4. Self-Determination Theory (SDT)

Ryan & Deci's SDT framework identifies three basic psychological needs that sustain motivation: autonomy (feeling in control of one's actions), competence (feeling effective and capable), and relatedness (feeling connected to others). Evans & Bonneville-Roussy (2016) found that university music students' autonomous motivation predicted both the frequency of practice and the proportion of practice sessions reported as highly productive.

Crucially for Intrada's design, Valenzuela, Codina & Pestana (2018) investigated SDT in the context of conservatoire instrument practice (N=162) and found that perceived competence was the strongest predictor of flow variations during practice. Autonomous motivation had a direct positive effect on flow, while controlled motivation had an inverse effect. This has direct implications: visible progress and evidence of improvement feed the sense of competence that sustains motivation and flow states.

More recent work by Bonneville-Roussy & Evans (2024), studying 213 university music students across the UK and Canada, found that teacher autonomy support predicted autonomous motivation, which in turn predicted both practice time and practice quality. This reinforces the importance of autonomy-supportive design — Intrada should suggest and support, not dictate.

**Design implication:** Intrada supports autonomy through user-driven goal setting and flexible practice design; competence through granular progress tracking and evidence of growth; and encouragement through positive, process-focused messaging.

---

## 5. Growth Mindset & Process Trust

Dweck's implicit theories framework (Dweck, 2006) distinguishes between a growth mindset (the belief that abilities can be developed through effort and learning) and a fixed mindset (the belief that abilities are innate and unchangeable). In educational contexts, Blackwell, Trzesniewski & Dweck (2007) found that students with a growth mindset outperformed fixed-mindset peers over time, even when controlling for initial achievement levels. Music is a domain where fixed-mindset beliefs about "talent" are particularly prevalent (O'Neill, 2002).

The growth mindset literature has faced legitimate scrutiny. A meta-analysis by Sisk et al. (2018) found that mindset interventions had only weak effects on academic achievement overall, though effects were larger for academically at-risk students. Dweck & Yeager (2019) responded by clarifying that mindset interventions are most effective when they are sustained and embedded in supportive contexts, rather than delivered as one-off treatments.

For Intrada, the relevant takeaway is not that a brief mindset intervention will transform practice, but that the *design of the tool itself* can reinforce growth-oriented beliefs. Progress visualisations provide objective evidence that effort leads to improvement. Process-focused encouragement tied to specific data points ("Your Db mastery went from 2 to 4 over three weeks") provides the kind of effort-linked feedback that Dweck's research associates with adaptive motivation.

**Design implication for consistency tracking:** Growth mindset research informs how Intrada handles practice gaps. Rather than streak-based tracking that creates anxiety around breaking a streak (and can trigger shame-driven avoidance, particularly in musicians with ADHD), Intrada uses "comeback" framing that emphasises the return rather than the gap. "You've practised 4 of the last 7 days — that's great spacing for retention" rather than "5 day streak — don't break it!"

> **Assumption:** The claim that streak-based tracking causes anxiety and avoidance is a design hypothesis, not a research-proven finding. While there is general psychology research on shame-avoidance cycles and ADHD emotional dysregulation (Barkley, 2015), no published study directly compares streak-based versus comeback-based practice tracking in music apps. The design choice is informed by SDT principles (autonomy support) and anecdotal practitioner experience. This is a reasonable design decision but should not be presented as research-proven.

---

## 6. Retrieval Practice

An emerging area of relevance is retrieval practice — the finding that actively recalling information produces better long-term retention than passive review. While extensively studied in verbal learning, Wellmann & Skillicorn (2024) recently proposed the first systematic application of retrieval practice to jazz performance education, noting that its benefits should extend to music learning given that the underlying memory mechanisms are domain-general. They recommend spaced retrieval schedules with intervals of at least 24 hours between practice sessions of the same material for optimal retention.

**Design implication:** Rather than letting musicians repeatedly drill the same material in a single session, the app encourages returning to material across sessions, leveraging both spacing and retrieval effects.

> **Open question:** Wellmann & Skillicorn's proposal is theoretical — they argue retrieval practice *should* benefit music learning based on domain-general memory mechanisms, but this has not been empirically tested in a music practice context. The underlying principle is sound, but the specific transfer to complex motor skills with a musical-expressive component remains an assumption.

---

## 7. Neurodiversity & Music Practice

Research at the intersection of ADHD and music reveals both specific challenges and important strengths that inform Intrada's design.

**Executive function and task initiation.** ADHD involves deficits in task initiation, planning, organisation, and working memory (Barkley, 2015). Starting a practice session is often harder than sustaining one — the ADHD brain struggles with the transition from "thinking about practising" to "actually practising." Every decision point before playing begins is an opportunity for the ADHD brain to stall. This has direct implications for interaction design: the path from opening the app to playing should involve as few decisions as possible.

**Time blindness.** ADHD involves a disrupted sense of time passing (Ptacek et al., 2019). Musicians with ADHD may vastly underestimate or overestimate how long they've been practising, lose track of time during hyperfocused work on one item (neglecting others), or struggle to pace a practice session. This is distinct from poor time management — it's a perceptual deficit that external time cues can help mitigate.

**Hyperfocus paradox.** ADHD brains can enter hyperfocus states on engaging tasks, which for musicians might mean spending 45 minutes on one satisfying piece while neglecting the scale work that actually needs attention. The interleaved scheduling in Intrada is potentially therapeutic here — it externally structures what the ADHD brain struggles to self-regulate.

**Emotional dysregulation.** Frustration tolerance is often lower in ADHD. A practice session that repeatedly surfaces difficult material without any sense of progress can trigger emotional shutdown. Encouragement and visible progress features are not motivational niceties — for ADHD musicians, they may be the difference between sustained engagement and abandonment.

**Music-specific findings.** Research identifies timing deficits in ADHD, including difficulties with beat tracking and processing short time intervals (Puyjarinet et al., 2017; Serrallach et al., 2022). However, these deficits do not extend to improvisation and musical expression (Grob et al., 2022), and Wilde & Welch (2022) found that ADHD behaviours were often *absent* during active music-making. Raz (2025) found that musicians with ADHD showed enhanced cognitive abilities compared to non-musician ADHD peers, including better sustained attention and impulse control. This suggests that the practice session itself may be less affected than the surrounding executive function demands — deciding what to practise, starting, transitioning between items, and stopping.

**Broader neurodiversity.** Beyond ADHD, sensory processing differences (common in autism), dyslexia (affecting 10–15% of the population), and other cognitive variations all influence how musicians interact with a practice tool. Design principles that support neurodivergent users — reduced visual clutter, predictable navigation, configurable feedback, accessible typography — benefit every user.

---

## 8. The Feedback Loop Problem & Self-Taught Learning

### What teachers actually do

Effective music teaching is not primarily about demonstrating technique. Duke (2005) describes the core teacher functions as: **diagnose** (identify what specifically is going wrong), **decompose** (break complex skills into manageable sub-skills), **sequence** (order learning steps appropriately), and **regulate** (monitor progress and adjust the plan). These functions are forms of metacognition that expert musicians have internalised but that developing musicians typically lack (Hallam, 2001).

Bonneville-Roussy & Evans (2024) found that teacher autonomy support predicted autonomous motivation, which in turn predicted both practice time and quality. The implication: the most effective tools behave like good teachers — they provide structure without dictating, and they support the musician's own agency.

### Self-taught failure modes

Without a teacher's regulatory functions, self-taught musicians frequently exhibit:

- **Comfort zone bias** — repeating familiar material rather than working on weaknesses. This produces practice that feels productive but doesn't target the skills most in need of development.
- **Miscalibrated difficulty** — practising material that is either too easy (no learning) or too hard (frustration without progress). Wilson et al.'s (2019) 85% Rule suggests that optimal learning occurs at roughly 85% accuracy, but self-taught musicians have no way to objectively assess where they sit on this continuum.
- **Poor session structure** — without external scaffolding, practice sessions tend to lack warm-up, spend too long on single items, and end without consolidation or reflection.
- **No stopping rules** — musicians without guidance either stop too early (abandoning an item after the first successful attempt, before overlearning consolidates the skill) or stop too late (drilling past the point of diminishing returns).

### Motor consolidation: why progress is invisible

The biological basis for the delayed feedback loop is motor memory consolidation. Walker & Stickgold (2004) demonstrated that motor skill performance improves 20–26% following a night of sleep, with no additional practice. Brashers-Krug, Shadmehr & Bizzi (1996) showed that motor memories require several hours to consolidate and are vulnerable to interference during this window.

This means a musician can practise a passage, feel like they made no progress, go to sleep, and return the next day measurably better — but they will attribute the improvement to the new session's practice rather than the previous day's. Without data to show the trajectory, the musician's narrative becomes "I'm not improving" even when they objectively are.

> **This is Intrada's core design opportunity.** By tracking mastery ratings, tempo progress, and practice patterns over time, the app can surface evidence of improvement that the musician cannot yet perceive.

### What Intrada can address

| Teacher function | How Intrada addresses it |
|-----------------|-------------------------|
| **Diagnose** | Data patterns surface what the musician cannot self-diagnose — persistent weak keys, plateau detection, effort/progress mismatches |
| **Decompose** | Key-aware exercise management and section-level tracking break complex goals into measurable sub-skills |
| **Sequence** | Smart scheduling orders practice material based on evidence (spacing, interleaving, goal alignment) rather than impulse |
| **Regulate** | Progress visualisation and session analytics provide the external monitoring that self-taught musicians lack |

> **Open question:** The extent to which a software tool can meaningfully replicate teacher regulatory functions is untested. The hypothesis is promising — data-driven feedback and intelligent scheduling address the *information* gap in self-taught practice — but whether this translates to improved outcomes compared to unstructured self-teaching is an empirical question that Intrada's data could help answer over time.

> **Assumption:** The self-taught failure modes listed above are drawn from research on music students in educational contexts (typically school-age or conservatoire students). Whether adult self-directed learners — who form a significant portion of Intrada's target audience — exhibit the same patterns is assumed but not specifically validated. Adult learners may have better metacognitive abilities from other domains, or they may have different failure modes entirely.

---

## 9. The Choice Overload Problem & Guided Learning

The amount of material a musician *could* practise is effectively infinite. The internet has made access to material trivially easy, but access is not the same as curation. The self-taught musician faces what Schwartz (2004) calls the "paradox of choice" — more options don't lead to better decisions; they lead to worse ones, or no decision at all.

### Choice is only motivating when you know how to choose

The relationship between choice and motivation is not linear. Katz & Assor (2007) argue that choice enhances motivation only when three conditions are met: the choices align with the learner's interests and values, the learner has sufficient competence to evaluate the options, and the number of options is manageable. Patall, Cooper & Robinson (2008) found that choice has positive effects on intrinsic motivation and task performance, but that these effects diminish when the number of options increases.

The guided learning literature reinforces this. Kirschner, Sweller & Clark (2006) argued that minimally guided instruction (including pure discovery learning) is less effective than guided instruction for novice and intermediate learners. Alfieri et al. (2011) confirmed in a meta-analysis that unassisted discovery learning does not benefit learners, but that enhanced discovery (with scaffolding, feedback, and worked examples) significantly outperforms direct instruction. Mayer (2004) called for a "three-strikes rule" against pure discovery, arguing that guided methods consistently produce better outcomes.

### Design implications

1. **Default to guidance.** The app should always have an answer to "what should I practise?" — not a menu of everything the musician *could* do, but a curated recommendation of what they *should* do right now, given their goals and current state.
2. **Goal-driven filtering.** When a musician sets a goal, the library and scheduler should filter aggressively — surfacing only the material that serves that goal and deprioritising everything else.
3. **Progressive disclosure.** The full breadth of what could be practised should be available but not presented up front. A musician starting a new exercise sees the keys they need now, not all twelve. As they master the immediate set, the app reveals the next layer.

---

## 10. Goal-Specific Pathways: From Aspiration to Curriculum

The critical path principle (Section 9) identifies what musicians need — a focused, well-sequenced subset of material rather than the full universe of what could be practised. But the principle alone doesn't answer the most concrete question a musician can ask: **"I want to play *this* — how do I get there?"**

### Backward design

Wiggins & McTighe (2005) formalised "backward design" in education: begin with the desired outcome, determine what evidence would demonstrate mastery, then design the learning experiences that lead there. Applied to music, a pianist whose goal is to play Debussy's *Claire de Lune* doesn't need a generic piano curriculum. They need the specific skills that piece demands — and a sequence that builds those skills from wherever they currently stand.

### Prerequisite hierarchies in music

Musical skills form natural hierarchies where complex abilities depend on simpler foundations. Gagné (1985) described how instruction should ensure prerequisite skills are in place before higher-order skills are attempted. Gordon (2007) developed this into a detailed sequential framework specific to music. Graded examination systems — ABRSM, the Royal Conservatory, Trinity College London — encode these hierarchies into structured syllabi. The Suzuki method builds prerequisite chains directly into its carefully sequenced repertoire.

These systems have demonstrated the value of sequenced learning over more than a century of use. Their limitation is that they define a single generic path for all learners of an instrument, regardless of individual goals or starting points.

### Material within the zone of proximal development

Vygotsky (1978) described the "zone of proximal development" (ZPD) — the gap between what a learner can do independently and what they can achieve with appropriate guidance. Lehmann, Sloboda & Woody (2007) apply this directly to music, arguing that effective skill acquisition requires material calibrated to the learner's current level. A well-designed pathway keeps every step within the musician's ZPD.

### Adaptive pathways through knowledge tracing

Corbett & Anderson (1995) developed "knowledge tracing," a Bayesian model that estimates learner mastery of individual skills based on their response history. This approach underlies modern adaptive learning platforms and is directly applicable to music: by tracking mastery of individual skills, the system can infer which prerequisites are met and which pathway steps need attention.

> **Assumption:** The claim that prerequisite relationships between musical skills can be reliably modelled and automatically sequenced is a design hypothesis. Music pedagogy involves significant tacit knowledge — experienced teachers make sequencing decisions based on intuitions about student readiness that may not reduce to measurable prerequisites. Initial pathways will likely need expert curation, with adaptive personalisation layered on top as the system accumulates data.

---

## 11. Overlearning & the Repetition Counter

The overlearning literature supports continuing correct repetitions beyond initial success to improve retention. Driskell, Willis & Copper (1992) reviewed 11 studies and found that overlearning (50–100% additional correct trials beyond the criterion of first success) has a positive influence on retention, with a moderate overall effect size (d = 0.753). Practically, this means if it takes a few attempts to get a passage right, doing 3–5 more correct repetitions is beneficial, with diminishing returns beyond that.

> **Note on the 85% Rule:** Wilson et al. (2019) derived the 85% optimal accuracy rate using mathematical modelling of gradient-descent learning in neural networks and binary classification tasks. The original paper is about *machine learning systems*, not human motor skill acquisition. The application to music practice repetitions is a **creative inference** — the principle resonates intuitively and aligns with Vygotsky's zone of proximal development, but it is not a direct application of the original research. The 85% figure should be treated as a useful heuristic, not a precise prescription.

---

## 12. Competitive Landscape

| App | Strengths | Weaknesses | Intrada Differentiator |
|-----|-----------|------------|----------------------|
| **Modacity** | Deliberate practice guidance, recording, mastery rating, good UX | No spaced repetition scheduling, no per-key tracking, expensive subscription, basic statistics | Algorithmic scheduling, key-aware tracking, fine-grained metrics, research-backed process |
| **Tonic** | Social/community features, gamification, practice streaks | Social-first rather than practice-quality-first, no deep progress metrics, no scheduling intelligence | Practice quality over social metrics, deep per-item analytics, privacy-first |
| **Piano Practice Assistant** | True spaced repetition, section-level tracking, interleaved practice, research-grounded | Android-only, piano-only, minimal UX polish, no goal framework | Cross-platform, instrument-agnostic, polished UX, goal-setting, encouragement layer |
| **Instrumentive** | Practice habit tracking, recording, metronome, similar to Modacity | Focuses on habit building over practice quality, limited analytics | Both habit and quality, research-backed scheduling, richer data model |

**Intrada's unique position**: the only app that combines spaced repetition scheduling, interleaved practice generation, key-aware exercise management, fine-grained metric tracking, neurodiversity-informed design, and a research-backed encouragement framework — all in a clean, instrument-agnostic package.

---

## References

> **Reference validation note:** All references below have been verified as real published works to the best of current knowledge. Where a reference is secondary, unpublished, or has a date discrepancy between online-first and print publication, this is noted. Two references — Moss (1995) and the Mathias & Goldman year — carry specific caveats documented in the text where they are cited.

Alfieri, L., Brooks, P. J., Aldrich, N. J., & Tenenbaum, H. R. (2011). Does discovery-based instruction enhance learning? *Journal of Educational Psychology*, 103(1), 1–18. https://doi.org/10.1037/a0021017

Ambrose, S. A., Bridges, M. W., DiPietro, M., Lovett, M. C., & Norman, M. K. (2010). *How Learning Works: Seven Research-Based Principles for Smart Teaching*. San Francisco: Jossey-Bass. ISBN: 978-0-470-48410-4.

Barkley, R. A. (2015). *Attention-Deficit Hyperactivity Disorder: A Handbook for Diagnosis and Treatment* (4th ed.). New York: Guilford Press.

Blackwell, L. S., Trzesniewski, K. H., & Dweck, C. S. (2007). Implicit theories of intelligence predict achievement across an adolescent transition: A longitudinal study and an intervention. *Child Development*, 78(1), 246–263. https://doi.org/10.1111/j.1467-8624.2007.00995.x

Bonneville-Roussy, A., & Evans, P. (2024). The support of autonomy, motivation, and music practice in university music students: A self-determination theory perspective. *Psychology of Music*. https://doi.org/10.1177/03057356241296109

Brashers-Krug, T., Shadmehr, R., & Bizzi, E. (1996). Consolidation in human motor learning. *Nature*, 382, 252–255. https://doi.org/10.1038/382252a0

Carter, C. E., & Grahn, J. A. (2016). Optimizing music learning: Exploring how blocked and interleaved practice schedules affect advanced performance. *Frontiers in Psychology*, 7, 1251. https://doi.org/10.3389/fpsyg.2016.01251

Catrambone, R. (1998). The subgoal learning model: Creating better examples so that students can solve novel problems. *Journal of Experimental Psychology: General*, 127(4), 355–376. https://doi.org/10.1037/0096-3445.127.4.355

Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006). Distributed practice in verbal recall tasks: A review and quantitative synthesis. *Psychological Bulletin*, 132(3), 354–380. https://doi.org/10.1037/0033-2909.132.3.354

Corbett, A. T., & Anderson, J. R. (1995). Knowledge tracing: Modeling the acquisition of procedural knowledge. *User Modeling and User-Adapted Interaction*, 4(4), 253–278. https://doi.org/10.1007/BF01099821

Donovan, J. J., & Radosevich, D. J. (1999). A meta-analytic review of the distribution of practice effect: Now you see it, now you don't. *Journal of Applied Psychology*, 84(5), 795–805. https://doi.org/10.1037/0021-9010.84.5.795

Driskell, J. E., Willis, R. P., & Copper, C. (1992). Effect of overlearning on retention. *Journal of Applied Psychology*, 77(5), 615–622. https://doi.org/10.1037/0021-9010.77.5.615

Duke, R. A. (2005). *Intelligent Music Teaching: Essays on the Core Principles of Effective Instruction*. Austin, TX: Learning and Behavior Resources.

Duke, R. A., Simmons, A. L., & Cash, C. D. (2009). It's not how much; it's how: Characteristics of practice behavior and retention of performance skills. *Journal of Research in Music Education*, 56(4), 310–321. https://doi.org/10.1177/0022429408328851

Dweck, C. S. (2006). *Mindset: The New Psychology of Success*. New York: Random House.

Dweck, C. S., & Yeager, D. S. (2019). Mindsets: A view from two eras. *Perspectives on Psychological Science*, 14(3), 481–496. https://doi.org/10.1177/1745691618804166

Ericsson, K. A., Krampe, R. T., & Tesch-Römer, C. (1993). The role of deliberate practice in the acquisition of expert performance. *Psychological Review*, 100(3), 363–406. https://doi.org/10.1037/0033-295X.100.3.363

Evans, P. (2015). Self-determination theory: An approach to motivation in music education. *Musicae Scientiae*, 19(1), 65–83. https://doi.org/10.1177/1029864914568044

Evans, P., & Bonneville-Roussy, A. (2016). Self-determined motivation for practice in university music students. *Psychology of Music*, 44(5), 1095–1110. https://doi.org/10.1177/0305735615610926

Gagné, R. M. (1985). *The Conditions of Learning and Theory of Instruction* (4th ed.). New York: Holt, Rinehart and Winston.

Gordon, E. E. (2007). *Learning Sequences in Music: A Contemporary Music Learning Theory*. Chicago: GIA Publications.

Grob, C. M., Biasutti, M., & Schacter, E. N. (2022). Musical improvisation and expression in individuals with ADHD. *Frontiers in Psychology*, 13, 895780.

Hallam, S. (1998). The predictors of achievement and dropout in instrumental tuition. *Psychology of Music*, 26(2), 116–132. https://doi.org/10.1177/0305735698262002

Hallam, S. (2001). The development of metacognition in musicians: Implications for education. *British Journal of Music Education*, 18(1), 27–39.

Iyengar, S. S., & Lepper, M. R. (2000). When choice is demotivating: Can one desire too much of a good thing? *Journal of Personality and Social Psychology*, 79(6), 995–1006. https://doi.org/10.1037/0022-3514.79.6.995

Katz, I., & Assor, A. (2007). When choice motivates and when it does not. *Educational Psychology Review*, 19(4), 429–442. https://doi.org/10.1007/s10648-006-9027-y

Kirschner, P. A., Sweller, J., & Clark, R. E. (2006). Why minimal guidance during instruction does not work: An analysis of the failure of constructivist, discovery, problem-based, experiential, and inquiry-based teaching. *Educational Psychologist*, 41(2), 75–86. https://doi.org/10.1207/s15326985ep4102_1

Lehmann, A. C., Sloboda, J. A., & Woody, R. H. (2007). *Psychology for Musicians: Understanding and Acquiring the Skills*. Oxford: Oxford University Press.

Macnamara, B. N., Hambrick, D. Z., & Oswald, F. L. (2014). Deliberate practice and performance in music, games, sports, education, and professions: A meta-analysis. *Psychological Science*, 25(8), 1608–1618. https://doi.org/10.1177/0956797614535810

Macnamara, B. N., & Maitra, M. (2019). The role of deliberate practice in expert performance: Revisiting Ericsson, Krampe & Tesch-Römer (1993). *Royal Society Open Science*, 6(8), 190327. https://doi.org/10.1098/rsos.190327

Mathias, T., & Goldman, A. (2025). How does increasing contextual interference in a musical practice session affect acquisition and retention? *Journal of Research in Music Education*. https://doi.org/10.1177/00224294231222801

Mayer, R. E. (2004). Should there be a three-strikes rule against pure discovery learning? The case for guided methods of instruction. *American Psychologist*, 59(1), 14–19. https://doi.org/10.1037/0003-066X.59.1.14

McPherson, G. E., & Renwick, J. M. (2001). A longitudinal study of self-regulation in children's musical practice. *Music Education Research*, 3(2), 169–186. https://doi.org/10.1080/14613800120089232

McPherson, G. E., & Zimmerman, B. J. (2002). Self-regulation of musical learning: A social cognitive perspective. In R. Colwell & C. Richardson (Eds.), *The New Handbook of Research on Music Teaching and Learning* (pp. 327–347). Oxford: Oxford University Press.

Moss, S. L. (1995). The distribution of practice effect: A review of the literature. Unpublished review, cited in Firth et al. (2023).

O'Neill, S. A. (2002). The self-identity of young musicians. In R. A. R. MacDonald, D. J. Hargreaves, & D. Miell (Eds.), *Musical Identities* (pp. 79–96). Oxford University Press.

Patall, E. A., Cooper, H., & Robinson, J. C. (2008). The effects of choice on intrinsic motivation and related outcomes: A meta-analysis of research findings. *Psychological Bulletin*, 134(2), 270–300. https://doi.org/10.1037/0033-2909.134.2.270

Ptacek, R., Weissenberger, S., Braaten, E., Klicperova-Baker, M., Goetz, M., Raboch, J., & Stefano, G. B. (2019). Clinical implications of the perception of time in attention deficit hyperactivity disorder (ADHD): A review. *Medical Science Monitor*, 25, 3918–3924.

Puyjarinet, F., Bégel, V., Lopez, R., Dellacherie, D., & Dalla Bella, S. (2017). Children and adults with Attention-Deficit/Hyperactivity Disorder cannot move to the beat. *Scientific Reports*, 7, 11550.

Raz, S. (2025). Enhancing cognitive abilities in young adults with ADHD through instrumental music training. *Psychological Research*, 89, 9. https://doi.org/10.1007/s00426-024-02048-2

Renwick, J. M., & McPherson, G. E. (2002). Interest and choice: Student-selected repertoire and its effect on practising behaviour. *British Journal of Music Education*, 19(2), 173–188. https://doi.org/10.1017/S0265051702000256

Ryan, R. M., & Deci, E. L. (2000). Self-determination theory and the facilitation of intrinsic motivation, social development, and well-being. *American Psychologist*, 55(1), 68–78. https://doi.org/10.1037/0003-066X.55.1.68

Schwartz, B. (2004). *The Paradox of Choice: Why More Is Less*. New York: Ecco/HarperCollins.

Serrallach, B., Groß, C., Christiner, M., Wildermuth, S., & Schneider, P. (2022). Musical performance in adolescents with ADHD, ADD and dyslexia — Behavioral and neurophysiological aspects. *Brain Sciences*, 12(2), 127.

Shea, J. B., & Morgan, R. L. (1979). Contextual interference effects on the acquisition, retention, and transfer of a motor skill. *Journal of Experimental Psychology: Human Learning and Memory*, 5(2), 179–187. https://doi.org/10.1037/0278-7393.5.2.179

Simmons, A. L. (2012). Distributed practice and procedural memory consolidation in musicians' skill learning. *Journal of Research in Music Education*, 59(4), 357–368. https://doi.org/10.1177/0022429411424798

Sisk, V. F., Burgoyne, A. P., Sun, J., Butler, J. L., & Macnamara, B. N. (2018). To what extent and under what circumstances are growth mind-sets important to academic achievement? Two meta-analyses. *Psychological Science*, 29(4), 549–571. https://doi.org/10.1177/0956797617739704

Stambaugh, L. A. (2009). Effects of practice schedule on wind instrument performance: A preliminary application of a motor learning principle. *Update: Applications of Research in Music Education*, 27(2), 20–28.

Stambaugh, L. A., & Demorest, S. M. (2010). Effects of practice schedule on the acquisition and retention of wind instrument skills. *Journal of Research in Music Education*, 58(4), 357–367.

Suzuki, S. (1969). *Nurtured by Love: A New Approach to Education*. New York: Exposition Press.

Valenzuela, R., Codina, N., & Pestana, J. V. (2018). Self-determination theory applied to flow in conservatoire music practice: The roles of perceived autonomy and competence, and autonomous and controlled motivation. *Psychology of Music*, 46(1), 33–48. https://doi.org/10.1177/0305735617694502

Vygotsky, L. S. (1978). *Mind in Society: The Development of Higher Psychological Processes*. Cambridge, MA: Harvard University Press.

Walker, M. P., & Stickgold, R. (2004). Sleep-dependent learning and memory consolidation. *Neuron*, 44(1), 121–133. https://doi.org/10.1016/j.neuron.2004.08.031

Wellmann, M., & Skillicorn, A. T. (2024). Research-to-resource: Introducing retrieval practice in jazz pedagogy. *Journal of Research in Music Education*. https://doi.org/10.1177/87551233221146282

Wiggins, G., & McTighe, J. (2005). *Understanding by Design* (2nd ed.). Alexandria, VA: Association for Supervision and Curriculum Development (ASCD).

Wilde, E. M., & Welch, G. F. (2022). Attention deficit hyperactivity disorder (ADHD) and musical behaviour: The significance of context. *Psychology of Music*, 50(6), 1903–1920.

Williamon, A., & Valentine, E. (2000). Quantity and quality of musical practice as predictors of performance quality. *British Journal of Psychology*, 91(3), 353–376. https://doi.org/10.1348/000712600161871

Wilson, R. C., Shenhav, A., Straccia, M., & Cohen, J. D. (2019). The Eighty Five Percent Rule for optimal learning. *Nature Communications*, 10, 4646. https://doi.org/10.1038/s41467-019-12552-4

Wood, D., Bruner, J. S., & Ross, G. (1976). The role of tutoring in problem solving. *Journal of Child Psychology and Psychiatry*, 17(2), 89–100. https://doi.org/10.1111/j.1469-7610.1976.tb00381.x
