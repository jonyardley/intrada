---
name: plan-team
description: Use when Jon asks to plan the next vertical slice, pick work for an agent team, or asks for the team command to paste into a terminal
---

# plan-team

Plan one core+iOS vertical slice and hand back a paste-ready team command.
`/team-vertical` (the command in the output) runs *inside* the terminal
session and starts the team; this skill only plans and hands over.

## Steps

1. `gh issue list -R jonyardley/intrada --label horizon:now` (plus the
   project board if that's thin). Candidates = issues forming ONE vertical
   slice; sequencing beats bundling when issues block each other.
2. Decompose by ownership: core teammate owns `crates/intrada-core`, ios
   teammate owns `ios/Intrada` + `ios/IntradaTests`. An issue that needs
   both teammates writing one serialisation-point file (CLAUDE.md, Parallel
   work streams) gets queued, not bundled.
3. Tier check: auth, DB schema, or FFI-contract redesign is Tier 3. Cite the
   spec section that already covers it, or say a spec is needed first —
   never hand back a command for unspecced Tier 3 work.
4. A core-only or ios-only slice is fine — say the team collapses to one
   builder plus a reviewer rather than forcing a fake split.
5. **Split-ratio check** (#1199): estimate the core/iOS work ratio and the
   dependency direction for the slice as decomposed in step 2. Below roughly
   70/30 either way, or when one side is entirely blocked until the other's
   contract lands, recommend a solo session with test-runner subagents
   instead of a team command — a team's overhead (contract handoff, streamed
   review, idle-teammate drafting) only pays for itself when both sides have
   enough independent work to run in parallel.

## Output (exactly this shape)

1. One line of rationale per chosen issue; one line per rejected near-miss.
2. The split-ratio call from step 5: team or solo, one line why.
3. The command, as a runnable block:

   ```bash
   cd ~/Dev/intrada && git pull && claude "/team-vertical <issue numbers>"
   ```

   Substitute a fresh worktree for `~/Dev/intrada` if another session is
   live in the main checkout, and say which path you chose. If step 5 called
   for solo instead, give the equivalent single-session opener with
   test-runner subagents rather than this command.
4. Close with the model line: lead on Opus/Fable high; teammates come from
   the terminal session's `/config` default (Sonnet). For a solo call, this
   is just the one session's model.
