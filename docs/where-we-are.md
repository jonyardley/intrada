# Where we are

*Orientation, hand-written, changed when the phase or the release changes and
not otherwise — so no two branches ever edit it at once. For what is in flight
right now, run `just status`; it reads GitHub, which is the source of truth.
Direction and phases: [`roadmap.md`](roadmap.md).*

**v0.9.0 is on TestFlight (2026-09-02).** Its headline is adding a piece from
a photo: photograph the page and the add form fills itself, with the on-device
model picking the fields where Apple Intelligence is available (phases A to C
of [`specs/piece-from-photo.md`](../specs/piece-from-photo.md); phase D, a
chord chart from a photo, is not started). Alongside it: the metronome click
and the tempo trend, the "Used in" card on exercises, and a run of Library
sorting and accessibility fixes.

Phase R ([`rethink-plan.md`](rethink-plan.md)) is in Stage 3, working the
audit backlog in [`audit-2026-08.md`](audit-2026-08.md) — the definitive
reference for what the audit found and the order it runs in. Phases 1 and 2
are closed; Phase 3 has the Session Complete notes (#1370) and the history
detail view (#1371) outstanding; Phase 4 has shipped the metronome and the
photo, leaving the pass counter.

**Next up is one round covering the rest of the Focus Player**: the session
timer (#1364), the pass counter (#1367), and clicking on chosen beats without
lying about the tempo (#1499). One Claude Design pass, one Tier 3 spec
(`specs/practice-instruments.md`), separate implementation PRs.

The next major direction stays open (Stage 4 of the rethink plan). The capture
line has earned its keep — the on-device page read was validated on device on
2026-09-02 — but nothing is decided between pushing capture further and the
weekly-lesson loop (#1087).
