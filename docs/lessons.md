# Lessons

> What sessions learned, so the next one does not relearn it (#2801). Every
> session start prints the lessons below. The retro that writes them, and
> what each lesson's outcome is, is the first step of the route in the
> [shipping skill](../.claude/skills/intrada-shipping/SKILL.md#the-route).

- A lesson is one line: the date, the issue, then what to do differently and
  why. **What to look for**, one of each kind the session has: a mistake or
  correction; a slow or costly step, read from the session's row in
  `just usage 1 --quality`; and a tool, recipe or doc that misled.
- A lesson already here or in [`lessons-archive.md`](lessons-archive.md) is
  not added again: its line ends "(seen again N)", counting up, and an
  archived one is copied back here with its count raised above the
  archive's. A fixed lesson that comes back means the fix did not hold.
- **At most 30 lessons**; `just check` refuses more. The session that reaches
  the limit archives any "(issue #N)" lesson whose issue has closed, with
  "issue:" in front, then the oldest with no "seen again" and no open issue.
- The archive is never printed at session start and nothing is deleted from
  it. A worktree prints its own copy, so one behind main misses newer lessons.
- A local merge of main keeps both sides' lessons (#2937), and the lessons
  check refuses a line kept twice, bumped on two branches, or brought back
  from the archive.

## Lessons

- 2026-11-12, #2801: `just claim` refuses in the main checkout, so make the worktree first and claim from it, whatever order the session's brief gives (seen again 1).
- 2026-11-12, #2801: in auto mode, the rtk hook (this machine only) rewrites some commands after review, so the first try is refused; resend the rewritten command exactly as the refusal quotes it (seen again 1).
- 2026-11-13, #2847: `sed -E` on macOS rejected a backslash-heavy mutation and wrote an empty script, so the self-test's failure looked like a caught mutant; swap literal text with perl's `\Q...\E` and diff before running.
- 2026-11-14, #2866: lowering an agent's effort pin makes `guard-spawn.sh` refuse every spawn above it, so a pin that is lifted on some diffs needs an `effort-ceiling:` beside it, or the lift is blocked.
- 2026-11-14, #2866: a guard mutation that removes a line can stay green because the line it guards is shadowed (the first frontmatter match wins); give each mutation a fixture where only that line decides the result.
- 2026-11-15, #2869: a `--settings` flag that holds any sandbox list drops every sandbox list in the worktree's local settings (Claude Code 2.1.325), so a loop that passes `strictAllowlist` with its domains must pass osti's lists too; probe a sandbox change live, since the session starts without complaint.
- 2026-11-15, #2869: a sandbox path must be the real one: `/var/folders` and `/tmp` are links, and an allowance written through them is ignored; use `/private/var/...` and `/private/tmp/...`.
- 2026-11-15, #2869: inside Claude's sandbox Xcode cannot start its own for package manifests (`sandbox_apply: Operation not permitted`), even with the packages fetched beforehand; xcodebuild takes the switch as `-Key=YES`, not `-Key YES`.
- 2026-11-15, #2876: a workflow command property value holding a comma needs `%2C`, or the annotation title stops at the comma; assert the escaped string in the test.
- 2026-11-15, #2876: a scheduled job on the self-hosted Mac shares the gate's warm workspace, so a default checkout there would wipe `target/`; fetch only what it needs into a subfolder with `clean: false`.
- 2026-11-15, #2531: zsh does not split an unquoted expansion, so `${VAR:+--flag "$VAR"}` reaches the command as one argument; write the flag and its value as two words, and leave the flag off with a separate command when the value is empty.
- 2026-11-15, #2531: an agent's `just ios-test` built but ran no test, and it reported "compiles"; a build report is not a test run, so the lead's own gate run decides.
- 2026-11-15, #2531: GitHub allows at most 100 sub-issues per issue, so `just epic-add` refuses on the Android epic; a deferred issue for it stands alone until the epic is split.
- 2026-11-18, #2845: the plan said to reuse a helper in the status script, but the script defined it below the line that needed it; check where a shell function is defined before calling it from earlier in a script.
- 2026-11-18, #2845: for a one-line script fix, reverting the fixed line and rerunning the self-test showed both new assertions fail in one command, cheaper than any mutation tool (seen again 1).
- 2026-11-18, #2793: deleting whole snapshot classes by guess missed two split view references that also showed the changed words; grep every snapshot test file for each changed view, split views included, before deleting references.
- 2026-11-20, no issue: a claude.ai link from an earlier session can be a Claude Doc; the Artifact read returns only the type's instructions, so read the doc's tab through the Claude Docs connector with a search view.
- 2026-11-20, no issue: `wt switch -c name origin/main` prints usage and makes nothing; `just worktree-new name` makes the worktree from fresh origin/main.
- 2026-11-20, #2931: a session without the Xcode tools cannot render a SwiftUI preview; a sketch gets its picture from one snapshot test and `just ios-snapshots-record <Class>`, about 70 seconds cold and 20 warm.
- 2026-11-20, #2922: the sensitive surface rule named `ios/generated/`, which is gitignored and so never appears in a diff; judge a bridge change by the Facet derive the type generator follows, not by folder.
- 2026-11-20, #2934: a request to research a process step found it had shipped a week earlier (#2801); search issues and merged PRs for the topic before researching it.
- 2026-11-20, #2917: a cache saved by a tag run is visible to that tag alone, so the next tag restores only what a run on main saved; warm a tag-only workflow's caches from a main dispatch.
- 2026-11-20, #2936: a union merge of the lessons file kept both versions of an edited rule line above the list, and the lessons check reads only the list, so the doubled line passed; read the rules after any merge that touched them (issue #2943).
- 2026-11-24, #2921: the handover brief gave its own order and said to stop at the draft, and the session followed it over the ship route, so the pull request opened with no lessons, no review comment, no template headings and no closing line; read the ship route before opening a pull request, whatever the brief says (issue #2955).
- 2026-11-24, #2921: the agent guide says merging needs a code owner's approval, but main's protection has asked for none since 2026-11-20; read the protection with `gh api` before trusting a doc line about it (issue #2955).
- 2026-11-24, #2926: xcodebuild accepts an unknown build setting override without a word, and the compilation cache path setting is in no Xcode settings list; prove an override took by its effect (the cache folder filling), never by a clean exit.
- 2026-11-24, #2926: the guard refuses `xcodebuild -version` even though it builds nothing; read the installed version from Xcode's Info.plist with `plutil -p` instead.
- 2026-11-24, #2955: a reviewer launched from the main checkout could not run the self-tests, since the main checkout guard refuses running scripts there, so both reviews left the mutation claims unverified; a reviewer that must run tests needs its own worktree (issue #2960).
- 2026-11-24, #2198: another session merged a change to the same ready script while this one was building, and the clash only showed at the merge; before editing a shared script, check `just status` for in-flight work that touches it.
