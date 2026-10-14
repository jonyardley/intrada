# Intrada: research foundation

*Why intrada works the way it does: the research behind its design, and how
strong the evidence is. Written for the [product vision](../VISION.md).*

Each section summarises what the studies found, flags where the evidence is
weak or the jump to music is an assumption, and ends with what intrada does
with it. Where something is not built yet, the section says so. What gets built
next is in the [roadmap](roadmap.md).

---

## 1. Spaced Repetition

The spacing effect is one of the best-supported findings in learning science: material reviewed at increasing intervals is retained far better than material crammed into one session. Donovan & Radosevich (1999), across 63 studies, found spaced practice beats massed practice with d=0.46. Cepeda et al. (2006), reviewing 317 experiments, confirmed the benefit and found the best interval grows with the desired retention interval.

**What this means for music:** complex tasks benefit less from spacing (Donovan & Radosevich, 1999), and Simmons (2012) found no spacing effect on a 17-note piano sequence for novices. Moss (1995) reports spacing helped motor skills in over 80% of 120 studies, but it is an unpublished review cited through Firth et al. (2023). Any spacing intrada does applies a *general principle with strong support*, not a system calibrated for instrument practice.

**Scheduling algorithms:** SM-2 was designed for verbal flashcards (Wozniak, 1987) and calibrated for declarative memory; motor memory consolidates differently (Walker & Stickgold, 2004). No published research validates SM-2 parameters for motor skills, so any intervals intrada uses are informed guesses, refined from real use.

**What intrada does:** it records when each item was last practised and shows the items going cold, which the Up next card draws on; today's plan will too (#57). Spacing reviews by how well an item is going is planned under #1978.

---

## 2. Interleaved Practice

Carter & Grahn (2016) compared blocked and interleaved practice schedules with advanced clarinettists (n=10) in a realistic practice setting. Whenever there was a ratings difference between conditions, pieces practised in the interleaved schedule were rated higher by expert assessors, though results varied across raters. Participant questionnaires showed that interleaved practice helped goal setting, focus and spotting mistakes. While 6 of 10 participants found the interleaved schedule more useful, most still preferred the blocked schedule, which fits a well-documented pattern: blocked practice feels more productive despite producing worse retention.

This "contextual interference effect" has broad support in motor skill research (Shea & Morgan, 1979; Magill & Hall, 1990), but music results are mixed. Stambaugh (2009) found interleaving helped beginner clarinettists; Stambaugh & Demorest (2010) found no effect on intermediate students' accuracy. Mathias & Goldman (2025) suggest a gradual shift from blocked to interleaved within a session may work best.

**What intrada does:** the musician groups and orders a session however they like. Because musicians resist interleaving, any mixing the app suggests stays a suggestion. One shape worth trying: start with similar items and increase the variety as the session goes. Neither is built.

---

## 3. Deliberate Practice

Ericsson, Krampe & Tesch-Römer (1993) proposed that expert performance comes from prolonged, focused, effortful practice aimed at specific weaknesses.

The evidence has been refined considerably since 1993. A meta-analysis by Macnamara, Hambrick & Oswald (2014) found that deliberate practice explained only about 21% of variance in music performance: substantial and practically meaningful, but far from the "sufficient account" Ericsson originally claimed. A direct replication by Macnamara & Maitra (2019), using improved double-blind methods, found the effect was "substantial, but considerably smaller than the original study's effect size," explaining about 26% of performance variance. They also found that practice designed by the performers themselves was perceived as more relevant to improvement than teacher-designed practice.

What remains well supported is the *qualitative* side: specific goals, working at the edge of one's ability, and a cycle of practising, evaluating and adjusting (Ambrose et al., 2010). It also needs recovery; noticing when practice volume runs well above usual is not built.

**What intrada does:** each item in a session can carry an aim, gets a score at the end, and offers a short note on what improved and what didn't. The claim is that practice *quality* matters, not that quantity alone decides the outcome.

---

## 4. Self-Determination Theory (SDT)

Ryan & Deci's (2000) self-determination theory names three needs that sustain motivation: autonomy (feeling in control), competence (feeling effective) and relatedness (feeling connected). Evans & Bonneville-Roussy (2016) found music students' autonomous motivation predicted how often they practised and how many sessions they rated highly productive.

Valenzuela, Codina & Pestana (2018) studied the theory in conservatoire instrument practice (N=162) and found that perceived competence was the strongest predictor of flow during practice. Autonomous motivation had a direct positive effect on flow, while controlled motivation had the opposite effect. Visible progress and evidence of improvement feed the sense of competence that sustains motivation and flow.

Bonneville-Roussy & Evans (2024), with 213 university music students, found a teacher's support for autonomy predicted autonomous motivation, which in turn predicted practice time and quality. Intrada should suggest and support, and leave the decisions to the musician.

**What intrada does:** autonomy through the musician choosing what matters (priorities now, the week's intent with #1926), and competence through tracking each item and key and showing the growth. A plan the musician can always change (#57) and encouragement about the process are planned.

---

## 5. Growth Mindset & Process Trust

Dweck (2006) distinguishes a growth mindset (abilities develop through effort) from a fixed one; growth-mindset students outperform peers over time (Blackwell, Trzesniewski & Dweck, 2007), and fixed beliefs about "talent" are common in music (O'Neill, 2002). Interventions have only weak effects overall (Sisk et al., 2018) and work best sustained and embedded (Dweck & Yeager, 2019). So the *design of the tool itself* carries the message: progress charts show effort leading to improvement, and encouragement tied to data ("Your Db went from 2 to 4 over three weeks") is the effort-linked feedback the research favours.

**What intrada does about gaps in practice:** intrada shows no streak; the weekly bars show how much was practised each week without counting a gap. Words that celebrate the return ("You've practised 4 of the last 7 days, which is good spacing for retention") are the intended tone, and are not built.

> **Assumption:** that streaks cause anxiety and shame-driven avoidance, particularly with ADHD, is a design hypothesis. There is general research on shame-avoidance cycles and ADHD emotional dysregulation (Barkley, 2015), but no study compares streak-based and comeback-based tracking in practice apps.

---

## 6. Retrieval Practice

Actively recalling material retains better than passive review. Wellmann & Skillicorn (2024) propose applying it to jazz education, with at least 24 hours between sessions on the same material.

**What intrada does:** it favours coming back to material across sessions over drilling it in one, which uses both spacing and retrieval.

> **Open question:** the proposal is theoretical and untested in music practice; its transfer to complex, expressive motor skills remains an assumption.

---

## 7. Neurodiversity & Music Practice

Research on ADHD and music shows both specific challenges and real strengths, and both shape intrada's design.

- **Starting and planning.** ADHD makes starting and planning hard (Barkley, 2015), and every decision before playing is a chance to stall, so the path from opening the app to playing should involve as few decisions as possible.
- **Time blindness.** ADHD disrupts the sense of time passing (Ptacek et al., 2019). Musicians with ADHD may badly misjudge how long they've been practising, lose track of time while hyperfocused on one item (neglecting others), or struggle to pace a session. This is a difference in perception rather than poor time management, and external time cues can help.
- **Hyperfocus.** A musician may spend 45 minutes on one satisfying piece while the scale work is neglected. A planned session with the time on each item visible supplies that structure from outside.
- **Frustration.** Tolerance is often lower, so encouragement and visible progress matter.
- **Music-specific findings.** ADHD brings beat-tracking and short-interval timing difficulties (Puyjarinet et al., 2017; Serrallach et al., 2022), but not in improvisation and expression (Grob et al., 2022), and ADHD behaviours are often *absent* during music-making (Wilde & Welch, 2022). Raz (2025) found that musicians with ADHD showed stronger cognitive abilities than non-musician ADHD peers, including better sustained attention and impulse control. So the practice itself may be less affected than everything around it: deciding what to practise, starting, moving between items and stopping.

**Beyond ADHD.** Sensory processing differences, dyslexia and other cognitive variations all affect how musicians use a practice tool. Less visual clutter, predictable navigation, adjustable feedback and readable type help every musician.

---

## 8. The Feedback Loop Problem & Self-Taught Learning

### What teachers actually do

Duke (2005) names four teacher functions: **diagnose**, **decompose**, **sequence** and **regulate** (monitor progress and adjust the plan). Developing musicians typically lack them (Hallam, 2001).

Bonneville-Roussy & Evans (2024) found that a teacher's support for autonomy predicted autonomous motivation, which in turn predicted both practice time and quality. The most effective tools behave like good teachers: they give structure, and leave the musician in charge.

### How self-taught practice goes wrong

Without a teacher doing those four jobs, self-taught musicians frequently show:

- **Comfort zone bias:** familiar material over weaknesses.
- **Miscalibrated difficulty:** too easy to learn from or too hard to progress on (see the 85% Rule in section 11).
- **Poor session structure:** no warm-up, too long on single items, no reflection at the end.
- **No stopping rules:** stopping at the first success, or drilling past diminishing returns.

### Motor consolidation: why progress is invisible

Motor performance improves 20 to 26% after a night's sleep with no extra practice (Walker & Stickgold, 2004), and motor memories take hours to consolidate (Brashers-Krug, Shadmehr & Bizzi, 1996). A musician who feels no progress, sleeps and comes back better credits the new session; without data, the story becomes "I'm not improving" even when they are.

> **This is intrada's core opportunity.** By tracking scores, tempo and practice patterns over time, the app can show evidence of improvement the musician cannot yet perceive.

### What intrada can take on

| Teacher function | How intrada helps |
|-----------------|-------------------|
| **Diagnose** | Later: patterns in the record show what the musician cannot self-diagnose, such as persistent weak keys, plateaus and effort that is not turning into progress |
| **Decompose** | Tracking by key, and pieces linked to the exercises that build them, break a big goal into measurable parts |
| **Sequence** | The Up next card suggests from the musician's priorities and what is going cold; today's plan (#57), then spacing and priorities (#1978), are planned, always changeable |
| **Regulate** | Scores, tempo and the session record give the outside view of progress that self-taught musicians lack |

> **Open question:** how far software can take on a teacher's regulating role is untested; intrada's own record could help answer it.

> **Assumption:** The failure modes above come from research on music students in education (typically school-age or conservatoire students). Whether adult self-directed learners, a large part of intrada's audience, show the same patterns is assumed, not validated. Adults may bring better metacognitive skills from other areas, or fail in different ways entirely.

---

## 9. The Choice Overload Problem & Guided Learning

The material a musician *could* practise is effectively infinite, and more options lead to worse decisions, or none (Schwartz, 2004).

### Choice motivates only when you know how to choose

Choice motivates only when the options fit the learner's interests, the learner can judge them, and there are few enough (Katz & Assor, 2007); its benefit shrinks as options grow (Patall, Cooper & Robinson, 2008). The guided learning literature agrees: minimal guidance is less effective than guided instruction for novice and intermediate learners (Kirschner, Sweller & Clark, 2006; Mayer, 2004), and Alfieri et al. (2011) found discovery with scaffolding and feedback beats both pure discovery and direct instruction.

### What intrada does

1. **Have a starting point ready.** The Up next card answers "what should I practise?" with one suggestion in place of a menu. Today's plan will go further: a short plan the musician can start in one tap or change (#57).
2. **Let the week narrow the choice.** The musician's priorities, and later the week's intent (#1926), decide what today's plan draws from.
3. **Show only what is needed now.** A musician starting a new exercise sees the keys they need now, not all twelve, with the next layer revealed as they master the first. This is an idea, not built.

---

## 10. Goal-Specific Pathways: From Aspiration to Curriculum

Section 9 narrows what could be practised. It leaves open the most concrete question a musician can ask: **"I want to play *this*. How do I get there?"** This is the curriculum help in the vision's Future vision, and is not scheduled.

Backward design (Wiggins & McTighe, 2005) starts from the outcome and the evidence of mastery, then designs the learning that leads there. Musical skills form prerequisite hierarchies (Gagné, 1985; Gordon, 2007) that graded exams and the Suzuki method encode, but as one generic path for every learner. A good pathway keeps each step inside the zone of proximal development (Vygotsky, 1978; Lehmann, Sloboda & Woody, 2007), and knowledge tracing (Corbett & Anderson, 1995), a Bayesian estimate of mastery per skill, could infer which prerequisites are met.

> **Assumption:** that prerequisites between musical skills can be modelled and sequenced automatically is a design hypothesis. Teachers sequence by tacit intuitions, so first pathways would likely need an expert to write them.

---

## 11. Overlearning & the Repetition Counter

Overlearning, 50 to 100% extra correct trials beyond the first success, improves retention (d = 0.753; Driskell, Willis & Copper, 1992): 3 to 5 more correct repetitions help, with diminishing returns after that.

**What intrada does:** the practice screen's repetition counter counts those extra correct repetitions.

> **Note on the 85% Rule:** Wilson et al. (2019) derived the 85% accuracy figure from machine learning models, not human motor learning. Applying it to music practice is a **creative inference** that fits Vygotsky's zone of proximal development: a rule of thumb, not a prescription.

---

## 12. Competitive Landscape

| App | Strengths | Weaknesses | Where intrada differs |
|-----|-----------|------------|----------------------|
| **Modacity** | Deliberate practice guidance, recording, mastery rating, good design | No spacing, no per-key tracking, expensive subscription, basic statistics | Tracking by key, a record that only shows what was measured |
| **Tonic** | Social and community features, gamification, practice streaks | Social first rather than practice first, no deep progress metrics, no planning help | Practice quality over social metrics, progress per item and key, everything stays on the device |
| **Piano Practice Assistant** | True spaced repetition, section-level tracking, interleaved practice, grounded in research | Android only, piano only, little design polish | Any instrument, iPhone and Android, an encouraging tone |
| **Instrumentive** | Practice habit tracking, recording, metronome, similar to Modacity | Habit building over practice quality, limited analytics | Quality as well as habit, a richer record of each item |

**Intrada's position:** quick capture, tracking by key, an honest record and design for every kind of mind, in one app for any instrument. A plan the musician stays in charge of is next (#57). Spacing and mixing items within a session (sections 1 and 2) are ideas, not built yet.

---

## References

Where a reference is secondary or unpublished, the text says so where it is cited.

Alfieri, L., Brooks, P. J., Aldrich, N. J., & Tenenbaum, H. R. (2011). Does discovery-based instruction enhance learning? *Journal of Educational Psychology*, 103(1), 1–18. https://doi.org/10.1037/a0021017

Ambrose, S. A., Bridges, M. W., DiPietro, M., Lovett, M. C., & Norman, M. K. (2010). *How Learning Works: Seven Research-Based Principles for Smart Teaching*. San Francisco: Jossey-Bass. ISBN: 978-0-470-48410-4.

Barkley, R. A. (2015). *Attention-Deficit Hyperactivity Disorder: A Handbook for Diagnosis and Treatment* (4th ed.). New York: Guilford Press.

Blackwell, L. S., Trzesniewski, K. H., & Dweck, C. S. (2007). Implicit theories of intelligence predict achievement across an adolescent transition: A longitudinal study and an intervention. *Child Development*, 78(1), 246–263. https://doi.org/10.1111/j.1467-8624.2007.00995.x

Bonneville-Roussy, A., & Evans, P. (2024). The support of autonomy, motivation, and music practice in university music students: A self-determination theory perspective. *Psychology of Music*. https://doi.org/10.1177/03057356241296109

Brashers-Krug, T., Shadmehr, R., & Bizzi, E. (1996). Consolidation in human motor learning. *Nature*, 382, 252–255. https://doi.org/10.1038/382252a0

Carter, C. E., & Grahn, J. A. (2016). Optimizing music learning: Exploring how blocked and interleaved practice schedules affect advanced performance. *Frontiers in Psychology*, 7, 1251. https://doi.org/10.3389/fpsyg.2016.01251

Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006). Distributed practice in verbal recall tasks: A review and quantitative synthesis. *Psychological Bulletin*, 132(3), 354–380. https://doi.org/10.1037/0033-2909.132.3.354

Corbett, A. T., & Anderson, J. R. (1995). Knowledge tracing: Modeling the acquisition of procedural knowledge. *User Modeling and User-Adapted Interaction*, 4(4), 253–278. https://doi.org/10.1007/BF01099821

Donovan, J. J., & Radosevich, D. J. (1999). A meta-analytic review of the distribution of practice effect: Now you see it, now you don't. *Journal of Applied Psychology*, 84(5), 795–805. https://doi.org/10.1037/0021-9010.84.5.795

Driskell, J. E., Willis, R. P., & Copper, C. (1992). Effect of overlearning on retention. *Journal of Applied Psychology*, 77(5), 615–622. https://doi.org/10.1037/0021-9010.77.5.615

Duke, R. A. (2005). *Intelligent Music Teaching: Essays on the Core Principles of Effective Instruction*. Austin, TX: Learning and Behavior Resources.

Dweck, C. S. (2006). *Mindset: The New Psychology of Success*. New York: Random House.

Dweck, C. S., & Yeager, D. S. (2019). Mindsets: A view from two eras. *Perspectives on Psychological Science*, 14(3), 481–496. https://doi.org/10.1177/1745691618804166

Ericsson, K. A., Krampe, R. T., & Tesch-Römer, C. (1993). The role of deliberate practice in the acquisition of expert performance. *Psychological Review*, 100(3), 363–406. https://doi.org/10.1037/0033-295X.100.3.363

Evans, P., & Bonneville-Roussy, A. (2016). Self-determined motivation for practice in university music students. *Psychology of Music*, 44(5), 1095–1110. https://doi.org/10.1177/0305735615610926

Gagné, R. M. (1985). *The Conditions of Learning and Theory of Instruction* (4th ed.). New York: Holt, Rinehart and Winston.

Gordon, E. E. (2007). *Learning Sequences in Music: A Contemporary Music Learning Theory*. Chicago: GIA Publications.

Grob, C. M., Biasutti, M., & Schacter, E. N. (2022). Musical improvisation and expression in individuals with ADHD. *Frontiers in Psychology*, 13, 895780.

Hallam, S. (2001). The development of metacognition in musicians: Implications for education. *British Journal of Music Education*, 18(1), 27–39.

Katz, I., & Assor, A. (2007). When choice motivates and when it does not. *Educational Psychology Review*, 19(4), 429–442. https://doi.org/10.1007/s10648-006-9027-y

Kirschner, P. A., Sweller, J., & Clark, R. E. (2006). Why minimal guidance during instruction does not work: An analysis of the failure of constructivist, discovery, problem-based, experiential, and inquiry-based teaching. *Educational Psychologist*, 41(2), 75–86. https://doi.org/10.1207/s15326985ep4102_1

Lehmann, A. C., Sloboda, J. A., & Woody, R. H. (2007). *Psychology for Musicians: Understanding and Acquiring the Skills*. Oxford: Oxford University Press.

Macnamara, B. N., Hambrick, D. Z., & Oswald, F. L. (2014). Deliberate practice and performance in music, games, sports, education, and professions: A meta-analysis. *Psychological Science*, 25(8), 1608–1618. https://doi.org/10.1177/0956797614535810

Macnamara, B. N., & Maitra, M. (2019). The role of deliberate practice in expert performance: Revisiting Ericsson, Krampe & Tesch-Römer (1993). *Royal Society Open Science*, 6(8), 190327. https://doi.org/10.1098/rsos.190327

Mathias, T., & Goldman, A. (2025). How does increasing contextual interference in a musical practice session affect acquisition and retention? *Journal of Research in Music Education*. https://doi.org/10.1177/00224294231222801

Mayer, R. E. (2004). Should there be a three-strikes rule against pure discovery learning? The case for guided methods of instruction. *American Psychologist*, 59(1), 14–19. https://doi.org/10.1037/0003-066X.59.1.14

Moss, S. L. (1995). The distribution of practice effect: A review of the literature. Unpublished review, cited in Firth et al. (2023).

O'Neill, S. A. (2002). The self-identity of young musicians. In R. A. R. MacDonald, D. J. Hargreaves, & D. Miell (Eds.), *Musical Identities* (pp. 79–96). Oxford University Press.

Patall, E. A., Cooper, H., & Robinson, J. C. (2008). The effects of choice on intrinsic motivation and related outcomes: A meta-analysis of research findings. *Psychological Bulletin*, 134(2), 270–300. https://doi.org/10.1037/0033-2909.134.2.270

Ptacek, R., Weissenberger, S., Braaten, E., Klicperova-Baker, M., Goetz, M., Raboch, J., & Stefano, G. B. (2019). Clinical implications of the perception of time in attention deficit hyperactivity disorder (ADHD): A review. *Medical Science Monitor*, 25, 3918–3924.

Puyjarinet, F., Bégel, V., Lopez, R., Dellacherie, D., & Dalla Bella, S. (2017). Children and adults with Attention-Deficit/Hyperactivity Disorder cannot move to the beat. *Scientific Reports*, 7, 11550.

Raz, S. (2025). Enhancing cognitive abilities in young adults with ADHD through instrumental music training. *Psychological Research*, 89, 9. https://doi.org/10.1007/s00426-024-02048-2

Schwartz, B. (2004). *The Paradox of Choice: Why More Is Less*. New York: Ecco/HarperCollins.

Ryan, R. M., & Deci, E. L. (2000). Self-determination theory and the facilitation of intrinsic motivation, social development, and well-being. *American Psychologist*, 55(1), 68–78. https://doi.org/10.1037/0003-066X.55.1.68

Serrallach, B., Groß, C., Christiner, M., Wildermuth, S., & Schneider, P. (2022). Musical performance in adolescents with ADHD, ADD and dyslexia — Behavioral and neurophysiological aspects. *Brain Sciences*, 12(2), 127.

Shea, J. B., & Morgan, R. L. (1979). Contextual interference effects on the acquisition, retention, and transfer of a motor skill. *Journal of Experimental Psychology: Human Learning and Memory*, 5(2), 179–187. https://doi.org/10.1037/0278-7393.5.2.179

Simmons, A. L. (2012). Distributed practice and procedural memory consolidation in musicians' skill learning. *Journal of Research in Music Education*, 59(4), 357–368. https://doi.org/10.1177/0022429411424798

Sisk, V. F., Burgoyne, A. P., Sun, J., Butler, J. L., & Macnamara, B. N. (2018). To what extent and under what circumstances are growth mind-sets important to academic achievement? Two meta-analyses. *Psychological Science*, 29(4), 549–571. https://doi.org/10.1177/0956797617739704

Stambaugh, L. A. (2009). Effects of practice schedule on wind instrument performance: A preliminary application of a motor learning principle. *Update: Applications of Research in Music Education*, 27(2), 20–28.

Stambaugh, L. A., & Demorest, S. M. (2010). Effects of practice schedule on the acquisition and retention of wind instrument skills. *Journal of Research in Music Education*, 58(4), 357–367.

Valenzuela, R., Codina, N., & Pestana, J. V. (2018). Self-determination theory applied to flow in conservatoire music practice: The roles of perceived autonomy and competence, and autonomous and controlled motivation. *Psychology of Music*, 46(1), 33–48. https://doi.org/10.1177/0305735617694502

Vygotsky, L. S. (1978). *Mind in Society: The Development of Higher Psychological Processes*. Cambridge, MA: Harvard University Press.

Walker, M. P., & Stickgold, R. (2004). Sleep-dependent learning and memory consolidation. *Neuron*, 44(1), 121–133. https://doi.org/10.1016/j.neuron.2004.08.031

Wellmann, M., & Skillicorn, A. T. (2024). Research-to-resource: Introducing retrieval practice in jazz pedagogy. *Journal of Research in Music Education*. https://doi.org/10.1177/87551233221146282

Wiggins, G., & McTighe, J. (2005). *Understanding by Design* (2nd ed.). Alexandria, VA: Association for Supervision and Curriculum Development (ASCD).

Wilde, E. M., & Welch, G. F. (2022). Attention deficit hyperactivity disorder (ADHD) and musical behaviour: The significance of context. *Psychology of Music*, 50(6), 1903–1920.

Wilson, R. C., Shenhav, A., Straccia, M., & Cohen, J. D. (2019). The Eighty Five Percent Rule for optimal learning. *Nature Communications*, 10, 4646. https://doi.org/10.1038/s41467-019-12552-4
