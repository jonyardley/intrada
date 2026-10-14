# Research note: the Space layer (spaced repetition and mastery decay)

*Why getting cold is a quiet, graded estimate. Kept for sections 4 and 6, which
`staleness.rs` and #1419 cite; delete when #1419 closes. 2026-08-14.*

## 4. Pedagogy evidence

The core literature is in [`research-foundation.md`](../research-foundation.md)
§1, §2 and §6.

**Spacing is robust in general, moderated for motor skills.** Donovan &
Radosevich (1999; 63 studies) found spaced practice beats massed with d=0.46,
but the effect shrinks as task complexity rises. Cepeda et al. (2006) found
optimal gaps grow with the desired retention interval. Continuous skills are
retained far better over long gaps than discrete ones (Adams, 1987; Lee &
Genovese, 1988). Music mixes both kinds, which cautions against one curve for
the whole library.

**Across-session returns have direct music evidence.** Simmons (2012) showed
non-pianists gained accuracy on a keyboard sequence only when sessions were 24
hours apart, though spacing effects are not always demonstrable for complex
motor sequences.

**The algorithm transfer is the weak link.** SM-2 was designed for verbal
flashcards, and no published research validates its parameters for motor
skills. FSRS (the Anki default) is validated on flashcard logs and needs about
a thousand reviews before fitting beats its defaults; Duolingo's half-life
regression (Settles & Meeder, 2016) used 13 million traces. Both assume graded
recall of a discrete fact and data volumes a solo musician will never produce.
Nothing validates any of these schedulers for repertoire.

**Prior art applying spaced repetition to repertoire is small.** Piano Practice
Assistant schedules at the subsection level with user overrides, and its
developer calls the approach "not particularly well-studied". Musicians using
Anki report two lessons: a piece is not a card, and opaque intervals get
abandoned.

## 6. Risks and open questions

- **The coach-pivot failure mode.** #1344 reversed a product where the app
  decided what you practised. The app **surfaces evidence and suggests; it
  never prescribes, gates, or auto-builds**. "Cold for three weeks, last
  scored 6" is a fact the user reads; "you must review this today" is the
  coach again. Every suggestion is one dismissible tap.
- **Borrowed authority.** A due date or retention percentage would be a
  heuristic wearing a lab coat. Copy presents staleness, not a claim about the
  user's memory: "getting cold" is honest where "72% retained" would not be.
- **Data sparsity breaks fitted models.** One user, irregular practice,
  optional ratings. A fixed, visible heuristic with manual override is the
  only defensible first version.
- **Self-report noise.** A score logged minutes after ten repetitions measures
  performance, not retention; weight scores from returns over scores from
  drilling.
- **Wrong grain.** Prior art says sections; pieces have only whole items until
  #50.
- **Error asymmetry.** Too-eager resurfacing nags and erodes trust (crying
  wolf); too-lax is silent rot. Consolidated repertoire decays slowly, so a
  flashcard-shaped curve over-alarms by default. Bias lax, let the user
  tighten.
- **Cold start.** An item with no history says "no data" rather than inventing
  a curve.

## 7. Sources

- Cepeda et al. (2006), distributed practice meta-analysis: https://doi.org/10.1037/0033-2909.132.3.354
- Donovan & Radosevich (1999), spacing × task complexity meta-analysis (via research-foundation §1)
- Lee & Genovese (1988), distribution of practice, discrete vs continuous: https://pubmed.ncbi.nlm.nih.gov/2489826/
- Simmons (2012), distributed practice and consolidation in musicians: https://doi.org/10.1177/0022429411424798
- Settles & Meeder (2016), half-life regression at Duolingo: https://research.duolingo.com/papers/settles.acl16.pdf
- FSRS scheduler (open-spaced-repetition, DSR model): https://github.com/open-spaced-repetition/free-spaced-repetition-scheduler
- Piano Practice Assistant, spaced repetition for musicians: http://pianopracticeassistant.com/spaced-repetition/
- Malcolm Sailor, Anki for jazz practice: https://malcolmsailor.com/2023/11/22/anki-for-jazz-practice.html
- Piano World forum, piano practice with SRS (Anki): https://forum.pianoworld.com/ubbthreads.php/topics/2012638/all/piano-practice-with-spaced-repetition-systems-anki.html
