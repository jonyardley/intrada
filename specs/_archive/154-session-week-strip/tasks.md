# Tasks: Session Week Strip Navigator

**Input**: Design documents from `/specs/154-session-week-strip/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Unit tests included for date helper functions (pure logic). No E2E test tasks — those are deferred to manual quickstart validation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- All paths are relative to repository root

---

## Phase 1: Setup

**Purpose**: Date utility functions and shared infrastructure that all user stories depend on

- [x] T001 [P] Add week calculation helpers (get_week_start, get_week_dates, get_month_label) in `crates/intrada-web/src/helpers.rs` — use chrono NaiveDate, Datelike, IsoWeek following patterns from `intrada-core/src/analytics.rs`. Include: `get_week_start(today: NaiveDate, offset: i32) -> NaiveDate`, `get_week_dates(week_start: NaiveDate) -> [NaiveDate; 7]`, `get_month_label(week_start: NaiveDate, week_end: NaiveDate) -> String` (single month or "Feb – Mar 2026" format), `day_abbrev(weekday: Weekday) -> &'static str` returning single letters M/T/W/T/F/S/S.
- [x] T002 [P] Add session date grouping helpers in `crates/intrada-web/src/helpers.rs` — `group_sessions_by_date(sessions: &[PracticeSessionView]) -> HashMap<NaiveDate, Vec<PracticeSessionView>>` parsing each `started_at` RFC3339 string to NaiveDate. Within each date bucket, sort chronologically (earliest first per FR-005). Also add `sessions_for_week(grouped: &HashMap<NaiveDate, Vec<PracticeSessionView>>, week_start: NaiveDate) -> HashSet<NaiveDate>` returning dates in the week that have sessions.
- [x] T003 [P] Add unit tests for week calculation helpers in `crates/intrada-web/src/helpers.rs` `#[cfg(test)]` module — test: week start calculation for various offsets, month label with single month, month label spanning two months, day abbreviation for all 7 weekdays, edge case at year boundary (e.g. week spanning Dec/Jan).
- [x] T004 [P] Add unit tests for session grouping helpers in `crates/intrada-web/src/helpers.rs` `#[cfg(test)]` module — test: empty sessions list, sessions grouped correctly by date, chronological sort within each day, sessions_for_week returns correct date set, sessions with UTC timestamps grouped by naive date.

**Checkpoint**: Date utilities tested and ready. Run `cargo test -p intrada-web` to confirm.

---

## Phase 2: Foundational (New Components)

**Purpose**: WeekStrip and DayCell components that all user stories render through

**⚠️ CRITICAL**: User story views depend on these components existing

- [x] T005 Create DayCell component in `crates/intrada-web/src/components/week_strip.rs` — a Leptos component rendering a single day cell: abbreviated day name (M/T/W/T/F/S/S), date number (1–31), selected state highlight (use `bg-accent-focus/20` + `text-accent-text` ring for selected, default `text-secondary`), session indicator dot (small `bg-accent-text` circle below date when `has_sessions` is true). Props: `date: NaiveDate`, `day_abbrev: &'static str`, `is_selected: bool`, `has_sessions: bool`, `on_click: Callback<NaiveDate>`. Use design tokens only — no raw Tailwind colours. Add ARIA: `role="button"`, `aria-label` with full date, `aria-pressed` for selected state.
- [x] T006 Create WeekStrip component in `crates/intrada-web/src/components/week_strip.rs` — a Leptos component rendering the full week strip: month/year label centred between left (◄) and right (►) arrow buttons, row of 7 DayCell components. Props: `week_start: NaiveDate`, `selected_date: Option<NaiveDate>`, `session_dates: HashSet<NaiveDate>`, `on_day_click: Callback<NaiveDate>`, `on_prev_week: Callback<()>`, `on_next_week: Callback<()>`. Use `get_month_label` and `get_week_dates` from helpers. Arrow buttons use Icon component with chevron-left/chevron-right (or text ◄/►). Layout: `Card` component as container, flex-row for day cells with even distribution. Responsive: full-width on mobile, content-width on desktop.
- [x] T007 Register week_strip module in `crates/intrada-web/src/components/mod.rs` — add `pub mod week_strip;` and re-export `pub use week_strip::{WeekStrip, DayCell};`.

**Checkpoint**: Components compile. Run `cargo check -p intrada-web` to confirm.

---

## Phase 3: User Story 1 — Browse Sessions by Week (Priority: P1) 🎯 MVP

**Goal**: Replace the flat session list with a week strip showing the current week, dot indicators for days with sessions, and auto-selection of today (or most recent session day).

**Independent Test**: Navigate to `/sessions` → see week strip with correct dates for the current week → dots on days with sessions → today (or recent session day) auto-selected → session cards for the selected day displayed below.

### Implementation for User Story 1

- [x] T008 [US1] Refactor `SessionsListView` in `crates/intrada-web/src/views/sessions.rs` — replace the flat session list with the week strip view. Add Leptos signals: `week_offset: RwSignal<i32>` (init 0), `selected_date: RwSignal<Option<NaiveDate>>` (init None). Compute derived signals: `week_start` from `week_offset` using `get_week_start(today, offset)`, `grouped_sessions` from `ViewModel.sessions` using `group_sessions_by_date`, `session_dates_in_week` using `sessions_for_week`. Render: PageHeading + "New Session" link (preserved from existing), WeekStrip component with computed props, session cards area below strip.
- [x] T009 [US1] Implement auto-select logic in `crates/intrada-web/src/views/sessions.rs` — create a derived signal or effect that runs when `selected_date` is None or when `week_offset` changes. Logic: (1) if today is in displayed week and has sessions → select today, (2) else find most recent day in week with sessions → select it, (3) else select today if in week, else Monday. Set `selected_date` when week changes (reset to None to trigger re-auto-select).
- [x] T010 [US1] Render session cards for selected day in `crates/intrada-web/src/views/sessions.rs` — below the WeekStrip, render a reactive block: when `selected_date` is Some and has sessions in `grouped_sessions`, map those sessions to `SessionRow` components (reuse existing `SessionRow` from the same file). When selected day has no sessions, show empty state: `<p class="empty-text">"No sessions on this day"</p>`. Preserve the existing `SessionRow` component unchanged (it handles card content + delete).
- [x] T011 [US1] Add loading skeleton for session cards in `crates/intrada-web/src/views/sessions.rs` — when `is_loading` is true, render `SkeletonCardList` (existing component) below the week strip instead of session cards. The week strip itself should render immediately (dates are computed locally, not from API).

**Checkpoint**: User Story 1 complete. Navigate to `/sessions` → week strip visible → dots on session days → auto-select works → session cards display for selected day. Run `cargo test -p intrada-web` and `cargo clippy -- -D warnings`.

---

## Phase 4: User Story 2 — Select a Day to View Sessions (Priority: P1)

**Goal**: Tapping a day in the week strip updates the selected day and displays that day's session cards. Tapping a session card navigates to the session detail page.

**Independent Test**: Click different days in the strip → session cards update → click a session card → navigates to session detail.

### Implementation for User Story 2

- [x] T012 [US2] Wire day selection click handler in `crates/intrada-web/src/views/sessions.rs` — the `on_day_click` callback passed to WeekStrip should set `selected_date` to `Some(clicked_date)`. The session cards area already reactively depends on `selected_date` from US1, so cards update automatically.
- [x] T013 [US2] Wrap each session card with a link to session detail in `crates/intrada-web/src/views/sessions.rs` — wrap the `SessionRow` component (or its outer Card) with an `<A href={format!("/sessions/{}", session.id)}>` using leptos_router's `A` component. Ensure the delete button click does not propagate to the link (use `stop_propagation` or structure the link to exclude the delete action area). If there is no existing session detail route, use the session summary route or confirm the existing route pattern.
- [x] T014 [US2] Add visual selected state styling to DayCell in `crates/intrada-web/src/components/week_strip.rs` — ensure the selected day has a clearly distinct visual treatment: background highlight ring (e.g. `ring-2 ring-accent-focus rounded-lg bg-accent-focus/10`), text colour `text-accent-text` for the date number. Unselected days use `text-secondary`. Hover state: `hover:bg-surface-hover` on unselected days. Ensure cursor is `cursor-pointer` on all day cells.

**Checkpoint**: User Story 2 complete. Click days → cards update → click card → navigates to detail. Visual selection is clear.

---

## Phase 5: User Story 3 — Navigate Between Weeks (Priority: P2)

**Goal**: Arrow buttons and mobile swipe gestures navigate to previous/next weeks. The strip updates with new dates and session indicators. First day with sessions is auto-selected.

**Independent Test**: Click left arrow → previous week dates shown → dots correct → auto-select works. Swipe on mobile → week changes.

### Implementation for User Story 3

- [x] T015 [US3] Wire arrow button handlers in `crates/intrada-web/src/views/sessions.rs` — `on_prev_week` callback decrements `week_offset` by 1 and resets `selected_date` to None (triggering auto-select). `on_next_week` increments by 1 and resets. The WeekStrip and session cards re-derive automatically from the changed signals.
- [x] T016 [US3] Add loading skeleton during week transition in `crates/intrada-web/src/views/sessions.rs` — when `week_offset` changes, briefly show `SkeletonCardList` in the session cards area while the new week's sessions are computed. Since filtering is client-side and near-instant, this may be imperceptible — but implement it as a short signal-based transition (set a `transitioning` signal to true on week change, use `set_timeout` to set it false after ~100ms, show skeleton while true). Per FR-010: "strip updates immediately; session cards area shows brief loading skeleton."
- [x] T017 [US3] Implement swipe gesture detection on WeekStrip in `crates/intrada-web/src/components/week_strip.rs` — add `on:pointerdown` and `on:pointerup` event handlers on the strip container. On pointerdown: store `(client_x, client_y)` in a signal. On pointerup: calculate deltaX = up.client_x - down.client_x. If `|deltaX| > 50 && |deltaX| > |deltaY|`: deltaX < 0 means swipe left → call `on_next_week`, deltaX > 0 means swipe right → call `on_prev_week`. Use existing `web_sys::PointerEvent` (already in web-sys features). Add `touch-action: pan-y` CSS on the strip container to allow vertical scrolling while capturing horizontal swipes.
- [x] T018 [US3] Add unit tests for auto-select on week navigation in `crates/intrada-web/src/helpers.rs` `#[cfg(test)]` — add helper function `auto_select_day(week_start: NaiveDate, today: NaiveDate, session_dates: &HashSet<NaiveDate>) -> NaiveDate` that encapsulates the auto-select logic as a pure function. Test: today in week with sessions → today, today in week without sessions but Wednesday has → Wednesday, no sessions in week → Monday (if today not in week), today if today in week but no sessions.

**Checkpoint**: User Story 3 complete. Arrows navigate weeks → dates update → dots correct → auto-select fires. Swipe works on mobile/touch.

---

## Phase 6: User Story 4 — Access Full Session List (Priority: P3)

**Goal**: A "Show all sessions" link below the week view navigates to the full chronological session list.

**Independent Test**: Click "Show all sessions" → full list appears → can return to week view.

### Implementation for User Story 4

- [x] T019 [US4] Create `SessionsAllView` in `crates/intrada-web/src/views/sessions_all.rs` — extract the existing flat session list rendering into this new view. Reuse `SessionRow` component (import from sessions module or make it public). Include PageHeading ("All Practice Sessions"), the full `vm.sessions` list rendered with `SessionRow` cards, session count footer, and a `BackLink` to `/sessions` ("Back to week view"). Keep the empty state message for no sessions.
- [x] T020 [US4] Register sessions_all view in `crates/intrada-web/src/views/mod.rs` — add `pub mod sessions_all;` and `pub use sessions_all::SessionsAllView;`.
- [x] T021 [US4] Add `/sessions/all` route in `crates/intrada-web/src/app.rs` — add route `<Route path=path!("/sessions/all") view=move || view! { <SessionsAllView /> } />`. Place it BEFORE the `/sessions/:id` pattern routes (if any exist) to avoid path conflicts. Import `SessionsAllView` in the use statement at the top.
- [x] T022 [US4] Add "Show all sessions" link in `crates/intrada-web/src/views/sessions.rs` — below the session cards area (after the session list or empty state), add `<A href="/sessions/all" attr:class="action-link text-muted hover:text-accent-text mt-4 block text-center">"Show all sessions →"</A>`.

**Checkpoint**: User Story 4 complete. Link visible → navigates to full list → BackLink returns to week view.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Design system compliance, accessibility, and final quality pass

- [x] T023 [P] Make `SessionRow` component public for reuse — if `SessionRow` is currently a private `fn` in `sessions.rs`, either make it `pub(crate)` or extract it to a shared location so `sessions_all.rs` can import it. Alternatively, move it to `components/session_row.rs` if it would benefit other views. Ensure delete functionality still works in both views.
- [x] T024 [P] Add week strip utility classes to `crates/intrada-web/input.css` if needed — review whether any repeated styling patterns in WeekStrip/DayCell should be extracted into `@utility` classes. Examples: `day-cell` base styling, `day-cell-selected` highlight, `session-dot` indicator. Follow the design system rule: "If a styling pattern appears in 2+ places, create a @utility."
- [x] T025 [P] Add WeekStrip and DayCell to design catalogue in `crates/intrada-web/src/views/design_catalogue.rs` — add a showcase section for the week strip component with sample states: empty week, week with sessions on 3 days, selected day, dual-month label.
- [x] T026 Verify responsive behaviour — test at mobile (<640px) and desktop (≥640px) breakpoints. Week strip should span full width on mobile with evenly distributed day cells. Session cards should be full-width on mobile. Arrow buttons always visible. Swipe functional on touch devices.
- [x] T027 Run `cargo fmt --check && cargo clippy -- -D warnings && cargo test` — all must pass.
- [x] T028 Run quickstart.md verification steps (V1–V12) — validate all 12 verification scenarios from `specs/154-session-week-strip/quickstart.md`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on T001, T002 (helpers must exist for components to compile)
- **User Story 1 (Phase 3)**: Depends on Phase 2 (WeekStrip + DayCell components)
- **User Story 2 (Phase 4)**: Depends on Phase 3 (session cards must render to add click-to-detail)
- **User Story 3 (Phase 5)**: Depends on Phase 3 (week strip must render to add navigation)
- **User Story 4 (Phase 6)**: Depends on Phase 3 (need SessionRow to be reusable)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 — core MVP, no other story dependencies
- **User Story 2 (P1)**: Depends on US1 (needs session cards to be rendered to add click handling)
- **User Story 3 (P2)**: Depends on US1 (needs week strip rendering to add navigation). Can run in parallel with US2 (independent concerns: day click vs week navigation)
- **User Story 4 (P3)**: Depends on US1 (needs SessionRow to exist). Can run in parallel with US2 and US3

### Within Each User Story

- Helpers/models before component logic
- Component rendering before interaction wiring
- Core implementation before polish

### Parallel Opportunities

- **Phase 1**: T001, T002, T003, T004 are all parallelisable (different functions, same file but independent sections)
- **Phase 2**: T005 and T006 are sequential (DayCell before WeekStrip); T007 after both
- **Phase 3–6**: US3 and US4 can run in parallel after US1 completes
- **Phase 7**: T023, T024, T025 are all parallelisable

---

## Parallel Example: Phase 1 (Setup)

```bash
# All four tasks touch different helper functions — run in parallel:
Task T001: "Week calculation helpers in helpers.rs"
Task T002: "Session date grouping helpers in helpers.rs"
Task T003: "Unit tests for week calculation"
Task T004: "Unit tests for session grouping"
```

## Parallel Example: After User Story 1

```bash
# US3 and US4 can run in parallel (independent concerns):
Task T015-T018: "User Story 3 — Week navigation (arrows + swipe)"
Task T019-T022: "User Story 4 — Show all sessions route"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T004) — date helpers with tests
2. Complete Phase 2: Foundational (T005–T007) — WeekStrip + DayCell components
3. Complete Phase 3: User Story 1 (T008–T011) — week strip on /sessions with auto-select
4. **STOP and VALIDATE**: Navigate to `/sessions`, verify week strip renders with correct dates, dots, and auto-selection
5. Deploy/demo — the core value is delivered

### Incremental Delivery

1. Setup + Foundational → Components ready
2. User Story 1 → Week strip with current week display (MVP!) ✅
3. User Story 2 → Day selection + session detail navigation ✅
4. User Story 3 → Arrow + swipe week navigation ✅
5. User Story 4 → Full session list fallback ✅
6. Polish → Design catalogue, responsive check, full validation ✅

Each story adds value without breaking previous stories.

---

## Notes

- All new code lives in `crates/intrada-web/` only — no changes to intrada-core or intrada-api
- `SessionRow` is the existing session card component — reuse it, don't rewrite
- Week/day selection state uses Leptos signals (ephemeral UI state per architecture rules)
- Date math uses chrono NaiveDate following established patterns in analytics.rs
- Swipe uses existing PointerEvent web-sys feature — no new dependencies needed
- All colours MUST use design tokens from input.css — no raw Tailwind grays/colours
