# Where we are

*What phase the work is in. Written by hand and changed only when the phase
changes, so no two branches edit it at once. What is being built right now:
`just status`, which reads GitHub. What comes next:
[`roadmap.md`](roadmap.md).*

**Which release is out on TestFlight: `gh release list`.** This file names no
current build, because a release does not change it and the name would go stale
with the next tag (#1932). Each release's write-up is its
[GitHub release](https://github.com/jonyardley/intrada/releases), from v0.12.0
on; v0.10.0 and v0.11.0 are tags without one.

**The phase now is the first run: get a new musician to their first marked
session (#2121), then watch three of them do it (#2120).** It follows the
vision rewrite of 2026-09-30 ([`VISION.md`](../VISION.md)); the order of
everything after it is in [the roadmap's ranking](roadmap.md#the-ranking).

**The capture phase before it is done.** It was chosen on 2026-09-07, at the
end of the rethink after the coach was removed
([`rethink-plan.md`](rethink-plan.md)), and shipped as:

- **v0.10.0** (2026-09-09): add a piece with its chord chart and exercises in
  one pass (#1390), plus the practice screen's session timer, repetition
  counter and click, and the first build that reports its crashes.
- **v0.11.0** (2026-09-11): the paper and marker look.
- **v0.12.0** (2026-09-12): the app knows who is practising, with a name, an
  instrument and a highlighter colour.
- **v0.13.0** (2026-09-16): variations belong to the exercise, and a session
  records what you actually played.
- **v0.14.0** (2026-09-24): the September audit's fixes for lost data and wrong
  numbers (#1967).

**Known gaps a tester will hit:** a crash report names the build but not the
line of code, because debug symbols are not uploaded (#1610), and several
screens break at the largest text sizes (#2134).
