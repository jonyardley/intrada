# How the work runs

How issues, epics, the board and releases are organised. What to build is in
[`roadmap.md`](roadmap.md); what is in flight is `just status`.

## Labels

Every issue a person or an agent opens carries a horizon and a kind (`bug`,
`ux`, `architecture`, `accessibility`, `documentation`, `ios`, `android`,
`security` or `epic`). `just issue-open` refuses one without both (#2696).

| Label | Purpose |
|-------|---------|
| `layer:capture` / `:plan` / `:space` / `:show` / `:guide` | The vision's five layers; `:space` is the Nudge layer |
| `horizon:now` / `:next` / `:later` | Rough timing, defined under [Horizons](#horizons) |
| `bug` | The app does something other than what it was built to do; a thing never built is not a bug |
| `documentation` | Docs, templates and the harness |
| `architecture` | Technical debt and infrastructure |
| `ux` / `accessibility` | Cross-cutting work on how the app feels and who can use it |
| `security` | Security-relevant |
| `ios` / `android` | The app the work touches |
| `epic` | A body of work whose issues are its sub-issues ([Epics](#epics)) |

### Horizons

- **Now**: a real musician hits a wall *because* this isn't built.
- **Next**: a real musician notices it's missing after a month of use.
- **Later**: the app could live without it for a year.

Where a horizon label disagrees with the roadmap's ranking, fix the label.

## Epics

An epic holds a body of work: a roadmap line, an audit's backlog, a spec's
phases or a tooling programme. The next piece of work is picked from an epic,
never from the flat list of issues; `just status` shows each open epic's done
count and next unclaimed child.

- **What earns one**: three or more issues that ship as separate PRs in an
  order worth writing down. A layer or journey label is only a filter, not an
  epic. Tidy-ups with no order stay single issues.
- **Membership is a GitHub sub-issue**, attached with
  `just epic-add EPIC N...` in working order; `just epic-move` takes an issue
  from the epic that holds it. A checklist in the body does not count. The
  parent carries the `epic` label, and no epic sits under another.
- **One parent per issue**, which GitHub enforces. Where two epics fit, the
  one that says when to build it holds the issue and the other names it in
  prose. An audit epic keeps its
  findings until it closes.
- **The body** follows the issue template; its What to do section names the
  working order the sub-issue list mirrors. The title names the outcome, with
  no "Epic:" prefix. #1967 is the model.
- **Horizon stays on each issue.** The epic carries its children's nearest.
- **Closing**: the session that closes the last child closes the epic in the
  same turn, after checking the body for work that never became an issue. An
  epic never closes with children open. When the direction changes, Jon closes
  it with a comment naming where the rest went.

## The board

The [project board](https://github.com/users/jonyardley/projects/2)'s columns
are workflow states: Backlog, In Progress, In Review and Done. Labels filter.
**The milestone is the release**: everything meant for the next release carries
its milestone. A `Priority` field (P0, P1, P2) ranks issues within an epic that
share a horizon.

## Cutting a release

A milestone is one headline plus whatever rides along. **Cut when the headline
works on iPhone and Android**, not on a fixed day, because the work comes in
bursts.

Write the headline on the first line of the milestone's description, where
`just status` reads it; a milestone with no description says so. Whatever is
still open at the cut rolls to the next milestone. If the headline keeps
growing, cut anyway and rename the milestone.

**A build on a phone does not need a release.**
`gh workflow run release-testflight.yml --ref <branch>` puts a signed build on
TestFlight from any branch in about 15 minutes. It skips the Sentry release
step and takes the version in `ios/project.yml`, so its crashes arrive with no
release attached.

**A release's write-up is its GitHub release**, generated from the milestone.
The phase is in the roadmap's [Now](roadmap.md#the-ranking) section.
