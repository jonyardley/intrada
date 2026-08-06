---
name: plan-slice
description: Use when Jon asks to plan the next slice of work, pick what to build next, or asks for the opener to paste into a terminal
---

# plan-slice

Pick the next slice and hand back a paste-ready opener for **one** session.

A vertical slice is one agent's job (CLAUDE.md, Parallel work streams). Do not
propose splitting core and iOS across two agents on the same slice; agent teams
were tried on #1223 and retired because a bridge contract couples the halves
tighter than any handoff protocol compensates for.

## Steps

1. `gh issue list -R jonyardley/intrada --label horizon:now` (plus the project
   board if that's thin). Candidates = issues forming ONE slice; sequencing
   beats bundling when issues block each other.
2. Size it for one session. If the slice is too big to hold, cut it at a
   natural seam and hand back two sequential steps, not two parallel ones.
3. Tier check: auth, DB schema, or FFI-contract redesign is Tier 3. Cite the
   spec section that already covers it, or say a spec is needed first — never
   hand back an opener for unspecced Tier 3 work.
4. **Fan-out check.** Suggest worktree fan-out only when the slice contains
   genuinely independent pieces (an audit sweep, a migration across many files,
   N approaches to one design question), and say what each agent owns. Anything
   coupled by a contract stays solo. When in doubt, solo.
5. Name the checkout. A fresh worktree if another session is live in the main
   checkout; say which path you chose.

## Output (exactly this shape)

Rationale first, then a step list. The commands ARE the deliverable — never
bury one mid-paragraph (Jon's feedback, 2026-08-05).

1. **Orientation** (3-5 lines, prose): phase, what just landed, one line of
   rationale per chosen issue, one line per rejected near-miss, and the
   fan-out call from step 4 if it applies.
2. **Step list** — one numbered step per session to start. Each step carries,
   in this order:
   - Bold title: what it ships, with issue numbers.
   - A `Model:` line — model + effort, and **where**: new vs existing session,
     which checkout or worktree.
   - TWO fenced blocks, never one combined command (Jon manages workspaces with
     his own tooling and launches Claude bare, then pastes the prompt inside):

     *Shell* — workspace setup ending in a bare `claude`:

     ```bash
     cd ~/Dev/intrada && git pull
     claude
     ```

     *Inside Claude, paste* — the opener prompt only:

     ```text
     <opener>
     ```
   - Steps that are Jon's own (hardware, practising) go last, no command.
3. Nothing between or after the command blocks except the next step —
   explanation lives in the orientation, above.
