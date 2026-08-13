# Session builder revert — restore the builder, remove the coach

> Status: implemented by the PR this spec rides with (#1344). Decision date
> 2026-08-13.

## Problem

The 2026-07 practice-coach pivot went too far too fast in a direction Jon was
never sure about, and on reflection does not want. The coach was built through
Phase 2b (v0.6.0): a five-stage planner, evidence-gated drill loop, mastery
track, built sessions with three altitudes, and qualitative capture. All of it
is now unwanted product direction sitting on top of a session-builder flow
that was deleted at Phase 2a close-out (2ae89f4).

## Target state

The pre-deletion session builder restored — build a setlist from the library,
group and reorder blocks, edit reps/duration/intention per entry, play through
the Focus Player, reflect on the summary screen — with **every piece of coach
machinery removed**, running on today's infra (Rust 1.97.1, crux_core 0.20,
current CI, current deps).

Two facts shape the method:

1. **The restore point is `9e92ab2`** (= `2ae89f4^`), named by the deletion
   commit's own message as the recovery point. But the coach engine *already
   exists there* — builder and coach coexisted between 2a and the deletion. So
   the target state never existed as one commit: this is restore **plus**
   excision.
2. **Restore wholesale, then delete** beats hand-reconciling forward: every
   post-deletion change to the restored files was coach work (verified per
   file, per commit), so the 9e92ab2 versions are the cheapest correct base.

## Keep / kill

| Surface | Call |
|---|---|
| Builder core (`BuildingSession`, builder `SessionEvent`/`SetEvent` vocabulary, `BuildingSetlistView`) | Restore from 9e92ab2 |
| Builder screens (`SessionBuilderScreen`, add/settings/related sheets, UITests, 8 snapshot references) | Restore from 9e92ab2 |
| `http.rs`, `analytics.rs` | **Keep HEAD** — zero coach content; restoring would silently revert the #1274 rejection-message feature and the 1.97.1 tie-order determinism fix |
| Coach engine (`crates/intrada-core/src/engine/`), `domain/built_session/` | Delete |
| Coach iOS (`ios/Intrada/Coach/`, `DesignSystem/Coach/`, drill/compose/feel/reflection/soft-landing screens, `PressStartHero`, `SessionOverview`, `ProposedSteerCard`, coach tests + snapshot references) | Delete |
| ClickEngine (metronome) | Delete (decision: recover from history if a metronome returns) |
| GRDB migrations v10–v15 (coach tables) | **Stay registered** — shipped migrations are never deleted; devices that ran them fail `DatabaseMigrator` otherwise. Dead tables are the price of the append-only chain |
| Coach persistence *ops* (`SaveCoachRecords`, built-session ops) and their Swift handling | Delete |
| Coach crash-recovery blob (`SaveCoachSessionInProgress`, `EngineSession`) | Delete; the builder-era `SaveSessionInProgress`/`ActiveSession` recovery returns |
| crux 0.19→0.20 | No API adaptation needed (verified); re-apply 4 one-line clippy fixes (3× `is_none_or` in `app.rs`, 1× `is_multiple_of` in `session.rs`); drop the now-unused `toml` dep from `intrada-core` |
| Live-bridge rejection tests (2 funcs in `StoreEffectLoopTests`) | Re-add verbatim from HEAD — only real-wire coverage of the kept `http.rs` feature |
| `content/` (paper-teacher material) | **Keep** — Phase 0 practice material usable without code; Jon's call whether it stays long-term |
| Coach design/engine specs, `rebuild-review.md`, `segmentation-findings.md` | Keep with retirement banners — the record of why |
| `specs/built-session*`, coach briefs, coach wireframes, `Drill Loop.dc.html`, `coach-orientation.html`, `coach-2a-slice-contract.md` | Delete — transient artefacts, recoverable from history |

## Key decisions

- **One atomic PR.** The bridge contract swaps both sides at once; a core-first
  PR cannot leave the shell compiling. Justified deviation from the two-PR
  rule: the content is deletions plus previously-reviewed historical code.
- **Marketing version stays 0.6.0.** Version numbers move forward regardless
  of direction.
- **The wire-pin test technique** (per-variant bincode fingerprint of the
  crash-recovery blob, born of #1223/#1244/#1291) dies with `engine/session.rs`
  where it lives; porting it to the restored `ActiveSession` blob is tracked as
  a follow-up issue rather than done in this PR.
- **Parked builder-era issues** (`superseded-by-pivot`: #1087, #1101, #1107)
  become eligible again but are NOT un-parked here — the product rethink after
  this PR decides direction first.

## Recovery

All removed coach code is recoverable from `071b85b` (the pre-revert HEAD).
The coach's design record stays in `specs/intrada-practice-coach-design.md`
and `specs/intrada-coach-engine.md` (bannered); the pivot rationale in
`docs/rebuild-review.md` (bannered).
