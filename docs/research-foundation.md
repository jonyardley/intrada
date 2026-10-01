# Intrada: research foundation

*Why intrada works the way it does: the research behind its design, and how
strong the evidence is. First written April 2026 for the
[product vision](../VISION.md); revised September 2026.*

Each section summarises what the studies found, flags where the evidence is
weak or the jump to music is an assumption, and ends with what intrada does
with it. Where something is not built yet, the section says so. What gets built
next, and in what order, is in the [roadmap](roadmap.md).

---

## 1. Spaced Repetition

The spacing effect is one of the best-supported findings in learning science. Material reviewed at increasing intervals is retained far more effectively than material crammed into a single session. A meta-analysis by Donovan & Radosevich (1999) across 63 studies with 112 effect sizes found an overall mean weighted effect size of d=0.46, indicating that spaced practice conditions significantly outperform massed practice conditions. A later meta-analysis by Cepeda et al. (2006), reviewing 317 experiments across 184 articles, confirmed the spacing benefit and found that the best spacing interval grows as the desired retention interval grows.

**What this means for music:** Donovan & Radosevich also found that the spacing effect is moderated by task complexity: more complex tasks benefit less from distributed practice than simpler ones. Musical skill involves complex motor coordination, and Simmons (2012) found no spacing effect when teaching a 17-note piano sequence to novices, suggesting that the effect "may not always be demonstrable for complex motor skills." However, Moss (1995), reviewing 120 articles, found spacing improved learning of motor skills in over 80% of studies reviewed. The weight of evidence supports spacing for music, but any spacing intrada does applies a *general principle with strong support*, not a system calibrated specifically for instrument practice.

> **Citation note:** Moss (1995) is an unpublished review cited secondarily via Firth et al. (2023). The specific "120 articles" and "80%+" claims cannot be independently verified from the primary source. The broader claim, that spacing benefits motor skills, is well supported by published meta-analyses, but this specific citation should be treated with caution.

**Scheduling algorithms:** SM-2 was designed for verbal flashcard learning (Wozniak, 1987), and its interval growth parameters are calibrated for declarative memory. Musical skill learning involves procedural and motor memory, which consolidates differently (Walker & Stickgold, 2004). No published research validates SM-2 parameters for scheduling motor skills, so any intervals intrada uses would be informed guesses, refined from real use.

**What intrada does:** it records when each item was last practised and shows the items going cold, which the Up next card draws on; today's plan will too (#57). Spacing reviews by how well an item is going is planned under #1978.

---

## 2. Interleaved Practice

Carter & Grahn (2016) compared blocked and interleaved practice schedules with advanced clarinettists (n=10) in a realistic practice setting. Whenever there was a ratings difference between conditions, pieces practised in the interleaved schedule were rated higher by expert assessors, though results varied across raters. Participant questionnaires showed that interleaved practice helped goal setting, focus and spotting mistakes. While 6 of 10 participants found the interleaved schedule more useful, most still preferred the blocked schedule, which fits a well-documented pattern: blocked practice feels more productive despite producing worse retention.

This "contextual interference effect" has broader support in motor skill research (Shea & Morgan, 1979; Magill & Hall, 1990), though results in music specifically are mixed. Stambaugh (2009) found interleaving helped beginner clarinettists, but Stambaugh & Demorest (2010) found no effect on technical accuracy in intermediate students. A more recent study by Mathias & Goldman (2025) with violinists explored increasing contextual interference within sessions, suggesting that a gradual shift from blocked to interleaved practice within a session may work best.

**What intrada does:** the musician groups and orders a session however they like. Because musicians tend to resist interleaving despite its benefits, any mixing the app suggests stays a suggestion. The Mathias & Goldman finding points to one shape worth trying: start a session with similar items and increase the variety as it goes. Neither is built.

---

## 3. Deliberate Practice

Ericsson, Krampe & Tesch-Römer (1993) introduced the deliberate practice framework, proposing that expert performance results from prolonged, focused, effortful practice designed to improve specific aspects of performance. Their original study with violinists at a Berlin music academy found that the best violinists had accumulated significantly more solitary practice hours than less accomplished peers by age 20. This work has been enormously influential across domains.

The evidence has been refined considerably since 1993. A meta-analysis by Macnamara, Hambrick & Oswald (2014) found that deliberate practice explained only about 21% of variance in music performance: substantial and practically meaningful, but far from the "sufficient account" Ericsson originally claimed. A direct replication by Macnamara & Maitra (2019), using improved double-blind methods, found the effect was "substantial, but considerably smaller than the original study's effect size," explaining about 26% of performance variance. They also found that practice designed by the performers themselves was perceived as more relevant to improvement than teacher-designed practice.

What remains well supported is the *qualitative* side of deliberate practice: focused attention on specific goals, working at the edge of one's ability, and acting on feedback lead to better outcomes than mindless repetition. Ambrose et al. (2010) synthesise this broader evidence, emphasising that a cycle of practising, evaluating and adjusting is central to developing a skill.

Ericsson's framework also emphasised that deliberate practice is effortful and needs recovery. An app that structures practice could also support rest, for example by noticing when a musician's practice volume runs well above their usual; this is not built.

**What intrada does:** each item in a session can carry an aim, gets a score at the end, and offers a short note on what improved and what didn't. The claim is that practice *quality* matters enormously, not that quantity alone decides the outcome.

---

## 4. Self-Determination Theory (SDT)

Ryan & Deci's self-determination theory identifies three basic psychological needs that sustain motivation: autonomy (feeling in control of one's actions), competence (feeling effective and capable), and relatedness (feeling connected to others). Evans & Bonneville-Roussy (2016) found that university music students' autonomous motivation predicted both how often they practised and the proportion of sessions they reported as highly productive.

Valenzuela, Codina & Pestana (2018) studied the theory in conservatoire instrument practice (N=162) and found that perceived competence was the strongest predictor of flow during practice. Autonomous motivation had a direct positive effect on flow, while controlled motivation had the opposite effect. Visible progress and evidence of improvement feed the sense of competence that sustains motivation and flow.

Bonneville-Roussy & Evans (2024), studying 213 university music students across the UK and Canada, found that a teacher's support for autonomy predicted autonomous motivation, which in turn predicted both practice time and practice quality. Intrada should suggest and support, and leave the decisions to the musician.

**What intrada does:** autonomy through the musician choosing what matters (stars now, the week's intent with #1926), and competence through tracking each item and key and showing the growth. A plan the musician can always change (#57) and encouragement about the process are planned.

---

## 5. Growth Mindset & Process Trust

Dweck's implicit theories framework (Dweck, 2006) distinguishes between a growth mindset (the belief that abilities can be developed through effort and learning) and a fixed mindset (the belief that abilities are innate and unchangeable). In educational contexts, Blackwell, Trzesniewski & Dweck (2007) found that students with a growth mindset outperformed fixed-mindset peers over time, even when controlling for initial achievement. Music is a domain where fixed-mindset beliefs about "talent" are particularly common (O'Neill, 2002).

The growth mindset literature has faced legitimate scrutiny. A meta-analysis by Sisk et al. (2018) found that mindset interventions had only weak effects on academic achievement overall, though effects were larger for academically at-risk students. Dweck & Yeager (2019) responded that mindset interventions work best when they are sustained and embedded in supportive contexts, rather than delivered once.

For intrada, the takeaway is that the *design of the tool itself* can reinforce growth-oriented beliefs. Progress charts give objective evidence that effort leads to improvement. Encouragement tied to specific data points ("Your Db went from 2 to 4 over three weeks") is the kind of effort-linked feedback Dweck's research associates with adaptive motivation.

**What intrada does about gaps in practice:** streaks create anxiety about breaking them, and can trigger shame-driven avoidance, particularly in musicians with ADHD. Intrada shows no streak; the weekly bars show how much was practised each week without counting a gap. Words that celebrate the return, such as "You've practised 4 of the last 7 days, which is good spacing for retention", are the intended tone, and are not built.

> **Assumption:** The claim that streaks cause anxiety and avoidance is a design hypothesis, not a research finding. There is general psychology research on shame-avoidance cycles and ADHD emotional dysregulation (Barkley, 2015), but no published study directly compares streak-based and comeback-based tracking in practice apps. The choice follows self-determination theory (supporting autonomy) and practitioners' experience. It is a reasonable design decision, but should not be presented as proven.

---

## 6. Retrieval Practice

Retrieval practice is the finding that actively recalling information produces better long-term retention than passive review. It is extensively studied in verbal learning; Wellmann & Skillicorn (2024) recently proposed the first systematic application of it to jazz education, arguing that its benefits should extend to music because the underlying memory mechanisms are domain-general. They recommend spaced retrieval with at least 24 hours between sessions on the same material.

**What intrada does:** it favours coming back to material across sessions over drilling it repeatedly in one, which uses both spacing and retrieval.

> **Open question:** Wellmann & Skillicorn's proposal is theoretical. They argue retrieval practice *should* benefit music learning because memory mechanisms are domain-general, but this has not been tested in music practice. The principle is sound; its transfer to complex motor skills with an expressive side remains an assumption.

---

## 7. Neurodiversity & Music Practice

Research on ADHD and music shows both specific challenges and real strengths, and both shape intrada's design.

**Starting and planning.** ADHD involves difficulties with starting tasks, planning, organisation and working memory (Barkley, 2015). Starting a practice session is often harder than sustaining one: the ADHD brain struggles to move from thinking about practising to actually practising. Every decision before playing begins is a chance to stall, so the path from opening the app to playing should involve as few decisions as possible.

**Time blindness.** ADHD disrupts the sense of time passing (Ptacek et al., 2019). Musicians with ADHD may badly misjudge how long they've been practising, lose track of time while hyperfocused on one item (neglecting others), or struggle to pace a session. This is a difference in perception rather than poor time management, and external time cues can help.

**Hyperfocus.** ADHD brains can hyperfocus on engaging tasks, which for a musician might mean 45 minutes on one satisfying piece while the scale work that needs attention is neglected. A session planned in advance, with the time on each item visible, supplies from outside the structure the ADHD brain struggles to provide for itself.

**Frustration.** Frustration tolerance is often lower in ADHD. A session that keeps surfacing difficult material with no sense of progress can end in shutdown. For ADHD musicians, encouragement and visible progress may be the difference between keeping going and giving up.

**Music-specific findings.** Research identifies timing difficulties in ADHD, including beat tracking and processing short time intervals (Puyjarinet et al., 2017; Serrallach et al., 2022). These do not extend to improvisation and musical expression (Grob et al., 2022), and Wilde & Welch (2022) found that ADHD behaviours were often *absent* during active music-making. Raz (2025) found that musicians with ADHD showed stronger cognitive abilities than non-musician ADHD peers, including better sustained attention and impulse control. So the practice itself may be less affected than everything around it: deciding what to practise, starting, moving between items and stopping.

**Beyond ADHD.** Sensory processing differences (common in autism), dyslexia (affecting 10 to 15% of the population) and other cognitive variations all affect how musicians use a practice tool. What supports neurodivergent musicians, such as less visual clutter, predictable navigation, adjustable feedback and readable type, helps every musician.

---

## 8. The Feedback Loop Problem & Self-Taught Learning

### What teachers actually do

Demonstrating technique is a small part of effective music teaching. Duke (2005) describes the core teacher functions as: **diagnose** (identify what specifically is going wrong), **decompose** (break complex skills into manageable parts), **sequence** (order the learning steps) and **regulate** (monitor progress and adjust the plan). These are forms of metacognition that expert musicians have internalised but developing musicians typically lack (Hallam, 2001).

Bonneville-Roussy & Evans (2024) found that a teacher's support for autonomy predicted autonomous motivation, which in turn predicted both practice time and quality. The most effective tools behave like good teachers: they give structure, and leave the musician in charge.

### How self-taught practice goes wrong

Without a teacher doing those four jobs, self-taught musicians frequently show:

- **Comfort zone bias:** repeating familiar material rather than working on weaknesses, which feels productive but misses the skills that most need work.
- **Miscalibrated difficulty:** material that is either too easy (no learning) or too hard (frustration without progress). Wilson et al.'s (2019) 85% Rule suggests learning is best at roughly 85% accuracy, but self-taught musicians have no objective way to tell where they sit.
- **Poor session structure:** without outside structure, sessions tend to lack a warm-up, spend too long on single items, and end without consolidation or reflection.
- **No stopping rules:** musicians without guidance either stop too early (abandoning an item after the first success, before extra repetitions consolidate it) or too late (drilling past the point of diminishing returns).

### Motor consolidation: why progress is invisible

The biological basis for the slow feedback loop is motor memory consolidation. Walker & Stickgold (2004) showed that motor skill performance improves 20 to 26% after a night of sleep, with no additional practice. Brashers-Krug, Shadmehr & Bizzi (1996) showed that motor memories take several hours to consolidate and are vulnerable to interference during that window.

So a musician can practise a passage, feel they made no progress, sleep, and come back measurably better, but credit the improvement to the new session rather than the previous day's. Without data showing the trajectory, the story becomes "I'm not improving" even when they objectively are.

> **This is intrada's core opportunity.** By tracking scores, tempo and practice patterns over time, the app can show evidence of improvement the musician cannot yet perceive.

### What intrada can take on

| Teacher function | How intrada helps |
|-----------------|-------------------|
| **Diagnose** | Later: patterns in the record show what the musician cannot self-diagnose, such as persistent weak keys, plateaus and effort that is not turning into progress |
| **Decompose** | Tracking by key, and pieces linked to the exercises that build them, break a big goal into measurable parts |
| **Sequence** | The Up next card suggests from what the musician starred and what is going cold; today's plan (#57), then spacing and priorities (#1978), are planned, always changeable |
| **Regulate** | Scores, tempo and the session record give the outside view of progress that self-taught musicians lack |

> **Open question:** How far a software tool can take on a teacher's regulating role is untested. The hypothesis is promising, because data and planning address the *information* gap in self-taught practice, but whether that leads to better outcomes than unstructured self-teaching is an empirical question intrada's own record could help answer.

> **Assumption:** The failure modes above come from research on music students in education (typically school-age or conservatoire students). Whether adult self-directed learners, a large part of intrada's audience, show the same patterns is assumed, not validated. Adults may bring better metacognitive skills from other areas, or fail in different ways entirely.

---

## 9. The Choice Overload Problem & Guided Learning

The amount of material a musician *could* practise is effectively infinite. The internet has made material trivially easy to find, but finding is not the same as choosing well. The self-taught musician faces what Schwartz (2004) calls the "paradox of choice": more options lead to worse decisions, or none at all.

### Choice motivates only when you know how to choose

Katz & Assor (2007) argue that choice increases motivation only when three conditions are met: the options align with the learner's interests and values, the learner is competent enough to judge them, and there are few enough of them. Patall, Cooper & Robinson (2008) found that choice has positive effects on intrinsic motivation and task performance, but that these effects shrink as the number of options grows.

The guided learning literature agrees. Kirschner, Sweller & Clark (2006) argued that minimally guided instruction (including pure discovery learning) is less effective than guided instruction for novice and intermediate learners. Alfieri et al. (2011) confirmed in a meta-analysis that unassisted discovery learning does not help learners, but that enhanced discovery (with scaffolding, feedback and worked examples) clearly outperforms direct instruction. Mayer (2004) called for a "three-strikes rule" against pure discovery, arguing that guided methods consistently produce better outcomes.

### What intrada does

1. **Have a starting point ready.** The Up next card answers "what should I practise?" with one suggestion in place of a menu of everything. Today's plan will go further: a short plan the musician can start in one tap or change (#57).
2. **Let the week narrow the choice.** The items the musician has starred, and later the week's intent (#1926), decide what today's plan draws from.
3. **Show only what is needed now.** The full breadth of what could be practised stays available without being put up front: a musician starting a new exercise sees the keys they need now, not all twelve, with the next layer revealed as they master the first. This is an idea, not built.

---

## 10. Goal-Specific Pathways: From Aspiration to Curriculum

Section 9 is about narrowing everything that could be practised to a focused, well-ordered subset. It leaves open the most concrete question a musician can ask: **"I want to play *this*. How do I get there?"** This is the curriculum help in the vision's Future vision, and is not scheduled.

### Backward design

Wiggins & McTighe (2005) formalised "backward design" in education: begin with the desired outcome, decide what evidence would show mastery, then design the learning that leads there. Applied to music, a pianist whose goal is Debussy's *Clair de lune* does not need a generic piano curriculum. They need the specific skills that piece demands, in a sequence that builds them from wherever they stand now.

### Prerequisite hierarchies in music

Musical skills form natural hierarchies in which complex abilities depend on simpler ones. Gagné (1985) described how instruction should make sure prerequisite skills are in place before higher-order ones are attempted. Gordon (2007) developed this into a detailed sequential framework specific to music. Graded examination systems (ABRSM, the Royal Conservatory, Trinity College London) encode these hierarchies into structured syllabuses, and the Suzuki method builds prerequisite chains into its carefully sequenced repertoire.

These systems have shown the value of sequenced learning over more than a century. Their limit is that they define one generic path for every learner of an instrument, whatever their goal or starting point.

### Material within the zone of proximal development

Vygotsky (1978) described the "zone of proximal development": the gap between what a learner can do alone and what they can achieve with the right guidance. Lehmann, Sloboda & Woody (2007) apply this directly to music, arguing that skill grows best on material matched to the learner's current level. A well-designed pathway keeps every step inside that zone.

### Adaptive pathways through knowledge tracing

Corbett & Anderson (1995) developed "knowledge tracing," a Bayesian model that estimates a learner's mastery of individual skills from their history of responses. It underlies modern adaptive learning platforms and applies directly to music: by tracking mastery of individual skills, a system can infer which prerequisites are met and which steps need attention.

> **Assumption:** The claim that prerequisites between musical skills can be reliably modelled and sequenced automatically is a design hypothesis. Music teaching involves a lot of tacit knowledge: experienced teachers sequence by intuitions about a student's readiness that may not reduce to measurable prerequisites. First pathways would likely need an expert to write them, with personalisation added as the record grows.

---

## 11. Overlearning & the Repetition Counter

The overlearning literature supports continuing correct repetitions beyond the first success to improve retention. Driskell, Willis & Copper (1992) reviewed 11 studies and found that overlearning (50 to 100% additional correct trials beyond the first success) improves retention, with a moderate overall effect size (d = 0.753). In practice, if it takes a few attempts to get a passage right, 3 to 5 more correct repetitions help, with diminishing returns after that.

**What intrada does:** the practice screen's repetition counter counts those extra correct repetitions.

> **Note on the 85% Rule:** Wilson et al. (2019) derived the 85% accuracy figure by mathematically modelling gradient-descent learning in neural networks and binary classification tasks. The paper is about *machine learning systems*, not human motor learning. Applying it to music practice is a **creative inference**: it fits intuitively with Vygotsky's zone of proximal development, but is not a direct application of the research. Treat the 85% figure as a useful rule of thumb, not a precise prescription.

---

## 12. Competitive Landscape

| App | Strengths | Weaknesses | Where intrada differs |
|-----|-----------|------------|----------------------|
| **Modacity** | Deliberate practice guidance, recording, mastery rating, good design | No spacing, no per-key tracking, expensive subscription, basic statistics | Tracking by key, a record that only shows what was measured |
| **Tonic** | Social and community features, gamification, practice streaks | Social first rather than practice first, no deep progress metrics, no planning help | Practice quality over social metrics, progress per item and key, everything stays on the phone |
| **Piano Practice Assistant** | True spaced repetition, section-level tracking, interleaved practice, grounded in research | Android only, piano only, little design polish | Any instrument, iPhone, an encouraging tone |
| **Instrumentive** | Practice habit tracking, recording, metronome, similar to Modacity | Habit building over practice quality, limited analytics | Quality as well as habit, a richer record of each item |

**Intrada's position:** quick capture, tracking by key, an honest record and design for every kind of mind, in one app for any instrument. A plan the musician stays in charge of is next (#57). Spacing and mixing items within a session (sections 1 and 2) are ideas, not built yet.

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
