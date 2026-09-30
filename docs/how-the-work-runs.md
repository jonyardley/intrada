# How the work runs

How issues, epics, the project board and releases are organised. What to
build and in what order is in [`roadmap.md`](roadmap.md); what is in flight
right now is `just status`, which reads GitHub.

## Labels

| Label | Purpose |
|-------|---------|
| `layer:capture` / `:plan` / `:space` / `:show` / `:guide` | The vision's five layers; `:space` is the Nudge layer |
| `horizon:now` / `:next` / `:later` | Rough timing, defined under [Horizons](#horizons) |
| `architecture` | Technical debt and infrastructure |
| `ux` / `accessibility` | Cross-cutting work on how the app feels and who can use it |
| `security` | Security-relevant |
| `ios` | The native SwiftUI app |
| `epic` | A body of work whose issues are its sub-issues ([Epics](#epics)) |

### Horizons

- **Now**: a real musician hits a wall *because* this isn't built.
- **Next**: a real musician notices it's missing after a month of use.
- **Later**: the app could live without it for a year.

The roadmap's ranking is the intent. Where an issue's horizon label disagrees
with it, fix the label.

## Epics

An epic holds a body of work: a roadmap line, an audit's backlog, a spec's
phases or a tooling programme. The next piece of work is picked from an epic:
`just status` shows every open epic with its done count and its next open,
unclaimed child.

- **What earns one**: three or more issues that ship as separate PRs and have
  an order worth writing down. A layer or journey label is only a filter.
  Tidy-ups with no order between them stay single issues.
- **Membership is a GitHub sub-issue**, attached with
  `just epic-add EPIC N...` in working order; `just epic-move` takes an issue
  from the epic that holds it. A checklist in the body does not count. The
  parent carries the `epic` label, and no epic sits under another.
- **One parent per issue**, which GitHub enforces. Where two epics fit, the
  one that says when to build it holds the issue and the other names it in
  prose. An audit epic keeps its findings until it closes.
- **The body** follows the issue template, and its What to do section names
  the working order, which the sub-issue list mirrors. The title names the
  outcome with no "Epic:" prefix, since the label says it. #1967 is the model.
- **Horizon stays on each issue.** The epic carries the nearest of its
  children's, so filtering on a horizon still finds it.
- **Closing**: the session that closes the last child closes the epic in the
  same turn, after checking the body for work that never became an issue. An
  epic never closes with children open; they move or close first. When the
  direction changes, Jon closes it with a comment naming where the rest went.

## The board

The [project board](https://github.com/users/jonyardley/projects/2)'s columns
are workflow states: Backlog, Ready, In Progress, In Review and Done. Labels do
the filtering.

**Ready means someone could start it without asking a question first**: the
shape is settled and what it points at has been read. It says nothing about
which build it lands in.

**The milestone is the release.** Everything meant for the next release carries
that release's milestone, which the board holds as a field and filters on.
Keeping Ready and the milestone apart lets an issue be ready to start and
deliberately left out of the release, or in the release and not yet
understood, without either field being wrong.

A `Priority` field (P0, P1, P2) ranks issues within an epic when several share
a horizon.

## Cutting a release

A milestone is one headline plus whatever rides along. **Cut when the headline
works on the phone.** The work here comes in bursts, so a fixed day would fire
on the empty weeks and miss the busy ones. v0.9.0 (photos) and v0.10.0
(capture) were both cut this way before the rule was written down.

Write the headline on the first line of the milestone's description.
`just status` reads it from there, so a milestone with no description says so
rather than looking like a release with no work in it.

Whatever is still open in the milestone at that point rolls to the next one.
If the headline keeps growing, cut anyway and rename the milestone: before
beta, a version number costs nothing.

**A build on a phone does not need a release.**
`gh workflow run release-testflight.yml --ref <branch>` puts a signed build on
TestFlight from any branch in about 15 minutes. It skips the Sentry release
step, which runs only on a tag, and takes the version in `ios/project.yml`, so
its crashes arrive without a release attached and it sits beside the released
build under the same version number.

**A release's write-up is its GitHub release**, generated from the milestone.
[`where-we-are.md`](where-we-are.md) changes only when the phase changes.
