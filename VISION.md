# Intrada

**A practice notebook for musicians**

*Product vision, first written April 2026, revised July 2026, rewritten
September 2026. What gets built next, and in what order:
[`docs/roadmap.md`](docs/roadmap.md).*

---

## The problem

Musicians practise but don't always progress. Hours pass at the instrument, yet
the same passages stumble, the same keys feel unfamiliar, and the sense of
moving forward stalls. Three things get in the way.

**Material gets lost.** A teacher shows a new voicing, sets ii-V-I progressions
in all twelve keys, suggests a tune to transcribe. The musician scrawls it in a
notebook; a week later the note is cryptic and half the assignment is
forgotten. Over a year, dozens of exercises pile up and most of them vanish.
Musicians without a teacher lose ideas, exercises and repertoire the same way,
across apps, loose sheets and memory.

**Deciding what to practise costs time.** The musician sits down with 30
minutes. It could go on voicings, scales over changes, a new tune, that passage
from last week or something from a month ago. Without a plan made in advance,
the choice either eats into the session or ends with them playing whatever is
on the music stand.

**Progress stays invisible for weeks.** Motor skills settle between sessions
rather than during them, so improvement often shows up long after the practice
that caused it. With no evidence that practice is working, doubt fills the gap:
am I doing the right thing, or wasting my time?

---

## What intrada is

Intrada is a practice notebook: the place a musician's material lives, where
today's practice starts, and where the record of what they actually played
builds up. **The musician decides what matters; the app decides where today
starts, keeps the record honest, and can always be overruled.**

From July to August 2026 intrada was a practice coach. It chose what you
practised, held each block until you showed evidence, and gave the musician no
way round it. It was removed on 2026-08-14 (#1344). The mistake was taking the
decision away from the musician; a ready-made plan the musician can change
stays. The notebook underneath survived, and every release since has built on
it.

### Five layers

Each layer depends on the ones before it. The roadmap ranks the work itself.

1. **Capture.** Every piece, exercise, scale and pattern lives in intrada, with
   its keys, tempo, chord chart and the exercises that build it. Adding one
   takes a single pass, typed or from a photo of the page.
2. **Plan.** The musician builds a session from the library: pick items, group
   and reorder them, and set a length, or start from the starred set or today's
   plan. Practice defaults and the preferred session length are remembered.
   Routines, saved starting points for a session, are being rebuilt (#1348).
3. **Nudge.** Over the week, the musician says what matters: the items they
   have starred to improve and what the teacher set (#1926). The app adds what
   they would lose track of, the items going cold. From those it offers today's
   plan, filled to the musician's session length, so starting takes one tap
   (#57). The plan is always shown before it starts, and can be changed or set
   aside for one built by hand. More below.
4. **Show.** Progress is real but gradual, so intrada makes it visible: a score
   and a tempo for each item, key by key, recorded only when there is evidence
   behind them. The session record keeps what the musician wrote beside the
   numbers.
5. **Guide.** Later, and only once the first four are solid: patterns the
   musician can't see for themselves ("your flat keys lag your sharp ones"),
   then suggested exercises. See Future vision.

### Today's plan

The first version of today's plan draws on starred items and items going cold.
The teacher's week joins it once that exists (#1926). How the three share the
time, and whether the plan says why each item is there, is still open (#2185).

A new musician with nothing to plan from sees a Start here card instead
(#2118). That rule is reviewed after watching three new musicians use the app
(#2120).

---

## Who it's for

Musicians past the beginner stage who want their practice to work harder:

- students with a teacher, who need somewhere structured for what was set
  between lessons;
- self-taught musicians, who have no one else providing that structure;
- adults coming back to an instrument and rebuilding skills;
- anyone preparing for an exam, an audition or a performance.

Intrada works for any instrument, but is designed from a keyboard and jazz
perspective first, where exercises multiply across twelve keys and the volume
of material is hardest to manage.

**So far the only regular user is the person building it.** The next step is
to watch new musicians get from a fresh install to their first marked session
(#2121), and let what they do test everything in this section.

### Musician types: an untested idea

Musicians come with different motivations: mastery, one piece for a party, a
return after years away. An earlier version of this vision named five types
(Entertainer, Jammer, Virtuoso, Soul Player, Late Starter) to shape onboarding
and tone. None is built, and none will be until real musicians show the
difference matters. The idea behind it stands: different motivations can want a
different experience. Its basis in self-determination theory is in the
[research foundation](docs/research-foundation.md#4-self-determination-theory-sdt).

---

## Principles

### Progress is the product

What intrada gives a musician is the sense of getting better. Every feature
either helps them progress or makes the progress visible; a feature that does
neither doesn't belong.

### The musician has the last word

The musician decides what matters. The app may decide where today starts, but
always shows its plan and accepts any change to it. A suggestion can be
dismissed, a default can be changed, and nothing waits on the app's judgement.
The practice coach broke this rule.

### Keep it light

Logging a session should take under 30 seconds. The app stays out of the way during
practice and does its work before and after.

### A short path to start

Every decision between "I want to practise" and "I am practising" is a chance
for the session not to happen, especially for musicians who find starting hard.
Today's plan makes it one tap (#57). Remembered defaults and the starred set
keep building your own session quick, and routines will join them (#1348).

### Reflection closes the loop

Practice without reflection is repetition. At the end of a session each item
offers a short note, always skippable, where the musician says what improved
and what still breaks. The notes stay in the session record; showing them on
the item comes with the week's intent (#1926).

### The record tells the truth

A number shown as a measurement is one. Tempo, score and time are recorded when
there is evidence for them and left empty otherwise, so a chart never draws a
default as if it had been played.

### Celebrate the comeback

Intrada never shows a broken streak or a zero. After a break, what went cold is
shown as information to act on.

### No journey is linear

Musicians plateau, change direction, take breaks and restart. A break is part
of practising, and coming back after six months picks up where you left off.

### Designed for every mind

Neurodivergent musicians shape every screen: a short path to start, time made
visible, comebacks celebrated, sessions of any length. Specific commitments:

- no sounds that play by themselves;
- a calm palette;
- text that stays readable at every size;
- navigation that stays where it was, version to version;
- feedback whose frequency and tone the musician can adjust.

---

## How practice material works

**Pieces and exercises.** Pieces are repertoire, with an optional chord chart.
Exercises are scales, arpeggios, patterns and technical studies.

**Keys.** An exercise practised in several keys tracks each key on its own, so
the weak ones stand out instead of being averaged away.

**Variations.** A variation belongs to its exercise, and a session records the
ones you played (#1739, [spec](specs/exercise-variations.md)). The remaining
wording and tidy-ups are in #1970.

**Building a piece.** A jazz standard collects the exercises that build it: the
melody, shell voicings in each inversion, scales over every change. The piece
links to each exercise and each exercise back to the piece; each is tracked on
its own, and they are practised together.

**Routines.** A saved run of items, such as a warm-up or a technical block,
that starts or joins a session so the musician doesn't rebuild it every day.
The first version was removed (#1747); the rebuild is planned under #1974
(#1348).

---

## The week's intent

A musician usually knows what this week is for: the tune for Thursday's gig,
the voicing the teacher set. Intrada will hold that as **one sentence on an
item**, dated and labelled with where it came from ("From Tuesday's lesson",
"Mine"). The current sentence shows while the item plays and fills in its aim
in the session builder; earlier sentences stay on the item as its history, and
the dates say which week each belongs to (#1926). There are no separate lessons
or goals to manage.

This replaces a separate goals feature, planned in July 2026 and never built.
Goals were built and removed twice before (#213, #769), and both times they
turned into a chore of their own. The star on each item stays as the simplest
way to say "this matters".

---

## Future vision

Each of these depends on the foundation above, and none is scheduled:

- **Patterns and gaps.** Weak keys, neglected material and plateaus, found in
  the musician's own record.
- **Teacher sharing.** A teacher suggests items, sets target tempos and shares
  routines, with the student's permission.
- **Audio recording.** Record a run-through and compare it with one from weeks
  ago.
- **Curriculum help.** Turning "I want to play Clair de lune" into a sequence of
  exercises, measured against the musician's record.
- **Checking it's right.** Answering "am I doing this right?": whether a
  voicing or a scale fits the harmony. It needs musical knowledge or a teacher,
  and it is the hardest problem here.
- **Plans that learn.** Today's plan learning from how the musician changes it,
  and noticing a plateau.

The app listening to whether a passage was right is on hold; a first
experiment's findings are in
[`docs/segmentation-findings.md`](docs/segmentation-findings.md). If listening
returns, input from a MIDI keyboard comes first.

---

## Research

The design draws on learning science, the psychology of motivation and music
education research: spacing practice out, mixing items within a session,
focused practice, what keeps people motivated, and what too much choice does.
The studies, and how strong the evidence is for each, are in the
[research foundation](docs/research-foundation.md).

---

## Other practice apps

Most practice apps either count minutes, which measures turning up rather than
getting better, pile on features that feel like work, or put social features
ahead of the practice itself. Intrada combines quick capture, tracking by key,
an honest record and design for every kind of mind, in a clean app for any
instrument that works entirely on the phone. The comparison app by app is in
the [research foundation](docs/research-foundation.md#12-competitive-landscape).

---

*This vision changes when the product teaches us something.*
