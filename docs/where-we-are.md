# Where we are

*Which release is out and what phase the work is in. Written by hand and
changed only when the phase changes, so no two branches edit it at once. What
is being built right now: `just status`, which reads GitHub. What comes next:
[`roadmap.md`](roadmap.md).*

**The latest release is v0.15.0, "Sit down and play" (2026-10-03).** Each
release's write-up is its [GitHub release](https://github.com/jonyardley/intrada/releases), from v0.12.0
on; v0.10.0 and v0.11.0 are tags without one.

**The phase now is finishing the data model (#50), chosen on 2026-10-03:**
sections, variations shared across the library, and exercises linked to
sections, so later features build on a settled shape. Alongside it, three new
musicians try the first run (#2120). The order of everything after it is in
[the roadmap's ranking](roadmap.md#the-ranking).

**The first-run phase before it shipped in v0.15.0** (#2121): a welcome, a
profile step, a first piece, and a Start here card on Practice.

**The capture phase before that is done.** It was chosen on 2026-09-07, at the
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
