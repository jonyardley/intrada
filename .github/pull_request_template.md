<!-- What each section holds, and the 350-word cap on a Tier 1 or 2 body
     before the checklist: the PR bodies section of
     .claude/skills/intrada-shipping/SKILL.md. Keep these headings. -->

Look at: <!-- One line: the decision or risk Jon should judge before merging. -->

## What this fixes

<!-- What the affected person would notice, in the first sentence; then one
     sentence on why this approach. No paths, symbols or code. -->

## Where this could bite

<!-- Tier 3 and the sensitive surfaces only; delete it otherwise. One
     paragraph per residual risk, each ending in its issue or "deliberate, not
     tracked". Never spell out an exploitable gap on a public repo. -->

## What it looks like

<!-- Screen changes only: paste `just pr-visuals` output, run after the push.
     Delete it otherwise. -->

## What I checked

<!-- The evidence each check produced; "gates green" is one line with counts. -->

Coverage: <!-- Tier 2+: the expected patch-coverage gaps and why. Tier 1: n/a. -->

## What changed where

<!-- Tier 3 and the sensitive surfaces only; delete it otherwise. At most
     five lines: the files worth opening first, each with why. -->

## Checklist

<!-- Only what CI cannot prove: the gates, formatting and snapshots run in
     CI. Delete a line that does not apply. A line left unticked is still
     owed. -->

- [ ] Roadmap item or issue: #___ (or explicitly agreed with Jon)
- [ ] One PR across the stack: core, iOS and Android together, core commits first (on the sensitive surfaces, `reviewer` read the core commits alone before the screens commits)
- [ ] A screen change carries the `just android-run` screenshot beside the iPhone one
- [ ] New UI uses `Intrada*` tokens (colour, spacing, radius, type), no raw literals
- [ ] Persistence or new-entity changes: the offline-first checklist in `.claude/rules/offline-first.md` applied
- [ ] CLAUDE.md updated (if architecture, components or patterns changed)
- [ ] Roadmap updated (if a feature is now complete or scope changed)
- [ ] Deferred items opened as tracked issues and listed in the self-review comment
