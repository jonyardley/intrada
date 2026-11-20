# Worktrees

Every session that edits works in its own checkout. The rule: **at most one
session mutates the tracked files, index and HEAD of one working tree at a
time** (#1741). Reading is never banned.

## Making one

`just worktree-new <name>` wraps worktrunk's `wt switch --create <name> --base
origin/main`: it fetches first and refuses a name whose simulator would collide
with another worktree's. It ends by printing `cd <worktree> && claude`, the
command that starts the session inside the new worktree (#2918): the
`.claude/rules/` files load only there, and every command runs in place.
Worktrunk's machine-local config
(`~/.config/worktrunk/config.toml`) does the rest:

- **Where.** `../intrada-worktrees/<name>`, beside the main checkout.
- **Warm caches.** A `pre-start` hook runs `wt step copy-ignored`, which
  reflinks the gitignored files the main checkout's `.worktreeinclude` names
  (`ios/generated/`, `ios/build/spm/`, `.env`, `.claude/settings.local.json`,
  `android/local.properties`, #2630) before anything builds. `target/` and the
  Xcode build folders stay out (#2191). The file is read from the main
  checkout's working tree, so a change to it takes effect once that checkout
  is updated. A stale `ios/generated/` is harmless:
  `_ios-sync` regenerates when its stamp does not match the core.
- **A cmux workspace.** The same hook opens one at the new tree; `pre-switch`
  reopens a missing one and `pre-remove` closes it.

`just ios-gen` builds the Swift type generator in a second Rust build folder,
`target/codegen`, so it runs beside the Rust library build (#2742). Each
checkout, main included, pays about 13 s once to fill it at its first core
change, and the folder takes about 0.9 GB of disk.

`just worktree-status` says which worktrees are finished: each one's issue,
PR, CI, review, local changes and whose move it is, ending with a count of
those safe to remove. It reads GitHub once and removes nothing.

Once its PR merges, the session removes its own worktree and simulator with
`just worktree-done` from inside it, then ends. It refuses unless the PR is
merged, the merged PR's last commit is this HEAD and the tree is clean; the
guard checks the same. Only `just worktree-done` runs that merged, clean and
pushed check. A manual removal, `just worktree-rm <name>` or `git worktree
remove`, and a forced removal do not: they remove what they are given, and
`just worktree-rm` also cleans the simulator. The guard refuses the usual
removal commands when they name another live session's worktree; some unusual
command shapes are known limits it does not catch, and the protection that
holds is the merge lock on GitHub. Claude Code's own subagent worktrees stay in
`.claude/worktrees`.

## Three places, one rule each

Once the repo has a linked worktree, as intrada always does:

- **The main checkout.** Any number of sessions can sit here, but shell is
  read-only (the list below) and file tools cannot write, so main stays the
  clean base. To build, a session starts inside its worktree with
  `cd <worktree> && claude`; a `cd <worktree> && ` prefix from main is refused.
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
| `just worktrees`, `just worktree-status`, `gh pr view` | yes | yes | yes |
| `just worktree-new`, `wt switch`, `wt list` | yes | yes      | yes            |
| Any other `wt` subcommand | no         | yes           | no             |
| Build, test, `just check` | no         | yes           | no             |
| Edit, commit, push     | no            | yes           | no             |
| `just worktree-done`   | no            | once merged, clean and pushed | no |

A pull in the main checkout is denied with the other mutating git subcommands
(#1740); agents fetch instead, and Jon refreshes main's working copy.

Read-only is judged on the whole command: every segment of a compound command
must be read-only, and any redirection makes it a write. Quoted text and
heredoc bodies are data. The read-only list is a list, so expect a tool that
reads and is denied anyway. It allows the usual file and text commands, `sed`
without `-i`, `find` without `-delete` or `-exec`, read-only `git`, `rtk read`,
`just` limited to `worktrees`, `worktree-status`, `status`, `worktree-new` and
`--list`, `wt` limited to `switch` and `list`, and `gh` limited to `view`,
`list`, `checks` and `status`.

The guarantee is on the file tools. Shell coverage is best effort, aimed at
redirection, `sed -i`, `cp`, `mv`, `rm`, `tee`, `patch` and mutating `git`. A
shell write by absolute path into another session's worktree, from a third
folder, is not seen; file tools still are, and the merge lock on GitHub holds.
A broken guard fails open.

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
