# Intrada

**A practice notebook for musicians**

*Product vision. What gets built next, and in what order:
[`docs/roadmap.md`](docs/roadmap.md).*

---

## The problem

Musicians practise but don't always progress. Three things get in the way.

**Material gets lost.** A teacher sets ii-V-I progressions in all twelve keys;
a week later the note is cryptic and half the assignment is forgotten. Without
a teacher, ideas and repertoire scatter across apps, loose sheets and memory.

**Deciding what to practise costs time.** Without a plan made in advance, the
choice eats into the session or ends with whatever is on the music stand.

**Progress stays invisible for weeks.** Motor skills settle between sessions
rather than during them, so improvement often shows up long after the practice
that caused it. With no evidence that practice is working, doubt fills the gap.

---

## What intrada is

Intrada is a practice notebook: the place a musician's material lives, where
today's practice starts, and where the record of what they actually played
builds up. **The musician decides what matters; the app decides where today
starts, keeps the record honest, and can always be overruled.**

A practice coach that chose for the musician was removed on 2026-08-14 (#1344)
because it took the decision away. A ready-made plan stays, as long as the
musician can change it.

### Five layers

Each layer depends on the ones before it. The roadmap ranks the work itself.

1. **Capture.** Every piece, exercise, scale and pattern lives in intrada, with
   its keys, tempo, chord chart and the exercises that build it, added in one
   pass, typed or from a photo of the page.
2. **Plan.** The musician builds a session from the library: pick items, group
   and reorder them, and set a length. Defaults are remembered.
3. **Nudge.** Over the week, the musician says what matters: their priorities
   and what the teacher set (#1926). The app adds what they would lose track
   of, the items going cold. Today the Up next card suggests a piece from
   these; next comes today's plan (#57).
4. **Show.** A score for each key and a tempo for each item, recorded only
   with evidence behind them, beside what the musician wrote.
5. **Guide.** Later, and only once the first four are solid: patterns the
   musician can't see for themselves ("your flat keys lag your sharp ones").

### Today's plan (next)

Today's plan, filled to the musician's session length, will always be shown
before it starts, and can be changed or set aside. Its first version will draw
on priorities and items going cold; the teacher's week joins once that exists
(#1926). How the three share the time, and whether the plan says why each item
is there, is still open (#2185).

A new musician with nothing to plan from will see a Start here card instead
(#2118, reviewed in #2120).

---

## Who it's for

Musicians past the beginner stage who want their practice to work harder:
students with a teacher, self-taught musicians, adults coming back to an
instrument, and anyone preparing for an exam, an audition or a performance.

Intrada works for any instrument, but is designed from a keyboard and jazz
perspective first, where exercises multiply across twelve keys.

**So far the only regular user is the person building it.** The next step is
to watch new musicians get from a fresh install to their first marked session
(#2121).

### Musician types: an untested idea

Different motivations may want a different experience. No musician types are
built until real musicians show the difference matters. The basis is in the
[research foundation](docs/research-foundation.md#4-self-determination-theory-sdt).

---

## Principles

### Progress is the product

Every feature either helps a musician progress or makes the progress visible; a
feature that does neither doesn't belong.

### The musician has the last word

The app may decide where today starts, but always shows its plan and accepts
any change to it. Nothing waits on the app's judgement.

### Keep it light

Logging a session should take under 30 seconds. The app stays out of the way
during practice.

### A short path to start

Every decision between "I want to practise" and "I am practising" is a chance
for the session not to happen, especially for musicians who find starting
hard, so starting should take one tap (#57).

### Reflection closes the loop

At the end of a session each item offers a short note, always skippable, where
the musician says what improved and what still breaks. The notes stay in the
session record; showing them on the item comes with the week's intent (#1926).

### The record tells the truth

Every number intrada shows was measured. Tempo, score and time are recorded when
there is evidence for them and left empty otherwise, so a chart never draws a
default as if it had been played.

### Celebrate the comeback

Intrada never shows a broken streak or a zero. After a break, what went cold is
shown as information to act on.

### No journey is linear

Musicians plateau, change direction, take breaks and restart. A break is part
of practising, and coming back after six months picks up where the musician
left off.

### Designed for every mind

Every screen is designed with neurodivergent musicians in mind:

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
ones played (#1739, [spec](specs/exercise-variations.md)).

**Building a piece.** A jazz standard links both ways to the exercises that
build it (melody, shell voicings, scales over the changes); each is tracked on
its own and practised together.

**Routines.** A saved run of items, such as a warm-up, that starts or joins a
session. The first version was removed (#1747); the rebuild is #1348.

---

## The week's intent

A musician usually knows what this week is for. Intrada will hold that as
**one sentence on an item**, dated and labelled with where it came from ("From
Tuesday's lesson", "Mine"), shown while the item plays; earlier sentences stay
as its history (#1926). There are no separate lessons or goals to manage: two
goals features were built and removed (#213, #769). Marking an item a priority
stays the simplest way to say "this matters".

---

## Future vision

Each of these depends on the foundation above, and none is scheduled:

- **Patterns and gaps.** Weak keys, neglected material and plateaus, found in
  the musician's own record.
- **Teacher sharing.** A teacher suggests items and target tempos, with the
  student's permission.
- **Audio recording.** Compare a run-through with one from weeks ago.
- **Curriculum help.** Turning "I want to play Clair de lune" into a sequence of
  exercises.
- **Checking it's right.** Whether a voicing fits the harmony.
- **Plans that learn.** Today's plan learning from how the musician changes it.

The app listening to whether a passage was right is on hold
([`docs/decisions.md`](docs/decisions.md)); if it returns, MIDI input comes
first.

---

## Research

The design draws on learning science, motivation psychology and music
education research. The studies, and how strong the evidence is for each, are
in the [research foundation](docs/research-foundation.md).

---

## Other practice apps

Most practice apps count minutes, pile on features that feel like work, or put
social features ahead of the practice. Intrada combines quick capture,
tracking by key, an honest record and design for every kind of mind, in one app
for any instrument that works entirely on the phone. The comparison app by app
is in the [research foundation](docs/research-foundation.md#12-competitive-landscape).
