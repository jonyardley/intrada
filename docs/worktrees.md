# Worktrees

Every session that edits works in its own checkout, and nothing else may edit
there while it does. On 2026-09-12 two sessions wrote one worktree and silently
overwrote each other's edits: `scripts/claim-issue.sh` claims a GitHub issue, and
nothing claimed the directory. This is the reference for what claims a directory
now, what it stops, and what it deliberately does not.

The rule it protects is narrow: **at most one session mutates the tracked files,
index and HEAD of one working tree at a time**. Not one session per repo, and
not a ban on reading.

## Three places, one rule each

Everything below assumes the repo has at least one linked worktree, which is how
intrada is always worked. Before the first one exists there is nothing to keep
separate, so the main checkout behaves like any other tree and one session holds
it at a time.

**The main checkout.** Any number of sessions can sit here, so a session never
arrives to find itself with nothing it can do. None of them can edit files or run
shell that mutates the source, which is what keeps main the clean base every
branch starts from.

**Your own worktree.** The first write claims it, and everything works inside.

**Someone else's worktree.** Reads pass, so a session can see what is there
before backing out. Builds are denied along with the writes, because two builds
share one `target/` and one throwaway simulator.

## The lease

The claim is a file called `claude-worktree-lease` in the worktree's own git
directory (`.git/worktrees/<name>/` for a linked worktree, `.git/` for the main
checkout). It cannot be committed, and it disappears when the worktree is
removed.

The holder is identified by its Claude Code session id. If no live process
matches that id, the recorded process id and that process's start time decide it
instead, which is what covers a session launched without a session id on its
command line. Liveness therefore survives process id reuse: a recycled id
running something that is not `claude` never reads as a live holder.

- **Taken** at session start for the tree the session starts in, and on the
  first write for any other tree the session drives.
- **Released** when the session ends, and the release sweeps **every** lease
  that session took in the repo, not only the one it started in. Before that
  swept, a session that had driven two worktrees handed back neither.
- **Taken over** when the holder has died. There is no timeout: a lease never
  expires while its holder is alive, however idle, and the next session claims
  it the moment that process is gone.
- **Never blocking in the main checkout.** Main's lease is still written, so
  `just worktrees` can say who is working there, but it never denies anything.
  Main is held by the rule above instead.

## What the guard allows

`~/.claude/hooks/guard-worktree.sh` runs before `Edit`, `Write`, `MultiEdit`,
`NotebookEdit` and `Bash`, and judges the write by the worktree it would land in:
the file path for a file tool, the working directory for a shell command.

|                        | main checkout | your worktree | someone else's |
| ---------------------- | ------------- | ------------- | -------------- |
| Read a file            | yes           | yes           | yes            |
| `just worktrees`, `gh pr view` | yes   | yes           | yes            |
| `just worktree-new`    | yes           | yes           | yes            |
| Build, test, `just check` | yes        | yes           | no             |
| Edit, commit, push     | no (unless it self-heals, below) | yes | no    |

A pull in the main checkout is denied along with the rest of the mutating git
subcommands, by design: agents fetch instead (#1740). `git fetch` is allowed,
and `just worktree-new` branches from fresh `origin/main` regardless. Jon
refreshes main's working copy; the only cost of it going stale is that a new
worktree seeds colder caches, and the bindings seed stops matching first.

Read-only is judged on the whole command, not its first word. Every segment of a
compound command has to be read-only, so `cat x && rm -rf y` is a write; a
redirection anywhere makes it one, so `echo x > f` is too. Quoted text is data,
so a pipe inside `grep -E 'a|b'` does not split the command, and a heredoc body
is data as well.

The read-only list is a list, not a principle, so expect to meet a tool that
reads and is denied anyway. It currently allows the usual file and text
commands, shell keywords, `sed` without `-i`, `find` without `-delete` or
`-exec`, read-only `git` subcommands, `rtk read`, `just` limited to `worktrees`,
`status`, `worktree-new` and `--list`, and `gh` limited to `view`, `list`,
`checks` and `status`. So `rtk read` reads while `rtk test` and `rtk err` run
whatever they are handed, and `gh pr checks` reads while `gh pr create` does not.

The guarantee is on the file tools. Shell coverage is best effort, aimed at the
shapes an agent actually writes files with: redirection, `sed -i`, `cp`, `mv`,
`rm`, `tee`, `patch`, and mutating `git`. A broken guard fails open rather than
blocking every write.

## Driving a worktree from the main checkout

A session cannot move its own working directory, so one in the main checkout
prefixes each shell command with `cd <worktree> && `. The guard resolves the
prefix and judges the command where it lands, so the commit, the gates and the
PR all work, and the first write takes the worktree's lease.

The prefix is honoured only in a shape the guard can read, and denies rather
than guesses:

- a literal absolute path, followed by `&&` or `;`. `~`, `$HOME` and `$VAR` are
  not expanded, and `||` breaks the match.
- one `cd` only. The command runs at the last one, so a second is never
  resolved.
- nothing in the command naming a path inside the session's own checkout, and no
  `..`. Both mean the prefix said nothing about where the write lands. The match
  is at a path boundary, so a sibling worktree at `intrada-worktrees/<name>` is
  not read as the `intrada` checkout, and absolute worktree paths in the command
  body are fine.
- `git -C <dir>` is deliberately not resolved and stays denied from main.

File tools take absolute paths inside the worktree as they always did.

### Forgetting the prefix self-heals when it is unambiguous

A session that forgot the `cd <worktree> && ` prefix, or wrote a relative file
path against the main checkout instead of the worktree, gets it added for it
rather than denied, when the session's own lease makes the answer unambiguous
(#1840, using a PreToolUse hook's `updatedInput` to rewrite the call before it
runs). A Bash command with no `cd` of its own is given the prefix; a file
tool's path is re-rooted at the same relative path under the worktree. An
absolute path already inside the worktree is unaffected either way, per the
row above.

This only fires when the session holds a lease on exactly one **non-main**
worktree. Main's own lease (taken at session start, see above) always exists
and never blocks, so it is never part of this count: zero means nothing has
been written to any worktree yet, so there is no candidate, and more than one
means the guard cannot tell which worktree the write was meant for. Both still
deny, same as before.

Left alone too, still denied outright rather than rewritten: a command that
already tries to move somewhere with its own `cd` (prepending a second one in
front of it would relocate the escape, not close it), a command that names a
directory explicitly (`git -C`, which ignores any `cd` regardless), and a
command or path that already reaches back into the main checkout by absolute
path or `..`, the same reach-back a manually typed prefix is checked against
above. Adding a prefix only changes where a *relative* path lands; one of
these would still write into main regardless of what came in front of it.

If a machine's Claude Code build does not honour a PreToolUse hook's
`updatedInput`, this simply never fires and every case above denies exactly as
it did before #1840.

## Escape hatches

Jon's, not an agent's. When the guard fires, the session says so and stops.

- `CLAUDE_ALLOW_MAIN_WRITES=1`, or `touch "$(git rev-parse --git-common-dir)/claude-allow-main-writes"`,
  allows writes in the main checkout.
- `~/.claude/hooks/worktree-lease.sh steal <path> <session-id>` hands a tree to
  another session when its holder has genuinely stopped.

## Where this lives

The hooks are machine-local and in no repository: `guard-worktree.sh`,
`worktree-lease.sh` and `worktree-session.sh` in `~/.claude/hooks/`, with a
battery of 100 cases in `guard-worktree.test.sh`. Run the battery after any edit
to either script, and mutation-test rather than trusting a green: set `GUARD=`
and `LEASE=` to doctored copies so a mutation run never breaks the hooks other
live sessions are relying on.

On a machine whose copy predates all this, nothing is enforced and the `cd`
prefix is denied, which puts a main-checkout session back to handing commands
over.

The recipes, and how a session picks up the path-scoped rules, are in
[`working-with-agents.md`](working-with-agents.md). Running more than one
session at once has its own rules in
`.claude/skills/intrada-parallel-streams/SKILL.md`.
