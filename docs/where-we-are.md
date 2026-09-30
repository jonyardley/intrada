# Where we are

*Orientation, hand-written, changed when the phase changes and not otherwise,
so no two branches ever edit it at once. A release does not change it: its
write-up is its GitHub release, generated from that release's milestone
([`roadmap.md`](roadmap.md)). For what is in flight
right now, run `just status`; it reads GitHub, which is the source of truth.
Direction and phases: [`roadmap.md`](roadmap.md).*

**v0.14.0, "Nothing goes missing, and the screen tells the truth", is the
latest release (2026-09-24, TestFlight build 23).** Its write-up, and those of
v0.12.0 and v0.13.0, are their
[GitHub releases](https://github.com/jonyardley/intrada/releases). v0.10.0 and
v0.11.0 are tags without a release write-up.

**The phase now is first run: get a new musician to their first marked
session (#2121), then watch three of them do it (#2120).** Agreed on
2026-09-30 with the vision rewrite ([`VISION.md`](../VISION.md) v3): intrada
is a practice notebook in which the musician plans and the app remembers,
keeps the record honest and nudges. The ranking of everything after it is in
[`roadmap.md`](roadmap.md#the-ranking); the week's intent (#1926), which
replaces the goals feature, is next.

**The capture phase before it is done.** Chosen on 2026-09-07 at the end of the
post-revert rethink ([`rethink-plan.md`](rethink-plan.md)), it shipped as:

- **v0.10.0** (2026-09-09): add a piece with its chord chart and exercises in
  one pass (#1390), the Focus Player's session timer, repetition counter and
  click, and the first build that reports its crashes.
- **v0.11.0** (2026-09-11): the paper and marker look.
- **v0.12.0** (2026-09-16): the app knows who is practising, with a name,
  an instrument and a highlighter colour.
- **v0.13.0** (2026-09-16): variations belong to the exercise, and a session
  records what you actually played.
- **v0.14.0** (2026-09-24): the September audit's data-loss and
  wrong-number fixes (#1967).

Since v0.14.0, practice defaults and the preferred session length landed on the
Profile screen and the builder (#1915, #1736), not yet released.

**Known gaps a tester will hit:** a crash names the build but not the line,
because debug symbols are not uploaded (#1610), and several screens break at
the largest text sizes (#2134).
