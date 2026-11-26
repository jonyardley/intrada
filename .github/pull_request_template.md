<!-- What each heading holds, and the 350-word cap on a Tier 1 or 2 body
     before the checklist: the PR bodies section of
     .claude/skills/intrada-shipping/SKILL.md. Keep these headings. -->

Closes #___

## Look at

<!-- One line: the decision or risk Jon should judge before merging. -->

Process fixes: <!-- Each retro fix with the lesson it answers; "none" when nothing went wrong. -->

## What this fixes

<!-- What the affected person would notice, then why this approach. -->

## What I checked

<!-- The evidence each check produced, screen pictures included. -->

Coverage: <!-- Tier 2+: the expected patch-coverage gaps and why. Tier 1: n/a. -->

## Checklist

<!-- Only what no check proves; delete a line that does not apply. -->

- [ ] One PR across the stack: core and iOS together, core commits first (on the sensitive surfaces, `reviewer` read the core commits alone before the screens commits)
- [ ] New UI takes radius and type from `Intrada*` tokens (CI checks colour and spacing)
- [ ] Persistence or new-entity changes: the offline-first checklist in `.claude/rules/offline-first.md` applied
- [ ] CLAUDE.md updated (if architecture, components or patterns changed)
- [ ] Roadmap updated (if a feature is now complete or scope changed)
- [ ] Each out-of-scope finding fixed or listed under "Not fixed"; only horizon now or next opened as an issue
