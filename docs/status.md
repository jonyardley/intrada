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

- Nothing. Audit Phase 1 (fix and trim) is complete, and the tone-of-voice
  doc (#1359 step 1) has landed; the copy sweep (#1359 step 2) is the next
  slice.
- #1376 — [`tone-of-voice.md`](tone-of-voice.md) (#1359 step 1), open, stacked
  under by the step 2 sweep PR.
- #1359 step 2 — the copy sweep against every user-facing string in `ios/`,
  applying V1 to V4. Practice's hero/subline is explicitly left for #1360
  (already tracked there, not a new deferral).
- Audit Phase 1 (fix and trim) — #1357, #1356 remain.

## Recently landed

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

- Phase 2, say it right: one sweep PR over every user-facing string (#1359
  step 2), then #1360 and #1361 with wordings the doc settles.
- Audit Phase 1, fix and trim: #1357 (Today bug), then #1356.
- Phase 2, say it right: #1360 (Practice hero/subline) and #1361 (create-form
  captions), with wordings the tone-of-voice doc already settled.
- Stage 4.2: one-page comparison of the five research notes; Jon picks the
  direction.
- Follow-up: port the wire-pin test technique (per-variant bincode
  fingerprint) to the `ActiveSession` crash-recovery blob (#1345).
