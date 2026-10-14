# Worktrees

Every session that edits works in its own checkout. The rule: **at most one
session mutates the tracked files, index and HEAD of one working tree at a
time** (#1741). Reading is never banned.

## Making one

`just worktree-new <name>` wraps worktrunk's `wt switch --create <name> --base
origin/main`: it fetches first and refuses a name whose simulator would collide
with another worktree's. Worktrunk's machine-local config
(`~/.config/worktrunk/config.toml`) does the rest:

- **Where.** `../intrada-worktrees/<name>`, beside the main checkout.
- **Warm caches.** A `pre-start` hook runs `wt step copy-ignored`, which copies
  the gitignored files the main checkout's `.worktreeinclude` names
  (`ios/generated/`, `ios/build/spm/`, `.env`, `.claude/settings.local.json`,
  `android/local.properties`, #2630) before anything builds. `target/` and the
  Xcode build folders stay out (#2191). A stale `ios/generated/` is harmless:
  `_ios-sync` regenerates when its stamp does not match the core.
- **A cmux workspace.** The same hook opens one at the new tree; `pre-switch`
  reopens a missing one and `pre-remove` closes it.

Remove one with `wt remove` from inside it, or `just worktree-rm <name>`, which
also cleans its simulator. Claude Code's own subagent worktrees stay in
`.claude/worktrees`.

## Three places, one rule each

Once the repo has a linked worktree, as intrada always does:

- **The main checkout.** Any number of sessions can sit here, but none can edit
  files or run shell that mutates the source, so main stays the clean base.
- **Your own worktree.** The first write claims it, and everything works.
- **Someone else's worktree.** Reads pass. Builds are denied with the writes,
  because two builds share one `target/` and one simulator.

## The lease

The claim is a file, `claude-worktree-lease`, in the worktree's own git
directory, so it is never committed and goes when the worktree does. The holder
is its Claude Code session id, or failing that the recorded process id and that
process's start time, so a recycled process id never reads as a live holder.

- **Taken** at session start for the starting tree, and on the first write for
  any other tree.
- **Released** when the session ends, sweeping every lease it took in the repo.
- **Taken over** once the holder has died. There is no timeout.
- **Never blocking in the main checkout.** Main's lease is written so
  `just worktrees` can say who is there, but main is held by the rule above.

## What the guard allows

`~/.claude/hooks/guard-worktree.sh` runs before `Edit`, `Write`, `MultiEdit`,
`NotebookEdit` and `Bash`, and judges a write by the worktree it lands in: the
file path for a file tool, the working directory for a shell command.

|                        | main checkout | your worktree | someone else's |
| ---------------------- | ------------- | ------------- | -------------- |
| Read a file            | yes           | yes           | yes            |
| `just worktrees`, `gh pr view` | yes   | yes           | yes            |
| `just worktree-new`, `wt switch`, `wt list` | yes | yes      | yes            |
| Any other `wt` subcommand | no         | yes           | no             |
| Build, test, `just check` | yes        | yes           | no             |
| Edit, commit, push     | no (unless it self-heals, below) | yes | no    |

A pull in the main checkout is denied with the other mutating git subcommands
(#1740); agents fetch instead, and Jon refreshes main's working copy.

Read-only is judged on the whole command: every segment of a compound command
must be read-only, and any redirection makes it a write. Quoted text and
heredoc bodies are data. The read-only list is a list, so expect a tool that
reads and is denied anyway. It allows the usual file and text commands, `sed`
without `-i`, `find` without `-delete` or `-exec`, read-only `git`, `rtk read`,
`just` limited to `worktrees`, `status`, `worktree-new` and `--list`, `wt`
limited to `switch` and `list`, and `gh` limited to `view`, `list`, `checks`
and `status`.

The guarantee is on the file tools. Shell coverage is best effort, aimed at
redirection, `sed -i`, `cp`, `mv`, `rm`, `tee`, `patch` and mutating `git`. A
broken guard fails open.

## Driving a worktree from the main checkout

A session in the main checkout prefixes each shell command with
`cd <worktree> && `. The guard judges the command where it lands, so commits,
gates and PRs work, and the first write takes the lease. It honours only a
shape it can read, and denies rather than guesses:

- a literal absolute path, followed by `&&` or `;`. `~`, `$HOME` and `$VAR` are
  not expanded, and `||` breaks the match.
- one `cd` only.
- nothing naming a path inside the session's own checkout, and no `..`.
  Absolute worktree paths in the command body are fine.
- `git -C <dir>` is never resolved and stays denied from main.

File tools take absolute paths inside the worktree.

### A forgotten prefix self-heals when unambiguous

When the session holds a lease on exactly one non-main worktree, a Bash command
with no `cd` gets the prefix added, and a relative file path is re-rooted under
that worktree (#1840). With zero or several such leases it still denies. So
does a command with its own `cd`, a `git -C`, or anything reaching into the
main checkout by absolute path or `..`.

## Escape hatches

Jon's, not an agent's. When the guard fires, the session says so and stops.

- `CLAUDE_ALLOW_MAIN_WRITES=1`, or `touch "$(git rev-parse --git-common-dir)/claude-allow-main-writes"`,
  allows writes in the main checkout.
- `~/.claude/hooks/worktree-lease.sh steal <path> <session-id>` hands a tree to
  another session when its holder has genuinely stopped.

## Where this lives

The hooks are machine-local: `guard-worktree.sh`, `worktree-lease.sh` and
`worktree-session.sh` in `~/.claude/hooks/`, tested by `guard-worktree.test.sh`.
Run it after any edit to either script, and mutation-test with `GUARD=` and
`LEASE=` pointing at doctored copies so live sessions keep working hooks.

The recipes are in [`working-with-agents.md`](working-with-agents.md); running
several sessions at once is in
`.claude/skills/intrada-parallel-streams/SKILL.md`.
