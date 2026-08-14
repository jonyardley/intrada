# Status — what's in flight now

*One screen: current phase, what's in flight, what just landed, what's next.
Updated in every PR that changes scope or state (see CLAUDE.md → After
completing work). Direction and phases live in [`roadmap.md`](roadmap.md); the
scope and timing detail on the
[project board](https://github.com/users/jonyardley/projects/2). When this
doc and the issues disagree, the issues are right. For the last-updated date,
ask git: `git log -1 --format=%cs docs/status.md`.*

## Where we are

**The coach pivot is reversed (2026-08-13, #1344).** The practice-coach
direction built through Phase 2b went too far too fast; the decision was to
restore the session-builder product and remove the coach. The builder as it
stood at 9e92ab2 is back on today's toolchain (Rust 1.97.1, crux 0.20,
current CI): build a setlist from the library, group and reorder blocks, edit
reps and duration per entry, play through the Focus Player, reflect on the
summary. All coach machinery (engine, drill loop, mastery track, built
sessions, click) is removed; it stays recoverable from commit 071b85b, and
the design record keeps its bannered specs.

What replaces the coach as product direction is an open question. The
rethink follows this revert, it did not precede it.

## In flight

Nothing in flight. The product rethink (see Next) has not started.

## Recently landed

- #1344 — the session-builder revert, merged 2026-08-13 (commit 9b8da6d).

## Next

- The product rethink is now planned: see [`rethink-plan.md`](rethink-plan.md)
  (agreed 2026-08-14). First up is Stage 1, the post-revert audit. Parked
  builder-era issues (#1087, #1101, #1107) are eligible again but stay parked
  until Stage 4 chooses a direction.
- Follow-up: port the wire-pin test technique (per-variant bincode
  fingerprint) to the `ActiveSession` crash-recovery blob.
