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

- #1355 — **a photo on a piece**, phase A of
  [`specs/piece-from-photo.md`](../specs/piece-from-photo.md) (#1439). One image
  per item as an aide-memoire: the page in front of you, kept on the device,
  shown on the item. No recognition of any kind in this phase, and it is useful
  alone. The **core PR** carries `Item.photo_id`, `SetPhoto`/`ClearPhoto`, the
  `ViewModel` projection the screens will read, migration v16 and the GRDB
  codec. Phase A never deletes a photo file — replacing, clearing and deleting
  the item all leave it on disk for the reaping pass (#1442), because an orphan
  costs disk and an eager delete costs the user their photo. Capture and display
  design, then the screens PR, follow.
- #1420, the tempo trend, was the last of the adopted order's numbered
  steps with anything to construct: steps 1 to 8 are done and steps 9 to 11 are
  each a fresh decision gated on lived use, not queued work. What *is* running
  is the real-use thread under **Next** (the week's lesson tune), which is where
  the next build should come from.

## Recently landed

- #1445 — **the exercise picker no longer orders exercises differently from the
  Library**. Sorting by **Last practised** put exercises that have never been
  practised in one order on the Library screen and another in the **Add
  exercises** sheet, because the sheet sorts its own candidates and had no rule
  for what happens when the sort key ties. It now uses the core's: newest
  first, then id. Same class as the search divergence in #1440, one function
  below it.
- #1440 — **search means the same thing in the exercise picker as in the
  Library**. Choosing an exercise for a piece (**Choose one** on the piece)
  opens a sheet whose search only read the exercise's title and its key/tempo
  line, so a composer name, a note or a tag found nothing there while the same
  words worked everywhere else. It now matches the fields the Library matches,
  and no longer matches the key/tempo line, which the Library never did.
  The two add sheets in Build session also put the Library's own filter back
  when they close, instead of leaving it as the sheet left it. The deeper
  tidy-up was weighed and left: the two pickers are still built differently,
  one on the shared query and one on its own state, and unifying them costs a
  morning for nothing a musician can see.
- #1103 — **the related-exercise picker now behaves like the add-to-session
  sheet**. Adding three exercises to a block used to mean opening the sheet
  three times, and a long exercise list had no search in it. The sheet now
  stays open, each row shows whether it is in the block (tap again to take it
  back out), and it carries the same browse bar as "Add to session" — search,
  sort and tag filter, minus the piece/exercise menu, since the sheet is
  exercises only. The added-state row treatment is now one shared component
  across both sheets.
- #1420 — **the tempo trend**, step 7 of the adopted order
  ([`research/comparison.md`](research/comparison.md)), in three PRs. The
  **chart** (#1425) draws an item's measured tempo over time on the detail
  screen: a line across the sessions that measured one, breaking wherever a
  session measured none, with `7 of 9 sessions measured` underneath and a plain
  line rather than a plot where there is less than a trend to show. Its rulings
  are design-principles **T17**, including the two worth knowing: the item's
  declared target is not drawn on the plot (the card sits directly under the
  Key/Tempo rows, which already carries it), and the vertical scale never spans
  less than 8 BPM, so a two-beat drift cannot draw the same climb as a
  thirty-beat one. Mock:
  [Tempo Trend](https://claude.ai/code/artifact/b0aff8b0-b9e7-45bb-a209-fe8bb866f6fd).

  Before it, the **evidence contract** (#1422):
  tempo capture (#1398) put the stepper on every item and always saved a number
  pre-filled from the click, so a large share of the recorded tempos were an
  untouched default nobody looked at, and for an item declaring a target that
  default *was* the target, so the app was quietly recording it as achieved. A
  tempo is now recorded only when it was measured (design-principles **T16**,
  answering roadmap Q3): the shell forwards two observed facts and the core
  rules on whether they amount to evidence. No new field, so no crash-recovery
  key bump. The **series** (#1423): `ItemPracticeSummary.tempo_history` is
  replaced by `tempo_trend`, one point per practised session oldest first with
  `tempo: None` where nothing was measured, so the gaps the chart draws have a
  position. What counts as a trend (two measured tempos or more) is the core's
  ruling, so the screen never has to make it.
- #1416 — **the getting-cold signal**, step 8 of the adopted order
  ([`research/comparison.md`](research/comparison.md)). Spec at
  [`specs/getting-cold-signal.md`](../specs/getting-cold-signal.md). How long an
  item has sat is now measured against the return interval its own history
  earns it, so a piece marked 3 of 10 and played once and a piece marked 9 of 10
  across twenty returns no longer go cold on the same clock, which is exactly
  what the binary 14-day flag got wrong. The Up next card's headline reason says
  the grade in words (`going cold after 3 weeks`, `cold for 3 months`) and its
  ranking moved to the same graded number, so the headline still says the thing
  the ranking used. `compute_neglected_items` reads the estimate instead of its
  own 14-day cut, so the core has one opinion about coldness rather than two.
  Core only: it arrived through the reason strings the card spec made
  core-owned, with no ViewModel field, no bindings change and no Swift logic.
  Nothing new gates anything. Whether the thresholds are right is empirical,
  so use it for a few weeks before tuning them (#1419). A coldness clause on
  the card's item rows was held back deliberately (#1418).
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

- Steps 9 to 11 of the adopted order remain the big bets, each a fresh decision
  gated on lived use.
- Running alongside, and not a build: **use** the Up next card and the
  getting-cold signal on the week's lesson tune and file the friction, the way
  step 2 and step 5 were treated. Two weeks of real use is what tells us
  whether the staleness thresholds are anywhere near right (#1419), and there
  is no substitute for it. A coldness clause on the card's item rows is held
  back deliberately (#1418).
- Real use has started (Like Someone in Love, 2026-08-14) and filed its first
  findings: #1390 (add the chord chart and related exercises while creating the
  item, not in a second trip) and a parser friction note on #1387. #1390 is
  design-first.
- Real use has filed a second round (2026-08-28, piece page). **#1431 has
  landed its core half** (#1434: `AddLinkedExercise`, so an exercise written on
  a piece saves already linked) and its screens are in flight: the piece page
  now leads with **Create an exercise**, the picker is the secondary path, and
  the chart's five are suggestions rather than a curriculum. Rulings are
  design-principles **T18**; spec and mockups in
  [`specs/piece-related-exercises.md`](../specs/piece-related-exercises.md).
  **Showing the chord chart properly** (#1432, next) is parked for a design
  conversation, but its first finding is concrete: the piece-page preview does
  not draw bars, so any bar holding two chords pushes the rest of the grid out
  of line. Adding several exercises to a grouped piece one sheet at a time
  (#1103) moved to now.
- Follow-up: port the wire-pin test technique (per-variant bincode
  fingerprint) to the `ActiveSession` crash-recovery blob (#1345).
- **Adding a piece from a photo is designed, not built**
  ([`specs/piece-from-photo.md`](../specs/piece-from-photo.md)). Four phases:
  #1355 stores the photo and does no recognition; #1436 reads title, composer
  and tempo with Vision OCR through a new `RecognitionOperation` effect; #1437
  adds on-device model suggestions, clamped so the model may only choose text
  that appears in the OCR; #1387 part 2 reads a photographed *text* chart.
  Phase A is useful alone and is the entry point. The load-bearing finding:
  chord symbols on a stave are text so OCR reads them, but barlines are
  graphics, so a lead sheet gives no bar structure and `parse_chart` needs bars
  above all else. That half is a spike (#1438), explicitly not a phase.
