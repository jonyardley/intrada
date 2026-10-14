---
paths:
  - "ios/**"
---

# The per-screen quality bar

Every screen ships with a snapshot test, VoiceOver labels, Dynamic Type and an
iPad `SplitView`, built with the screen rather than retrofitted. Sentry is
wired from the first build.

## What the gate checks (#1950)

- **VoiceOver labels:** a control a UI test drives carries an
  `accessibilityIdentifier` named `screen.control` (`player.skip`); repeated
  rows share one and the test picks by label, through `control(_:spoken:)`,
  which asserts the spoken label. Put the identifier on the button itself, not
  on a wrapper that ignores its children, or the tap lands on the wrapper's
  centre.
- **Not gated:** labels on controls no test drives, system controls, text
  fields' spoken labels, hints, reading order and Dynamic Type at accessibility
  sizes. Hand check those with VoiceOver on.
- **Dynamic Type:** no `.accessibility5` snapshot; #2134 is set aside, so don't
  add one. A fixed-size glyph beside text scales with `@ScaledMetric`, capped so
  the row still fits (`WeekStrip`).

## Principles from the 2026-06 review

- **Surface, don't swallow. Every `ViewModel.error` has a UI surface.** Never
  fire a success haptic or dismiss a sheet before the core confirms (re-read
  `viewModel.error` after `store.send`; see `LibraryAddScreen.add`). A send
  confirms the core accepted the event; disk work runs later, so a failed write
  arrives on the banner and the core rolls back (#2004). Saving a practice
  session waits for the disk (#974).
- **A stated invariant is an enforced invariant**: back each offline-first
  invariant with a test and a CI gate, because an opt-in local hook is
  effectively off.
- **Bridge-crossing types get a real round-trip test before a screen reads
  them**: the Rust `assert_round_trips` helper and `LiveBridge`, per
  `.claude/rules/sensitive-surfaces.md` (#846).

## Snapshot hygiene (`ios/IntradaTests/__Snapshots__`)

References are binaries in git, re-recorded on every intentional UI change,
and the only UI quality gate on the free tier. Keep the suite lean:

- **One device and scale, deterministic host.** Pin `.iPhone13` and
  `displayScale`, force light mode, use the stub bridge. Snapshot a variant
  only when it can regress independently.
- **Snapshot load-bearing states, not the cross-product.** Prefer
  component-level (`sizeThatFits`) or structural snapshots where the assertion
  is not pixel-perfect.
- **Re-record with `just ios-snapshots-record <filter>`**, never by hand: it
  records (recording is fail-then-pass by design), optimises and re-checks
  hygiene. Recorded another way? Run
  `just ios-snapshots-optimize` before committing, or CI's Snapshot Hygiene job
  fails.
- **Over the size ceiling? Read `scripts/check-snapshots.sh` first.** Its
  allowlist holds references that stay large as lossless PNG; cropping does not
  help a gradient. Add to the allowlist with the reason.
- **No orphans.** Delete a test, delete its PNG. CI fails any reference with
  no matching test; `just ios-snapshots-check` runs it locally.
