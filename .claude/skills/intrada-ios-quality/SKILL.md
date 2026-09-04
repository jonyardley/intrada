---
name: intrada-ios-quality
description: "The per-screen quality bar for the native iOS app: the 2026-06 review principles (surface don't swallow, enforce stated invariants, sync-boundary discipline, consolidate before you template, bridge round-trip tests) and snapshot-test hygiene (one device+scale, what to snapshot, re-recording with just ios-snapshots-record, the size-ceiling allowlist, no orphans). MUST read before adding or changing a SwiftUI screen, or before touching anything under ios/IntradaTests/__Snapshots__."
---

## Principles (from the 2026-06 review)

Hard-won lessons from the first full review of the native app. **Treat them like
the non-negotiables under CLAUDE.md § Native iOS Shell (SwiftUI + Crux).**

- **Surface, don't swallow — at every layer.** A core error state with no UI
  surface is the silent-no-op bug (#846) one level up. Every `ViewModel.error`
  must have a UI surface, and optimistic UI must reconcile with the core's
  confirmed outcome — never fire a success haptic or dismiss a sheet before the
  core confirms (re-read `viewModel.error` after `store.send`; see
  `LibraryAddScreen.add`).
- **A stated invariant must be an enforced invariant.** Back each offline-first
  invariant with a test *and* a CI gate. Prose plus an opt-in local hook is
  effectively off — assume the hook isn't installed.
- **Sync-boundary discipline now, not at Phase D.** Both writers (shell + core)
  must agree on the timestamp format, and reconciliation/merge policy lives in
  the core, even before the sync engine exists. Don't encode merge rules in
  shell SQL.
- **Consolidate before you template.** Extract a shared primitive or form the
  moment a *second* screen would copy it. The Library screens are the template
  the other pillars will clone — duplication here multiplies.
- **Bridge-crossing types need a real round-trip test as a build precondition.**
  Extend the Rust `assert_round_trips` helper to every write payload *before* it
  is wired to a screen — a stub-bridge test can't catch a bincode-wire break
  (#846).

## Snapshot test hygiene

Snapshot references (`ios/IntradaTests/__Snapshots__/**/*.png`) are binaries
committed to git and re-recorded on every intentional UI change, so each
re-record adds a full copy to history forever. On the free offline tier they are
the only UI quality gate, so keep the suite but keep it lean:

- **One device + scale, deterministic host.** Pin `.iPhone13` + `displayScale`,
  force light mode at the controller, use the stub bridge. Do **not** multiply
  references by device/theme/size-class — snapshot a variant only when it can
  independently regress.
- **Snapshot load-bearing states, not the cross-product.** Prefer
  component-level (`sizeThatFits`) or structural/text snapshots where the
  assertion isn't pixel-perfect.
- **Re-record with `just ios-snapshots-record <filter>`**, not by hand. It
  deletes the matching references, runs only those tests (recording is
  fail-then-pass by design), optimises what it wrote and re-checks hygiene —
  one command instead of five, and scoped, so it is seconds rather than the
  whole suite.
- **Optimize before committing.** `ios-snapshots-record` does it; if you record
  another way, run `just ios-snapshots-optimize`. CI's **Snapshot Hygiene** job
  enforces a per-file size ceiling and fails on un-optimized references.
- **Over the ceiling? Read `scripts/check-snapshots.sh` before reacting.** It
  carries an allowlist for references that stay large as lossless PNG — the
  smooth gradients (practice hero, focus-player radial) and dense-control screens.
  **Cropping does not help those**: flat paper costs almost nothing and the
  gradient is the whole bill. Add to the allowlist with the reason, and keep the
  list tight.
- **No orphans.** Delete a test → delete its PNG. The same job fails any
  reference with no matching `func test…`. Check locally with
  `just ios-snapshots-check`.

