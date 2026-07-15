## Summary

<!-- What does this PR do? 1-3 sentences. -->

## Roadmap alignment

- [ ] Links to a roadmap item in [`docs/roadmap.md`](../docs/roadmap.md): issue #___ (or explicitly agreed with Jon)
- [ ] Pillar: Plan / Practice / Track / Cross-cutting
- [ ] Horizon: Now / Next / Later

## Coverage

Coverage: <!-- Tier 2+: "full: <what the new tests cover>" or the expected patch-coverage gaps and why. Tier 1: n/a. -->

## Checklist

- [ ] `just check` passes (fmt + clippy + tests, mirrors CI's flags)
- [ ] `ios/` changes: `just ios-fmt-check` + `just ios-test` pass; snapshots re-recorded + `just ios-snapshots-optimize` if UI changed
- [ ] Persistence / new-entity changes: offline-first PR checklist in CLAUDE.md applied (`updated_at`/`deleted_at`, client-minted ulid, `local_first` branches tested both ways)
- [ ] CLAUDE.md updated (if architecture, components, or patterns changed)
- [ ] Roadmap updated (if a feature is now complete or scope changed)
