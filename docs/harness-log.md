# Harness log

> Every change to the harness since 2026-09-12, with the date it merged and the
> number it should move. Harness, speed, rework and spend are defined in the
> glossary in [`reference.md`](reference.md). Set on 2026-09-15 (#1897),
> replacing the freeze on harness work set the day before.

## The rule

1. A harness change, in the repo or on Jon's machine, adds its row here in the
   same PR. A machine-local change with no PR of its own gets its row in the
   next harness PR.
2. Every Monday, `just usage 7 --quality`, and GitHub for speed and fix PRs,
   are read against the table below, and each row whose first read has come
   gets what its number did in its Reads cell.
3. A lever whose number has not moved after two reads is reverted, not patched.
   A row whose number is "none" is a measure, not a lever. A number marked
   "not in the report" is counted by hand at its read and judged like any
   other. A number several rows share, such as corrections, reverts nothing
   until the Monday read has classified corrections by kind; from then each
   row is judged on its own kind. Decided 2026-09-15 (#1898).
4. The 2026-09-27 `just usage 14` read stays as the first fortnight check for
   #1849.

## Where each number is read

| Number | Measure | Read from | Baseline |
|---|---|---|---|
| Speed | Claim to merged PR | GitHub: the claim comment on the issue to the merge of its last PR | None yet; the 2026-09-21 read sets it |
| Rework | Corrections, by dominant rung | `just usage 7 --quality`, "By dominant rung" | 7% to 12% of prompts on every rung, 11 to 14 September (#1893) |
| Rework | Rung switches per session | `just usage 7 --quality`, the `sw` column, printed for the thirty costliest sessions only | 5 of 125 sessions switched, four by hand (#1893) |
| Rework | Sessions and fix PRs per issue, reverts | `just usage 7 --quality`, "Issues touched by 2+ sessions"; GitHub for fix PRs and reverts | The page-crop bug took three fix PRs and eleven sessions (#1893) |
| Spend | Total and subagent share | `just usage 7`, the first line, divided by seven | $408 a day ($1,225 over 12 to 14 September), subagents 43% (#1849) |
| Spend | Sessions past 200k | `just usage 7`, the main sessions line | 30 of 94 (#1849) |
| Spend | Repeat text reads | `just usage 7`, the Reads line | 51%, 981 of 1,930 (#1849) |
| Spend | Cold turns | `just usage 7`, the cache writes line | 194 turns and $233 in the week to 2026-09-13 (#1842) |
| Spend | Long subagent runs and gate runs inside them | `just usage 7 --quality`, "Subagent runs by type" and "Subagent runs over 100 turns" | The five biggest `task` runs $246 of $557; 264 gate runs across 44 `task` runs (#1893) |

## Changes

The first read for every row below is Monday 2026-09-21 unless the row says
otherwise. The baselines above come from 11 to 14 September, so a row dated
before 2026-09-15 is part of the baseline it is read against. Machine-local
means a file under `~/.claude/` on Jon's machine, dated by the PR that
documented it, or by the file's own date where no PR did.

| Date | PR | What changed | Should move | First read | Reads |
|---|---|---|---|---|---|
| 2026-09-12 | #1706 | `just claim` refuses an issue already claimed on another branch or referenced by an open PR | Rework: an issue built twice, as #1694 was on 2026-09-11 (not in the report) | 2026-09-21 | |
| 2026-09-12 | #1712 | `just worktrees` names the session holding each lease; machine-local `worktree-session.sh` and `worktree-lease.sh` take and print the lease at session start | Rework: corrections | 2026-09-21 | |
| 2026-09-12 | #1721 | `just worktree-new` stops launching a Claude session of its own | Rework: corrections | 2026-09-21 | |
| 2026-09-12 | #1741 | The worktree rules move into one reference, `docs/worktrees.md` | Rework: corrections | 2026-09-21 | |
| 2026-09-13 | #1773 | Agents touch the main checkout only to `git fetch` | Rework: corrections | 2026-09-21 | |
| 2026-09-13 | #1788 | `just usage`, with machine-local `usage-report.py` and the day's first usage line from `usage-daily.sh` | None: the measure itself | 2026-09-21 | |
| 2026-09-13 | machine-local | `cmux-worktree-pill.sh` and `cmux-driving-pill.sh` show in the cmux sidebar which worktree a session is driving | Rework: corrections | 2026-09-21 | |
| 2026-09-13 | #1789 | `task` pinned to `high` effort | Spend: subagent share | 2026-09-21 | |
| 2026-09-13 | #1790 | Sessions open on Sonnet 5 medium; machine-local `statusline.sh` shows context in thousands, amber from 200k and red from 400k | Spend: total; sessions past 200k | 2026-09-21 | |
| 2026-09-13 | #1792 | `test-runner` pinned to Haiku 4.5 | Spend: subagent share | 2026-09-21 | |
| 2026-09-14 | #1816 | The bridge rung split into deciding and building | Rework: fix PRs on bridge work | 2026-09-21 | |
| 2026-09-14 | #1817 | The model ladder shaped by activity, with measured cost and latency per rung | Rework: rung switches | 2026-09-21 | |
| 2026-09-14 | machine-local | `guard-spawn.sh` refuses a spawn that lifts model or effort above the definition's pin, `reviewer` excepted | Spend: `task` runs lifted to `xhigh`, $353 in the fortnight to 2026-09-14 | 2026-09-21 | |
| 2026-09-14 | #1843 (#1837) | One worktree rule: the session makes and drives its own worktree | Rework: corrections, eleven about worktrees over 12 to 14 September | 2026-09-21 | |
| 2026-09-14 | #1843 (#1839) | Two streams by default; "what's next" answered from `just status` alone | Spend: "what's next" sessions, seven over 12 to 14 September (not in the report) | 2026-09-21 | |
| 2026-09-14 | #1843 (#1841) | The usage guidelines written into `docs/working-with-agents.md` | None: where later levers land | 2026-09-21 | |
| 2026-09-14 | machine-local | Global `CLAUDE.md` cut from 264 to about 210 lines; twelve auto-memories folded into two | Spend: total | 2026-09-21 | |
| 2026-09-14 | #1850 (#1838) | `effort:` pinned in every agent definition; `Explore`, `fork` and `general-purpose` spawned only from `high` or below | Spend: subagent share | 2026-09-21 | |
| 2026-09-14 | #1851 (#1847) | `reviewer` pinned to Sonnet 5 high, lifted to Opus on the sensitive surfaces | Spend: subagent share; `reviewer` was $77 over 52 runs | 2026-09-21 | |
| 2026-09-14 | #1852 (#1848) | Machine-local `context-watch.sh` warns at 150k, hands the unit to a subagent at 200k and says `/compact` at 400k | Spend: sessions past 200k | 2026-09-21 | |
| 2026-09-14 | #1853 (#1846) | The lead verifies a `task` result through `reviewer` and the gate counts, never by re-reading its files | Spend: subagent share; `task` was $391, 32% | 2026-09-21 | |
| 2026-09-14 | machine-local (#1845) | `usage-report.py` weights image reads by token estimate and prints the repeat-read share | None: the measure itself | 2026-09-21 | |
| 2026-09-14 | #1854 (#1844) | Machine-local `read-guard.sh` denies a repeat whole-file Read, and an unranged one over 400 lines | Spend: repeat text reads | 2026-09-21 | |
| 2026-09-14 | #1855 (#1842) | Machine-local `cold-nudge.sh` on a launchd timer notifies when a large session has sat idle 50 minutes | Spend: cold turns | 2026-09-21 | |
| 2026-09-14 | #1856 (#1840) | Machine-local `guard-worktree.sh` adds a forgotten `cd` prefix when the session holds one lease, instead of denying | Rework: denied writes in main, 51 over 12 to 14 September (not in the report) | 2026-09-21 | |
| 2026-09-14 | #1859 (#1857) | The settings-path table notes `CLAUDE_CONFIG_DIR` | None: a correction | 2026-09-21 | |
| 2026-09-14 | #1863 (#1861) | Every agent definition tells the subagent to work in the worktree its brief names | Rework: corrections | 2026-09-21 | |
| 2026-09-14 | #1869 (#1864) | The when-to-do-what table in `docs/working-with-agents.md` | Spend: status and handover-only sessions, 12 of 76 (not in the report) | 2026-09-21 | |
| 2026-09-14 | #1870 (#1865) | `docs/agentic-primer.md` rewritten as a from-scratch guide for another repo | None: a reference for other projects | 2026-09-21 | |
| 2026-09-14 | #1887 (#1884) | The full iOS gate runs once per PR, not after every review fix | Speed: claim to merged PR; the gate's wall time is tracked on #1884 | 2026-09-28, since the 2026-09-21 read sets the speed baseline | |
| 2026-09-14 | machine-local (#1892) | `usage-report.py --quality` reports rung switches, corrections, long subagent runs, gate runs inside subagents and issues across sessions | None: the measure itself | 2026-09-21 | |
| 2026-09-15 | #1896 (#1888) | `task` runs no gates (machine-local `guard-bash.sh` denies them), stops after 60 turns or a second failed build, and refuses a brief over ten files | Spend: long subagent runs and gate runs inside them | 2026-09-21 | |
| 2026-09-15 | #1896 (#1889) | The opening reply gives the exact `/model` and `/effort` and stops when the rung is wrong (machine-local `turn-reminder.txt`); openers start with them | Rework: rung switches | 2026-09-21 | |
| 2026-09-15 | #1896 (#1890) | `just claim` refuses a third claim at two merged PRs without a named decision; two corrections on one point stop the change | Rework: sessions and fix PRs per issue | 2026-09-21 | |
| 2026-09-15 | #1896 (#1891) | A screen change runs on Opus 5 high and shows Jon its screenshot before the PR opens | Rework: reverts | 2026-09-21 | |
| 2026-09-15 | #1895 | `just usage` forwards extra flags, so `--quality` runs through the recipe | None: the measure itself | 2026-09-21 | |
| 2026-09-15 | #1899 (#1897) | The harness words in the glossary; this log and the Monday read replace the freeze | None: the measure itself | 2026-09-21 | |
| 2026-09-15 | #1900 (#1897) | Sessions open on Opus 5 `xhigh`; `task` and `reviewer` pinned to Opus 5 high; `smol` deleted; the activity ladder becomes three rungs and the Fable list; Tier 2 and 3 start with a plan comment on the issue | Speed: claim to merged PR; rework: corrections, by dominant rung. Spend is forecast to rise about $200 a week (#1897) | 2026-09-28, since the 2026-09-21 read sets the speed baseline | |
| 2026-09-15 | machine-local | `turn-reminder.txt` asks for the `/model` and `/effort` switch only for work on the Fable list | Rework: rung switches | 2026-09-21 | |
| 2026-09-15 | #1902 (#1897) | `docs/working-with-agents.md` rewritten along the process steps; the primer, the shipping skill, `/ship` and two path-scoped rules brought into line with the three rungs and the plan comment | Rework: fix PRs on the sensitive surfaces, now built on Fable at high rather than max | 2026-09-21 | |
| 2026-09-15 | machine-local | `context-watch.sh` hands the rest of a unit past 200k to `task` alone, and the global `CLAUDE.md` stops naming the deleted `smol` agent | None: a correction | 2026-09-21 | |
| 2026-09-17 | #1991 (#1989) | Machine-local `turn-reminder.txt` (the model line at every boundary and the rung-switch bounce) and `post-push-watch.sh` unhooked, with a dated backup in `~/.claude/backups/2026-09-17/`; the tier naming leaves the global `CLAUDE.md` handoffs and token-spend sections; `docs/playbook.md` is the one-page reminder | Rework: corrections of the harness-nag kind, rung switches; Spend: total, which should not rise | 2026-09-21 | |
| 2026-09-17 | #1985 | Sessions open on Opus 5 `high`, not `xhigh`, reversing that half of the 2026-09-15 row (#1900); the Fable list narrows to the four silent-failure surfaces, with the plan comment, the direction call and the unexplained bug following the work rather than triggering on their own | Spend: $ per turn by rung, and Fable's share of the fortnight (32% of spend on 20% of turns before the change) | 2026-09-24 | |
| 2026-09-17 | machine-local (#1985) | `turn-reminder.txt` and the global `CLAUDE.md` name Opus 5 `high` and the narrowed Fable list | Spend: rung switches per session | 2026-09-24 | |
| 2026-09-17 | #1986 | `just handover [N]` prints the next session's opener, so finishing a unit and clearing costs a paste rather than a retelling | Spend: sessions past 200k, and the 123 cold turns that cost $199 in the fortnight to 2026-09-17 | 2026-09-24 | |
| 2026-09-17 | machine-local (#1986) | `read-guard.sh` also denies a bare `cat`, `sed`, `head` or `tail` of a file already in context or over 400 lines, and its hook matcher widens from `Read` to `Read\|Bash`, which is what the Read-only registration let round it | Spend: tool output tokens, and the 42% of text reads that repeated a file | 2026-09-24 | |
| 2026-09-17 | #1992 | `docs/agentic-primer.md` deleted: it still taught the rung line at every boundary and the old ladder after #1985 and #1989. The generic newcomer guide is the shared Claude Doc, which took the primer's start-small sections, linked from `working-with-agents.md` and the README | None: a correction, one generic guide instead of two that disagreed | 2026-09-24 | |
| 2026-09-23 | #2045 (#2044) | Trial to 2026-10-07: the Fable list runs on Opus 5.5 `xhigh` instead of Fable 5.1 `high`, `task` drops to Opus 5.5 `medium`, the lead stays on Opus 5.5 `high` as the control, and the Fable review of a sensitive diff stays as the check. A Fable review Blocker the Opus 5.5 build missed puts that surface back on Fable; two end the trial. The lead had already moved from Opus 5 to Opus 5.5 that morning through the `opus` alias | Spend: $ per turn by rung and Fable's share (32% of spend on 20% of turns before #1985); rework: corrections by dominant rung, and Blockers from the Fable reviews of sensitive diffs, not in the report | 2026-09-27, with #1984; the verdict on 2026-10-07 | |
| 2026-09-23 | machine-local (#2044) | The global `CLAUDE.md` handoffs line and `turn-reminder.txt` (still unhooked since #1989) name Opus 5.5 `high` and the trial's `xhigh` for the Fable list | None: a correction, so a new session does not get two rules | 2026-09-27 | |
| 2026-09-23 | #2052 (#2044) | For the rest of the trial the lead drops from Opus 5.5 `high` to `medium`, where every session now opens, and the Fable list from Opus 5.5 `xhigh` to `high`. The trial gives up its `high` control; the Fable review of a sensitive diff stays as the check, and the stop rule is unchanged. The global `CLAUDE.md` handoffs line still names the old rungs and is Jon's to edit | Spend: $ per turn in the lead against Opus 5 `high` and Opus 5.5 `high` earlier on 2026-09-23; rework: corrections by dominant rung, and Blockers from the Fable reviews of sensitive diffs, not in the report | 2026-09-27, with #1984; the verdict on 2026-10-07 | |
| 2026-09-23 | #2064 (#2044) | Fable retired from every role, the review of a sensitive diff included. `reviewer` stays on its Opus 5.5 `high` pin for every diff and `/ship` no longer lifts it; "the Fable list" is renamed the sensitive surfaces, still built at Opus 5.5 `high`. Ends the trial's stop rule, which needed a Fable review to count a miss | Spend: total, which should fall with no Fable turns; rework: Blockers found after merge on a sensitive surface, not in the report | 2026-09-27, with #1984; the verdict on 2026-10-07 | |
| 2026-09-23 | machine-local (#2044) | `guard-spawn.sh` refuses any spawn on Fable and no longer lets `reviewer` lift its model; the global `CLAUDE.md` handoffs and token-spend lines and `turn-reminder.txt` name the sensitive surfaces instead of the Fable list | None: a correction, so no session is sent to a retired model | 2026-09-27 | |
| 2026-09-24 | #2095 (#2092) | The whole-app audit becomes repeatable: `docs/audit.md` as the rubric, `just audit-sweep` for the counts, and the saved `audit` workflow for the review | None: the measure itself, of the app rather than the harness | 2026-09-28 | |
| 2026-09-24 | #2123 (#2113) | The Mac gate runs on a push to main only when native paths changed, and a tree the runner has already seen pass skips the unit and UI tests there, on a PR or on main | Speed: claim to merged PR; the wait for the Mac runner on a PR, median 78 s and 529 s at the 90th percentile over 23 to 24 September (not in the report) | 2026-09-28 | |
| 2026-09-24 | #2123 (#2114) | `/ship` stops running the full iOS tier locally: the fast tier plus any UI class the diff edits, with CI's gate on the draft as the first full UI run. Takes #1887's row from once per PR to none | Speed: claim to merged PR; local gate time per PR, 4 to 5.5 minutes for the full tier in test-runner reports (not in the report) | 2026-09-28 | |
| 2026-10-13 | #2594 (#2593) | Applies the [rules review](https://claude.ai/artifact/VcAmj1s2USL15ceZxvg9ad). A `task` subagent may build an additive core or bridge change with a LiveBridge test, the lead pairing `reviewer`; the blob, wire-pinned settings, migrations, auth and non-additive changes stay with the lead, at `high`. The `task` brief cap counts hand-written files only | Speed: claim to merged PR on core slices; core slices built by `task` rather than the lead, from none (not in the report); no Blocker after merge on a bridge type | 2026-10-19 | |
| 2026-10-13 | #2594 (#2593) | An additive bridge change ships as one PR, core commits first; two PRs only for lead-only work (`.claude/rules/sensitive-surfaces.md`) | Speed: PRs per bridge issue, from 2 to 1 by the 2026-10-26 read (18 of 58 PRs in the week to 2026-10-13 were halves of a split; not in the report) | 2026-10-19 | |
| 2026-10-13 | #2594 (#2593) | A screen change opens as a draft with its screenshot and goes ready only after Jon's dated yes, instead of waiting before the PR | Speed: claim to merged PR on screen work; every screen PR goes ready carrying the line "Jon approved the screenshot on YYYY-MM-DD" (2 of 23 did; not in the report) | 2026-10-19 | |
| 2026-10-13 | #2594 (#2593) | `just pr-open` refuses a Tier 1 or 2 body over 350 words without a Tier 3 section and warns on sentences over 25 words; the two Tier 3 sections leave the Tier 1 and 2 template; the style guide borrows ASD-STE100's sentence caps | Speed: claim to merged PR; median Tier 1 and 2 body, from 544 words to under 350 (not in the report) | 2026-10-19 | |
| 2026-10-13 | #2594 (#2593) | `just claim`'s third-fix check counts merged PRs titled with the issue, leaving out Dependabot | Rework: decision-named claims, from about 7 a week to under 1, with no real third fix let through (not in the report) | 2026-10-19 | |
| 2026-10-13 | #2594 (#2593) | The next Tier 2 child of an epic whose body names it posts its plan and builds without waiting for approval; the matching global `CLAUDE.md` wording is Jon's to make | Speed: claim to first commit on epic children (not in the report) | 2026-10-19 | |
| 2026-10-13 | #2594 (#2593) | `just pr-ready PR ISSUE` marks a PR ready and moves the board to In review; the after-merge steps lean on the closing keyword and the board's own automation; the Ready status leaves `project-status.sh`; the board citation is #1659, not #1660 | None: a correction, so the board stops missing In review | 2026-10-19 | |
| 2026-10-13 | #2594 (#2593) | One statement of who reviews: `reviewer` on every code or test diff, `/code-review` for text only. Draft, never-merge, CI-watch and comment rules each kept where a hook or one file holds them; CLAUDE.md back to 196 lines | None: a correction, so every agent reads one rule; CLAUDE.md lines, from 210 | 2026-10-19 | |
| 2026-10-13 | #2594 (#2593) | Simulator waits since the full UI tier moved to CI (#2114), counted before deciding on a third iOS stream | None: the measure, for the 2026-10-19 read | 2026-10-19 | |
| 2026-10-13 | #2595 | Trial to 2026-10-26: the rules every session loads cut from 4,932 words to about 1,000. `CLAUDE.md` keeps commands, the architecture invariants and the gotchas (1,878 to 446 words); code style and tests move to the path-scoped `code-and-tests.md`, the tiers into `working-with-agents.md` step 4. Every hook kept. Undo: corrections above 0.93 a session at either read, a quarter over the baseline, and the cut prose comes back by revert, the global file by Jon's hand. The #2594 rows read the same Mondays, so a rise is checked against corrections that cite a cut rule before it is pinned on this one; `reviewer` Blockers and Important findings are counted at each read too, since a missed code or test rule shows there | Rework: corrections a session, 0.74 (about 230 corrected prompts over 309 sessions, `just usage 7 --quality` on 2026-10-13); speed: claim to merged PR, which has no recorded figure yet, so the 2026-10-19 read counts the fortnight before 2026-10-13 as the baseline | 2026-10-19; the verdict on 2026-10-26 | |
| 2026-10-13 | machine-local (#2595) | The auto-memory index cut to bare titles, 753 to 150 words, every memory file kept (backup in `~/.claude/backups/2026-10-13/`); Jon's global `CLAUDE.md` cut to about 275 words from wording in #2595, by his hand | The #2595 row's numbers. The global cut is pending Jon's hand: until it lands the always-loaded total is about 2,900 words, and the read says which days ran with it | 2026-10-19; the verdict on 2026-10-26 | |
| 2026-10-14 | #2668 (#2632) | The path rules, agent briefs, `/ship` and the shipping and streams skills cut to the rule and its reason, every hook and `paths:` list unchanged. A rule that moved files leaves a one-line pointer where it used to load, so it still fires on the paths it protects | None: a trim inside the #2595 trial, read with its numbers; `reviewer` Important findings that cite a cut rule are counted at each read | 2026-10-19; the verdict on 2026-10-26 | |
| 2026-10-15 | #2697 (#2696) | `just issue-open` refuses an issue without a horizon and a kind label; bug reports name the app, build and device; the PR checklist drops the three lines CI already proves; the style guide, sentence caps included, binds every technical and design document | Rework: issues opened with no horizon label, 8 of the 30 to 2026-10-15 (not in the report) | 2026-10-26 | |

## Later levers

Named, not started. Each gets a row above when it ships.

| Lever | Should move |
|---|---|
| The isolation machinery, once a day of streams has been watched live | Rework: corrections |
| Corrected prompts classified by kind (scope, approach, style, harness nag), the first Monday read's job | None: it makes the rework number readable by cause |
| Unhook the spend nudges (context watch, read guard, spawn guard, cold nudge) if rule 3 says so at the 2026-09-28 read (#1990), and shorten step 8 of `working-with-agents.md` | Spend: total; rework: corrections |
