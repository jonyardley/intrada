# Status — what's in flight now

*One screen: current phase, what's in flight, what just landed, what's next.
Updated in every PR that changes scope or state (see CLAUDE.md → After
completing work). Direction and phases live in [`roadmap.md`](roadmap.md); the
scope and timing detail on the
[project board](https://github.com/users/jonyardley/projects/2). When this
doc and the issues disagree, the issues are right. For the last-updated date,
ask git: `git log -1 --format=%cs docs/status.md`.*

## Where we are

**v0.7.0, the restored session builder, is on TestFlight (2026-08-14).**
The coach revert (#1344) landed clean: gates green, remnant sweep clean,
docs reconciled (#1350). Phase R ([`rethink-plan.md`](rethink-plan.md))
Stage 1 is complete: Jon's on-device audit of v0.7.0 covered every screen,
the triage closed 30 defunct coach-era issues and emptied the
`superseded-by-pivot` label, and the synthesis is
[`audit-2026-08.md`](audit-2026-08.md) — the definitive reference for the
audit backlog and its five-phase build order.

## In flight

Nothing. The next unstarted item is step 7 of the adopted order — see **Next**.

## Recently landed

- #1082 (#1409, #1414) — **the Up next card**, step 6 of the adopted order
  ([`research/comparison.md`](research/comparison.md)). Tier 3, spec at
  [`specs/up-next-card.md`](../specs/up-next-card.md), shipped core-first then
  screens. The core derives one block worth resuming from the same
  `LibraryItemView` projection the Library screens read, ranks it on named keys
  rather than a weighted score so every reason it gives is sayable, and owns
  every string the card shows. The Practice hero then carries that suggestion
  instead of a play glyph (design-principles **T15**): the piece, why, its two
  or three items with their own reasons, and one `Start · N min`, with "Build
  my own instead" revealing exactly the hero that shipped. Held to
  suggest-never-gate throughout — with nothing to suggest, the tab is
  unchanged. The design system's canonical Practice screen was folded in with
  the screens, which also cleared the retired coach-era "Today's plan" frame it
  was still showing. Deferred: #1411, #1412, #1413.

- #1081 — per-piece tracking **was already shipped**, back in July, and the
  issue was simply never closed. Verified 2026-08-20 against the whole of its
  stated scope: the core derivation (`build_exercise_contexts`, #1095), the
  "By piece" rows and the per-this-piece rings on piece detail (#1097), the
  FFI round-trip guard
  (`setlist_entry_group_id_round_trips_on_ffi_bincode_wire`), the
  `testExerciseDetailByPiece` snapshot, and the design mock committed under
  `specs/track-exercises-per-piece/`. It came through the coach revert intact:
  `AddToSetlist` still forms blocks, and `group_id` still round-trips the
  local-first JSON codec, so the feature works with no network. No Tier 3 spec
  and no core PR were written, because there was nothing left to build. This
  is the same stale-issue pattern as #1106 in step 1 of the adopted order:
  built during the coach era, frozen behind the pivot, never closed. **It has
  not been used in anger**, which is the honest next step for it: the same
  treatment step 2 gave the chord-chart flow.

- #1404 (#1406) — tempo capture: the end-of-item stepper pre-fills from the
  click's last tempo and shows even when the item declares no target. Step 4
  of the adopted order ([`research/comparison.md`](research/comparison.md)),
  following on from the click (#1398). iOS-only; no core change was needed —
  `UpdateEntryTempo` already accepted an undeclared-target entry.

- #1398 (#1401) — the metronome click is back in the Focus Player, step 3 of
  the adopted order ([`research/comparison.md`](research/comparison.md)). The
  player's tempo line now doubles as the control: one tap sounds a click at the
  item's target, and the slower/faster steppers appear only while it sounds
  (design-principles T14). The engine is recovered from `8af4891^` and trimmed
  to a plain click, keeping the host-time grid, the stranded-clock guard and
  the interruption handling. iOS-only: no schema, no bridge, no core change.
  Deferred: background audio (#1399), the design reference (#1400), the tempo
  constants' home (#1402), accessibility-label tests (#1403). **The click has
  not been heard on a device yet** — no automated check can hear it.

- Phase 2 (say it right) is **closed**. #1360 (#1383) repurposed the Practice
  hero to the last session plus its one-tap start and moved the header subline
  off the lifetime session count, both reading one core projection
  (`compute_last_practised`) so the day turns over on the user's clock.
- #1361 (#1388) — the create form names what a piece and an exercise are, in a
  line under the pills. It rides on `KindSegment`, so the edit form carries it
  too, and the wording is a worked example in
  [`tone-of-voice.md`](tone-of-voice.md). Review fixes landed as #1392.
- #1359 step 2 — the copy sweep over every user-facing string in `ios/`
  (#1380), applying V1 to V4. Practice's hero and subline were left to #1360
  by design.
- Stage 4.2 of Phase R **decided** (Jon, 2026-08-14):
  [`research/comparison.md`](research/comparison.md) compares the five
  research notes and carries the adopted order, quick wins first: fix the
  stale chord-chart issues (done: #1106 closed as shipped, #1107 re-scoped to
  twelve-key generation), use the chord-chart flow for real, the metronome
  click pulled forward, then per-piece tracking (#1081) and the Up next card
  (#1082), then the getting-cold signal, with the twice-deleted admin-shaped
  features (quick lesson entry, goals) last and gated on use.
- Plain-language rule for docs and issues (Jon, 2026-08-14): name features by
  the musician-visible outcome, issue numbers as the only stable handles.
  Rule in CLAUDE.md → Conventions, glossary in
  [`reference.md`](reference.md); older docs renamed as touched.
- Stage 4.1 of Phase R: five research notes under
  [`docs/research/`](research/) — goals-rebuilt-small, the Space layer,
  lesson-to-mastery (#1087), chart-to-scaffold Phase B (#1106), metronome
  (#1366). Pedagogy evidence plus repo archaeology, one note per candidate;
  Stage 4.2 (the comparison, Jon picks) is next. Headline finding:
  chart-to-scaffold Phase B already shipped (#1110, 2026-07-17) and #1106 is
  stale; only the twelve-key ladder wiring (#1107) remains.
- #1359 step 1 — [`tone-of-voice.md`](tone-of-voice.md) written and red-penned
  (#1376). Jon's four rulings are in the doc as V1 to V4; the worked examples
  carry the chosen option. Step 2 (the copy sweep) is unblocked.
- Audit Phase 1 closes with #1357 + the UTC day-boundary class (#1330, #1346):
  the Practice screen's dot-vs-Today-list pair was proven consistent (both
  local-time, one classification function). The real bugs were the week
  header counting all-time practice days as "This week", now scoped to the
  visible week, and core analytics turning the day over at midnight UTC, fixed
  by lifting the device's UTC offset into `Model` (`SetUtcOffset`, reported at
  launch and on foreground) and deriving every analytics day via `LocalClock`.
  #1356 (pill animation) validated clean on the stable simulator: the jank is
  an iOS 27 beta artefact, noted on the issue.
- v0.7.0 tagged and uploaded to TestFlight (version bump #1351).
- #1352 — the Phase R rethink plan.
- Stage 1 of Phase R: audit, triage and synthesis
  ([`audit-2026-08.md`](audit-2026-08.md)); findings filed as #1354–#1371.
- Audit Phase 1: #1354 (Library scroll-triggered fade removed) and #1365
  (live-view tempo font bumped within the `IntradaFont` scale).
- Audit Phase 1: #1368 (Session Complete celebration animation and the
  Improved/Rough/Next-target reflect trio removed — the core `SummaryView`
  reflection fields and `UpdateSessionReflection` event stay, now shell-dead
  like `Set`/#1348, tracked as its own follow-up) and #1358 (Practice's
  "Build a custom session" footer entry point removed — the hero's one-tap
  start already reaches the same builder via the same `StartBuilding` event,
  so this was a duplicate entry point, not a distinct capability).

## Next

- The adopted Stage 4 order ([`research/comparison.md`](research/comparison.md))
  is clear through step 6. **Step 7, the tempo trend, is gated**: it is blocked
  on roadmap Q3 (does a mastery score mean anything without its tempo?) being
  answered, so the honest next build is **step 8, the getting-cold signal** —
  a graded staleness estimate replacing the binary 14-day flag, landing as new
  Up next reason lines rather than a surface of its own. The reason strings are
  the seam it arrives through, which is why they are core-owned. 1–2 days.
- Real use has started (Like Someone in Love, 2026-08-14) and filed its first
  findings: #1390 (add the chord chart and related exercises while creating the
  item, not in a second trip) and a parser friction note on #1387. #1390 is
  design-first.
- Follow-up: port the wire-pin test technique (per-variant bincode
  fingerprint) to the `ActiveSession` crash-recovery blob (#1345).
