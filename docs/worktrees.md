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

- **The main checkout.** Any number of sessions can sit here, but its working
  tree is read-only: the shell may only look (the list below) and file tools
  cannot write, so main stays the clean base. Writes outside every checkout,
  such as the scratchpad, still pass. To build, a session starts inside its
  worktree with `cd <worktree> && claude`; a `cd <worktree> && ` prefix from
  main is refused for writes, and reads pass.
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
`NotebookEdit` and `Bash`. A file tool is judged by the checkout its path lands
in, and a session in main may not write into any checkout with one. A shell command is split into parts, and each part is judged in the folder
it runs in; a write to a named path (a redirection, `cp`, `mv`, `rm`, `tee`,
`sed -i`, `perl -i` and the like) is judged by where that path lands.

|                        | main checkout | your worktree | someone else's |
| ---------------------- | ------------- | ------------- | -------------- |
| Read a file            | yes           | yes           | yes            |
| The `just` reads listed below | yes | yes  | yes            |
| `just worktree-new`, `wt switch`, `wt list` | yes | yes      | yes            |
| Any other `wt` subcommand | no         | yes           | no             |
| `gh`                   | see below     | yes           | reads only     |
| Build, test, `just check` | no         | yes           | no             |
| Edit, commit, push     | no            | yes           | no             |
| Write outside every checkout | yes     | yes           | yes            |
| `git worktree remove`, `just worktree-rm` | unless another session holds it | the same | no |
| `just worktree-done`   | no            | once merged, clean and pushed | no |

A pull in the main checkout is denied with the other mutating git subcommands
(#1740); agents fetch instead, and Jon refreshes main's working copy.

Read-only is judged on every part of a compound command. Quoted text, heredoc
bodies and here-strings are data. The read-only list is a list, so expect a
tool that reads and is denied anyway. It allows:

- the usual file and text commands, `sed` without an in-place flag, `sort`
  without `-o`, `find` without `-delete` or `-exec`, and `mktemp` into the
  temp folder;
- read-only `git`, including `git branch` with no name or only listing flags,
  and no `--output` to a file in the checkout; `rtk read`;
- `just` limited to `worktrees`, `worktree-status`, `status`, `worktree-new`,
  `handover`, `issue`, `claim`, `ready-queue`, `--list` and a bare `just`,
  never with `--fmt`;
- `wt` limited to `switch`, `list`, `help`, `config show`, `hook show`,
  `step diff` and `step eval`, plus `--help` and `--version`;
- in main, every `gh` call but `pr checkout`, `repo clone` and `repo sync`;
  in someone else's worktree, `gh` limited to `view`, `list`, `checks`,
  `status` and `diff`.

A redirection is a write unless it goes to `/dev/null`, to a path main's
`.gitignore` excludes (runtime state such as a ledger, spelled plainly), or to
a path outside every checkout, such as the scratchpad or `/tmp`. A script in a
temp folder outside every checkout, run by absolute path with `python3`,
`perl` or `ruby`, is not a build where it runs; inline code (`-c`, `-e`) is
still judged.

The guard follows a literal `cd` or `pushd`, and a literal `git -C <path>` for
that one part; a subshell's `cd` ends with the subshell. A `cd` it cannot
follow (a variable, a substitution, `-`, a folder that does not exist yet)
makes the rest of the command read-only. From main, a prefix such as
`cd <worktree> && ` is refused for writes; reads pass.

The guarantee is on the file tools. Shell coverage is best effort, and the
guard is a convenience: the merge lock on GitHub is what holds. Known limits,
recorded rather than fixed:

- a write from main must spell its path literally: a variable such as `$D` or
  `$TMPDIR` is not resolved, so the write is refused even when it lands
  outside every checkout;
- a shell write by absolute path from a command the guard does not know
  writes, run from a third folder, into another session's worktree;
- `make -C <path>` or `bash -c '...'` into another session's worktree, since
  only `cd`, `pushd` and `git -C` move a part;
- `awk` printing to a file;
- a quoted or commented `<<`, or `$((1<<n))`, can hide the lines after it, as
  can a heredoc tag with a dash or backslash;
- `sort -uo <path>` and `sort --output <path>`, perl in-place clusters such as
  `-lpi`, `-0pi` and `-ni`, and `ruby -pi`;
- a named path is judged where its part runs, not where it lands, for
  `find <path> -delete`, `xargs ... cp`, `cp --target-directory <dir>`,
  process substitution and `env -C`.

A broken guard fails open, except for a worktree removal.

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
