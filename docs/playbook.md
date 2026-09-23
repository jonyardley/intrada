# The playbook

> One page: what Jon does at the keyboard to get a unit of work shipped at
> good quality for the least spend. The rules and their evidence live in
> [`working-with-agents.md`](working-with-agents.md); this page is the
> reminder. Set on 2026-09-17 (#1989), when the behaviour coaching was
> unhooked and the guards kept.

## The shape of a unit

One issue, one session, one worktree, one PR, then `/clear`. The session does
the mechanics; you pick, approve, merge and clear.

| Step | You | The session | Minutes of yours |
|---|---|---|---|
| 1. Pick | `just status` in the main checkout; choose the epic's next open child | | 1 |
| 2. Open | A new session in the main checkout; type the issue number and one sentence of intent | Makes its worktree, claims the issue, reads it and its epic | 1 |
| 3. Plan | Tier 1: nothing. Tier 2 and 3: read the plan comment on the issue, reply "go" or correct once | Posts about 150 words: done looks like, decisions, files, tests, out of scope | 3 |
| 4. Build | Nothing, or answer the one question it asks. A screen change: look at the screenshot it shows you | Test-first on the core; gates in `test-runner`; shows the screen before the PR | 0 |
| 5. Ship | Read the PR body and the self-review comment; merge when green and mergeable | `/ship`: gates, `reviewer`, a draft PR marked ready after the self-review, CI watched to a conclusion | 5 |
| 6. Close | `/clear`, or `just handover N` and paste what it prints into a fresh session | Closes the issue, moves the board, removes the worktree | 1 |

Two corrections on one point stop the change: ask whether the approach is
wrong before a third (#1890). A session opened only to ask "what's next" pays
a cold start for an answer `just status` already prints (#1836).

## Which model

The rung is not where the money is; context length is (#1985). Set the model
once, at the start, and leave effort alone off the sensitive surfaces.

| Work | Model | Who sets it |
|---|---|---|
| The lead: everything with judgement in it | Opus 5.5 medium | `.claude/settings.json`, on open |
| The sensitive surfaces: the bridge, a migration, the crash-recovery blob, auth | Opus 5.5 high (#2044, #2052) | You: `/effort high` before the first line, and `/effort medium` back when that work ends |
| `reviewer` | Opus 5.5 high, sensitive diffs included | Pinned in the agent definition |
| `task` | Opus 5.5 medium until 2026-10-07 (#2044) | Pinned in the agent definition |
| `test-runner` | Haiku 4.5 low | Pinned in the agent definition |
| A one-off thinking question | Your call | You |

## Which tool, when

| You want | Use |
|---|---|
| To know what is next | `just status` in the terminal, not a session |
| To start a unit | A new session, the issue number, one sentence |
| To carry a unit to the next session | `just handover N`, then paste what it prints |
| To ship | `/ship` in the session |
| To see a screen | `just ios-run` with seeded data; `SEED=0 just ios-run` to test persistence |
| To design a screen | Claude Design, project "Intrada"; the session pulls the output by id |
| An architecture answer | The graphify graph in the main checkout, never a full-repo read |
| To know what the week cost | `just usage 7`; add `--quality` for corrections, rung switches and long subagent runs |
| Gate logs kept out of the lead | The session's `test-runner`; nobody types `xcodebuild` |
| A big file looked at | Ask for the symbol and a line range, not the file |

## Spend: the four rules the numbers support

1. **Context length is the bill.** One unit per session, `/clear` when it
   ships. The status line goes amber at 200k.
2. **A cold turn costs over a dollar**: $233 for 194 of them in the week to
   2026-09-13, $199 for 123 in the fortnight to 2026-09-17, because the cache
   is rewritten after an hour idle. Finish or clear before a break; never leave
   a session open over lunch.
3. **Repeat reads were half of all reads** in the 11 to 14 September baseline.
   A second look at a file is grep, then a range.
4. **Subagents were 43% of spend.** `task` takes a settled slice under ten
   files; `reviewer` runs once per PR; a fan-out, a Workflow or a full-repo
   scan is flagged with its cost before it runs, and only when you asked.

## What a refusal means

The guards stay on. When the session reports one, this is what happened and
what you do.

| The session says | What happened | You |
|---|---|---|
| Edit denied in the main checkout | It tried to write outside its worktree | Nothing; it moves to the worktree |
| `gh pr merge` denied | Merging is yours | Merge on GitHub |
| Push to main refused | The deny list, with branch protection behind it | Nothing |
| `xcodebuild` refused | Only the `just` recipes carry the destination pin and the guards | Nothing; it uses the recipe |
| Waiting on the simulator lock | Another run holds the simulator | Wait; it queues |
| A dash on a changed line | An em or en dash in the diff | Nothing; it fixes the line |
| `just claim` refused | Another branch or an open PR has the issue | Pick another issue, or finish that PR |
| Claim refused at two merged PRs | Two strikes (#1890) | Decide whether the approach is wrong |

## The experiment, and the Monday read

Off since 2026-09-17: the turn reminder (the model line at every boundary and
the rung-switch bounce), the post-push CI note, and tier naming in handoffs.
The style rules still hold; they live in the global `CLAUDE.md`.

Still on until their first read on 2026-09-21: the context watch, the read
guard, the spawn guard and the cold nudge. The harness log's rule is that a
lever whose number has not moved after two reads is reverted, not patched, so
the second read on 2026-09-28 decides them (#1990).

To read: `just usage 7 --quality` on the Monday, against the rows in
[`harness-log.md`](harness-log.md). To switch a hook off: delete its line from
the `hooks` block of `~/.claude/settings.json`; the script stays on disk. To
bring one back: copy its line from `~/.claude/backups/2026-09-17/settings.json`.
