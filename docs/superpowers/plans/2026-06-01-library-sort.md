# Library Sort Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a native sort control to the library toolbar — sort by Date Added, Last Practiced, or Title (each with a direction toggle) — with the choice persisted across launches.

**Architecture:** Sorting is domain logic, so it lives in the Crux core: a new `active_sort: LibrarySort` model field drives the comparator in `view()`, and a `SetSort` event mutates it. The SwiftUI shell renders `viewModel.items` in core order and presents a native `Menu`; the chosen sort persists via the existing fire-and-forget `AppEffect` singleton path written to `UserDefaults`, restored at launch by re-dispatching `SetSort`.

**Tech Stack:** Rust / crux_core 0.18 (core), facet+UniFFI typegen (bindings), SwiftUI + `@Observable` Store (iOS shell), swift-snapshot-testing.

**Spec:** [`specs/library-sort.md`](../../../specs/library-sort.md). This is Tier 3 (core + FFI + new persisted singleton); the spec rides as the first commit on this branch.

**Pre-flight:** Confirm a roadmap item exists in `docs/roadmap.md` for the library sort control. If none, add one under the Plan pillar before starting (CLAUDE.md "Always" §1).

---

## File Structure

**Core (Rust):**
- `crates/intrada-core/src/domain/types.rs` — new `SortField`, `SortDirection`, `LibrarySort` types.
- `crates/intrada-core/src/model.rs` — `active_sort` on `Model` and `ViewModel`; `last_practiced_at` on `ItemPracticeSummary`.
- `crates/intrada-core/src/app.rs` — `SetSort` event, `AppEffect::SaveLibrarySort` variant, the sort comparator in `view()`, `last_practiced_at` in `build_practice_summaries`, and all the unit tests.
- `crates/intrada-core/src/lib.rs` — re-export the new public types.

**iOS (Swift):**
- `ios/generated/**` — regenerated bindings (never hand-edited).
- `ios/Intrada/Core/Store.swift` — write the sort to `UserDefaults` on `SaveLibrarySort`; `restorePersistedSort()`; inject `UserDefaults`.
- `ios/Intrada/Views/Components/LibrarySortMenu.swift` — **new** menu component + the `LibrarySortField` display wrapper.
- `ios/Intrada/Views/Screens/LibraryScreen.swift` — place the menu on the right of the filter row; expose an `active_sort` binding.
- `ios/Intrada/Views/RootView.swift` — call `restorePersistedSort()` at launch.
- `ios/IntradaTests/StoreEffectLoopTests.swift` — persistence write/restore tests.
- `ios/IntradaTests/ScreenSnapshotTests.swift` — snapshot of the sort control.

---

## Task 1: Core sort types

**Files:**
- Modify: `crates/intrada-core/src/domain/types.rs` (after `ListQuery`, ~line 197)
- Modify: `crates/intrada-core/src/lib.rs:21`
- Test: `crates/intrada-core/src/domain/types.rs` (existing `mod tests`)

- [ ] **Step 1: Write the failing test**

Add to the `mod tests` block in `crates/intrada-core/src/domain/types.rs`:

```rust
#[test]
fn library_sort_defaults_to_date_added_descending() {
    let sort = LibrarySort::default();
    assert_eq!(sort.field, SortField::DateAdded);
    assert_eq!(sort.direction, SortDirection::Descending);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p intrada-core library_sort_defaults`
Expected: FAIL — `cannot find type LibrarySort` / `SortField` / `SortDirection`.

- [ ] **Step 3: Write minimal implementation**

Add to `crates/intrada-core/src/domain/types.rs` (mirror the derive set used by `ListQuery` so facet typegen emits Swift types):

```rust
/// Which library column the list is ordered by.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SortField {
    #[default]
    DateAdded,
    LastPracticed,
    Title,
}

/// Ascending vs descending. Defaults to descending (newest/most-recent first).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SortDirection {
    #[default]
    Descending,
    Ascending,
}

/// The library list's sort order. Default = Date Added, newest first
/// (the historical hardcoded order).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LibrarySort {
    pub field: SortField,
    pub direction: SortDirection,
}
```

Then extend the re-export in `crates/intrada-core/src/lib.rs:21`:

```rust
pub use domain::types::{
    CreateItem, LibraryData, LibrarySort, ListQuery, SessionsData, SortDirection, SortField, Tempo,
    UpdateItem,
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p intrada-core library_sort_defaults`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/intrada-core/src/domain/types.rs crates/intrada-core/src/lib.rs
git commit -m "feat(core): add LibrarySort/SortField/SortDirection types"
```

---

## Task 2: `last_practiced_at` on the practice summary

**Files:**
- Modify: `crates/intrada-core/src/model.rs:240-249` (`ItemPracticeSummary`)
- Modify: `crates/intrada-core/src/app.rs:524-582` (`build_practice_summaries`)
- Test: `crates/intrada-core/src/app.rs` (existing `mod tests`)

- [ ] **Step 1: Write the failing test**

Add to the `mod tests` block in `crates/intrada-core/src/app.rs`. This needs a session with two entries for one item on different dates; assert the summary keeps the latest. Use the existing session-test helpers if present, otherwise build a minimal `PracticeSession` inline:

```rust
#[test]
fn summary_last_practiced_is_max_session_date() {
    use crate::model::{PracticeSession, SetlistEntry};
    let earlier = chrono::Utc::now() - chrono::Duration::days(3);
    let later = chrono::Utc::now() - chrono::Duration::days(1);

    let mk = |id: &str, started: chrono::DateTime<chrono::Utc>| PracticeSession {
        id: id.to_string(),
        started_at: started,
        completed_at: started,
        notes: None,
        session_intention: None,
        entries: vec![SetlistEntry {
            id: format!("{id}-e"),
            item_id: "item-1".to_string(),
            duration_secs: 60,
            score: None,
            achieved_tempo: None,
            ..SetlistEntry::test_default()
        }],
    };

    let summaries = build_practice_summaries(&[mk("s1", earlier), mk("s2", later)]);
    let summary = summaries.get("item-1").expect("summary for item-1");
    assert_eq!(summary.last_practiced_at, Some(later.to_rfc3339()));
}
```

> NOTE: `PracticeSession` / `SetlistEntry` field names — confirm against `model.rs` before writing; adjust the literal to match (e.g. if there is no `SetlistEntry::test_default`, spell out every field). The behaviour asserted (latest of two dates) is the contract.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p intrada-core summary_last_practiced_is_max`
Expected: FAIL — `no field last_practiced_at on ItemPracticeSummary`.

- [ ] **Step 3: Write minimal implementation**

Add the field to `ItemPracticeSummary` in `crates/intrada-core/src/model.rs` (after `tempo_history`):

```rust
    /// Most recent session date for this item (max `started_at`), independent
    /// of whether a score/tempo was recorded. `None` if never practised.
    /// RFC3339 — sorts chronologically as a string.
    pub last_practiced_at: Option<String>,
```

In `build_practice_summaries` (`crates/intrada-core/src/app.rs`), track the max date alongside the existing accumulator. Change the accumulator tuple to carry an `Option<String>` and update it per entry:

```rust
    // tuple: (count, secs, score_history, tempo_history, last_practiced_at)
    let mut acc: HashMap<
        String,
        (usize, u64, Vec<ScoreHistoryEntry>, Vec<TempoHistoryEntry>, Option<String>),
    > = HashMap::new();

    for session in sessions {
        let session_date = session.started_at.to_rfc3339();
        for entry in &session.entries {
            let record = acc
                .entry(entry.item_id.clone())
                .or_insert_with(|| (0, 0, Vec::new(), Vec::new(), None));
            record.0 += 1;
            record.1 += entry.duration_secs;
            // Keep the latest date (RFC3339 strings compare chronologically).
            if record.4.as_deref().is_none_or(|cur| session_date > *cur) {
                record.4 = Some(session_date.clone());
            }
            // ... existing score / tempo pushes unchanged ...
        }
    }
```

And in the final `.map(...)`, destructure the extra element and set the field:

```rust
        .map(
            |(item_id, (session_count, total_secs, mut score_history, mut tempo_history, last_practiced_at))| {
                // ... existing sorts / latest_score / latest_tempo ...
                (
                    item_id,
                    ItemPracticeSummary {
                        session_count,
                        total_minutes: (total_secs / 60) as u32,
                        latest_score,
                        score_history,
                        latest_tempo,
                        tempo_history,
                        last_practiced_at,
                    },
                )
            },
        )
```

> `is_none_or` is stable since Rust 1.82 (CI is 1.90). If the linter objects, use `record.4.as_deref().map_or(true, |cur| session_date > *cur)`.

Update any other `ItemPracticeSummary { ... }` literals (search the workspace: `rg "ItemPracticeSummary \{"`) to include `last_practiced_at` — at minimum any test fixtures. Web (`intrada-web`) reads the field via wasm-bindgen and needs no change, but must still compile.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p intrada-core summary_last_practiced_is_max`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/intrada-core/src/model.rs crates/intrada-core/src/app.rs
git commit -m "feat(core): track last_practiced_at on item practice summary"
```

---

## Task 3: Sort comparator driven by `active_sort`

**Files:**
- Modify: `crates/intrada-core/src/model.rs:18-19` (`Model` — add `active_sort`)
- Modify: `crates/intrada-core/src/model.rs:163-191` (`ViewModel` — add `active_sort`)
- Modify: `crates/intrada-core/src/app.rs:379-380` (replace hardcoded sort) and `:492` (ViewModel build)
- Test: `crates/intrada-core/src/app.rs` (existing `mod tests`)

- [ ] **Step 1: Write the failing tests**

Add to `mod tests` in `crates/intrada-core/src/app.rs`. Add a small helper that stamps a last-practiced date onto an item's summary, then four ordering tests:

```rust
fn set_last_practiced(model: &mut Model, item_id: &str, at: chrono::DateTime<chrono::Utc>) {
    model.practice_summaries.insert(
        item_id.to_string(),
        crate::model::ItemPracticeSummary {
            session_count: 1,
            total_minutes: 1,
            latest_score: None,
            score_history: vec![],
            latest_tempo: None,
            tempo_history: vec![],
            last_practiced_at: Some(at.to_rfc3339()),
        },
    );
}

#[test]
fn view_sorts_by_title_ascending() {
    let app = Intrada;
    let mut model = Model::test_default();
    let now = chrono::Utc::now();
    model.items = vec![
        make_item("a", "Sonata", ItemKind::Piece, now),
        make_item("b", "etude", ItemKind::Piece, now), // lowercase: case-insensitive
        make_item("c", "Ballade", ItemKind::Piece, now),
    ];
    model.active_sort = LibrarySort { field: SortField::Title, direction: SortDirection::Ascending };
    let vm = app.view(&model);
    let titles: Vec<_> = vm.items.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(titles, vec!["Ballade", "etude", "Sonata"]);
}

#[test]
fn view_sorts_by_last_practiced_descending_most_recent_first() {
    let app = Intrada;
    let mut model = Model::test_default();
    let now = chrono::Utc::now();
    model.items = vec![
        make_item("a", "Stale", ItemKind::Piece, now),
        make_item("b", "Fresh", ItemKind::Piece, now),
    ];
    set_last_practiced(&mut model, "a", now - chrono::Duration::days(5));
    set_last_practiced(&mut model, "b", now - chrono::Duration::days(1));
    model.active_sort = LibrarySort { field: SortField::LastPracticed, direction: SortDirection::Descending };
    let vm = app.view(&model);
    assert_eq!(vm.items[0].title, "Fresh");
    assert_eq!(vm.items[1].title, "Stale");
}

#[test]
fn view_never_practiced_sorts_as_oldest() {
    let app = Intrada;
    let mut model = Model::test_default();
    let now = chrono::Utc::now();
    model.items = vec![
        make_item("a", "Practiced", ItemKind::Piece, now),
        make_item("b", "NeverPractised", ItemKind::Piece, now),
    ];
    set_last_practiced(&mut model, "a", now - chrono::Duration::days(2));
    // "b" has no practice summary → never practised.

    // Ascending (longest since practised first): never-practised rises to the top.
    model.active_sort = LibrarySort { field: SortField::LastPracticed, direction: SortDirection::Ascending };
    assert_eq!(app.view(&model).items[0].title, "NeverPractised");

    // Descending (most recent first): never-practised sinks to the bottom.
    model.active_sort = LibrarySort { field: SortField::LastPracticed, direction: SortDirection::Descending };
    assert_eq!(app.view(&model).items.last().unwrap().title, "NeverPractised");
}

#[test]
fn view_default_sort_is_date_added_newest_first() {
    // Regression for the existing behaviour, now via active_sort default.
    let app = Intrada;
    let mut model = Model::test_default();
    let t1 = chrono::Utc::now() - chrono::Duration::hours(2);
    let t2 = chrono::Utc::now();
    model.items = vec![
        make_item("a", "Old", ItemKind::Piece, t1),
        make_item("b", "New", ItemKind::Piece, t2),
    ];
    let vm = app.view(&model); // default active_sort
    assert_eq!(vm.items[0].title, "New");
    assert_eq!(vm.items[1].title, "Old");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p intrada-core view_sorts_ view_never_practiced view_default_sort`
Expected: FAIL — `no field active_sort on Model`.

- [ ] **Step 3: Write minimal implementation**

Add to `Model` (`crates/intrada-core/src/model.rs`, near `active_query`):

```rust
    /// Library list sort order. Defaults to Date Added / newest-first.
    pub active_sort: LibrarySort,
```

(`Model` derives `Default` and `LibrarySort: Default`, so no other init site needs touching — `test_default()` and `Model::default()` pick up the default.)

Add to `ViewModel` (`crates/intrada-core/src/model.rs`, near `active_query`):

```rust
    /// Active sort, mirrored so the shell's menu reads one source of truth.
    pub active_sort: LibrarySort,
```

Import the types at the top of `model.rs` (extend the existing `use crate::domain::...`):

```rust
use crate::domain::{LibrarySort, ListQuery, SortDirection, SortField};
```

In `crates/intrada-core/src/app.rs`, replace the hardcoded sort at line ~380:

```rust
        // Sort by the active order (defaults to newest-added first).
        sort_library_items(&mut items, &model.active_sort);
```

Add the comparator as a free function near `apply_query_filter`:

```rust
fn sort_library_items(items: &mut [LibraryItemView], sort: &LibrarySort) {
    items.sort_by(|a, b| {
        let primary = match sort.field {
            SortField::DateAdded => a.created_at.cmp(&b.created_at),
            SortField::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            SortField::LastPracticed => {
                // None = "never practised" = earliest. Option ordering puts
                // None < Some, which is exactly that.
                let la = a.practice.as_ref().and_then(|p| p.last_practiced_at.as_deref());
                let lb = b.practice.as_ref().and_then(|p| p.last_practiced_at.as_deref());
                la.cmp(&lb)
            }
        };
        let directed = match sort.direction {
            SortDirection::Ascending => primary,
            SortDirection::Descending => primary.reverse(),
        };
        // Stable tiebreaker so equal keys don't jitter between renders.
        directed
            .then_with(|| b.created_at.cmp(&a.created_at))
            .then_with(|| a.id.cmp(&b.id))
    });
}
```

Add `LibrarySort, SortDirection, SortField` to the `app.rs` imports (the file already imports `ListQuery` at line 24 — extend that `use`).

In the `view()` ViewModel construction (`app.rs:~492`), add the field alongside `active_query`:

```rust
            active_sort: model.active_sort,
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p intrada-core view_sorts_ view_never_practiced view_default_sort view_items_sorted`
Expected: PASS (including the pre-existing `view_items_sorted_newest_first`).

- [ ] **Step 5: Commit**

```bash
git add crates/intrada-core/src/model.rs crates/intrada-core/src/app.rs
git commit -m "feat(core): sort library by active_sort with stable tiebreaker"
```

---

## Task 4: `SetSort` event + `SaveLibrarySort` persistence effect

**Files:**
- Modify: `crates/intrada-core/src/app.rs:111` (Event enum), `:147-156` (`AppEffect`), `:302-305` (handler)
- Test: `crates/intrada-core/src/app.rs` (existing `mod tests`)

- [ ] **Step 1: Write the failing test**

Add to `mod tests` in `crates/intrada-core/src/app.rs`. Drive the event through `update` and assert both the model mutation and the emitted save effect:

```rust
#[test]
fn set_sort_updates_model_and_emits_save_effect() {
    use crux_core::testing::AppTester;
    let app: AppTester<Intrada> = AppTester::default();
    let mut model = Model::test_default();

    let sort = LibrarySort { field: SortField::Title, direction: SortDirection::Ascending };
    let mut update = app.update(Event::SetSort(sort), &mut model);

    assert_eq!(model.active_sort, sort, "model sort is updated");

    // The save effect carries the chosen sort.
    let saved = update
        .effects_mut()
        .find_map(|eff| match eff {
            Effect::App(req) => match &req.operation {
                AppEffect::SaveLibrarySort(s) => Some(*s),
                _ => None,
            },
            _ => None,
        });
    assert_eq!(saved, Some(sort), "SetSort emits SaveLibrarySort with the chosen order");
}
```

> NOTE: the exact `AppTester` effect-introspection API (`effects_mut`, `req.operation`) varies by crux version. Confirm against an existing effect-asserting test in this file (search `AppTester` / `effects`); if the suite has no precedent, assert just `model.active_sort == sort` here and cover the save-effect wiring in the Swift test (Task 6) instead. The model mutation is the non-negotiable assertion.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p intrada-core set_sort_updates_model`
Expected: FAIL — `no variant SetSort` / `no variant SaveLibrarySort`.

- [ ] **Step 3: Write minimal implementation**

Add the event variant to `Event` (`app.rs`, near `SetQuery` at line 111):

```rust
    /// User chose a library sort order; persist it and re-render.
    SetSort(LibrarySort),
```

Add the effect variant to `AppEffect` (`app.rs:147`):

```rust
    /// Persist the chosen library sort order (small singleton — UserDefaults
    /// on iOS / localStorage on web). Fire-and-forget; output is `()`.
    SaveLibrarySort(LibrarySort),
```

Add the handler in `update` (`app.rs`, beside `SetQuery` at line 302):

```rust
            Event::SetSort(sort) => {
                model.active_sort = sort;
                Command::all([
                    Command::notify_shell(AppEffect::SaveLibrarySort(sort)).into(),
                    crux_core::render::render(),
                ])
            }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p intrada-core set_sort_updates_model`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/intrada-core/src/app.rs
git commit -m "feat(core): SetSort event persists library sort via SaveLibrarySort effect"
```

---

## Task 5: Regenerate Swift bindings

**Files:**
- Modify: `ios/generated/**` (generated — do not hand-edit)

- [ ] **Step 1: Run the core test suite (whole workspace) before regenerating**

Run: `cargo test --workspace`
Expected: PASS. (Catches any shared-core-type breakage in `intrada-web`/`intrada-api` from the new `ItemPracticeSummary` field — see CLAUDE.md "compile the whole workspace for shared core types".)

- [ ] **Step 2: Regenerate bindings**

Run: `just ios-gen`
Expected: bindings regenerate; `git status` shows changes under `ios/generated/SharedTypes/...` adding `SortField`, `SortDirection`, `LibrarySort`, the `Effect.app` `SaveLibrarySort` case, the `Event.setSort` case, `ViewModel.activeSort`, and `ItemPracticeSummary.lastPracticedAt`.

- [ ] **Step 3: Verify the generated types appear**

Run: `grep -n "SortField\|LibrarySort\|saveLibrarySort\|setSort\|lastPracticedAt\|activeSort" ios/generated/SharedTypes/Sources/SharedTypes/SharedTypes.swift`
Expected: matches for each. If any are missing, the Rust type lacks the `facet`/`repr(C)` derives — fix the core type (Task 1) and re-run, never edit the generated file.

- [ ] **Step 4: Commit**

```bash
git add ios/generated
git commit -m "chore(ios): regenerate bindings for library sort types"
```

---

## Task 6: Swift persistence — write on save, restore at launch

**Files:**
- Modify: `ios/Intrada/Core/Store.swift:16-28` (inject `UserDefaults`), `:41-44` (`.app` handler), add `restorePersistedSort()`
- Test: `ios/IntradaTests/StoreEffectLoopTests.swift`

- [ ] **Step 1: Write the failing tests**

Add to `StoreEffectLoopTests.swift`. Use an isolated `UserDefaults` suite so tests don't touch the real domain:

```swift
func testSaveLibrarySortEffectWritesToDefaults() throws {
  let defaults = UserDefaults(suiteName: "sort-test-\(UUID().uuidString)")!
  let sort = LibrarySort(field: .title, direction: .ascending)
  let bridge = FakeBridge()
  bridge.updateHandler = { _ in [Request(id: 5, effect: .app(.saveLibrarySort(sort)))] }
  let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)

  store.send(.setQuery(nil)) // any event to drive the scripted effect

  let data = try XCTUnwrap(defaults.data(forKey: Store.sortDefaultsKey))
  let restored = try LibrarySort.bincodeDeserialize(input: [UInt8](data))
  XCTAssertEqual(restored, sort, "save effect persists the chosen sort")
  XCTAssertEqual(bridge.emptyResolved, [5], "save effect still acks via resolveEmpty")
}

func testRestorePersistedSortReplaysSetSort() throws {
  let defaults = UserDefaults(suiteName: "sort-test-\(UUID().uuidString)")!
  let sort = LibrarySort(field: .lastPracticed, direction: .ascending)
  defaults.set(Data(try sort.bincodeSerialize()), forKey: Store.sortDefaultsKey)

  let bridge = FakeBridge()
  var sentEvents: [Event] = []
  bridge.updateHandler = { event in sentEvents.append(event); return [] }
  let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)

  store.restorePersistedSort()

  XCTAssertEqual(sentEvents, [.setSort(sort)], "restore re-dispatches SetSort with the stored order")
}

func testRestorePersistedSortNoopWhenAbsent() {
  let defaults = UserDefaults(suiteName: "sort-test-\(UUID().uuidString)")!
  let bridge = FakeBridge()
  var sentEvents: [Event] = []
  bridge.updateHandler = { event in sentEvents.append(event); return [] }
  let store = Store(bridge: bridge, session: mockSession(), sortDefaults: defaults)

  store.restorePersistedSort()

  XCTAssertTrue(sentEvents.isEmpty, "no stored sort → no event")
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run (Xcode or CLI): the iOS test target. Expected: FAIL to compile — `Store` has no `sortDefaults:` parameter / no `sortDefaultsKey` / no `restorePersistedSort`.

Use the project's test runner (see `just` recipes / CI). If running from CLI:
`xcodebuild test -scheme Intrada -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:IntradaTests/StoreEffectLoopTests`
Expected: build failure referencing the missing API.

- [ ] **Step 3: Write minimal implementation**

In `ios/Intrada/Core/Store.swift`, add the key and inject `UserDefaults`:

```swift
  static let sortDefaultsKey = "intrada.library-sort"

  private let bridge: CoreBridge
  private let session: URLSession
  private let store: (any ItemStore)?
  private let sortDefaults: UserDefaults

  init(
    bridge: CoreBridge = LiveBridge(), session: URLSession = .shared,
    store: (any ItemStore)? = nil, sortDefaults: UserDefaults = .standard
  ) {
    self.bridge = bridge
    self.session = session
    self.store = store ?? (try? LibraryStore.inMemory())
    self.sortDefaults = sortDefaults
    self.viewModel = guarded { try bridge.view() }
  }
```

Replace the `.app` case in `process(_:)`:

```swift
      case .app(let appEffect):
        handleAppEffect(appEffect)
        process(guarded { try bridge.resolveEmpty(request.id) } ?? [])
```

Add the handler + restore method:

```swift
  private func handleAppEffect(_ effect: AppEffect) {
    switch effect {
    case .saveLibrarySort(let sort):
      if let bytes = guarded({ try sort.bincodeSerialize() }) {
        sortDefaults.set(Data(bytes), forKey: Self.sortDefaultsKey)
      }
    case .saveSessionInProgress, .clearSessionInProgress:
      break  // localStorage crash-recovery: no-op on native for now
    }
  }

  /// Re-apply the persisted library sort at launch by replaying `SetSort`.
  func restorePersistedSort() {
    guard let data = sortDefaults.data(forKey: Self.sortDefaultsKey),
      let sort = guarded({ try LibrarySort.bincodeDeserialize(input: [UInt8](data)) })
    else { return }
    send(.setSort(sort))
  }
```

- [ ] **Step 4: Run tests to verify they pass**

Run the `StoreEffectLoopTests` target again. Expected: the three new tests PASS and existing `testAppEffectResolvesEmpty` / `testBatchProcessesEveryRequest` still PASS (the `.app` case still acks).

- [ ] **Step 5: Commit**

```bash
git add ios/Intrada/Core/Store.swift ios/IntradaTests/StoreEffectLoopTests.swift
git commit -m "feat(ios): persist + restore library sort via UserDefaults"
```

---

## Task 7: `LibrarySortMenu` component

**Files:**
- Create: `ios/Intrada/Views/Components/LibrarySortMenu.swift`

- [ ] **Step 1: Write the component**

This is presentational SwiftUI (verified on-device/snapshot, not unit-tested). Create `ios/Intrada/Views/Components/LibrarySortMenu.swift`:

```swift
import SharedTypes
import SwiftUI

/// Display wrapper over the core `SortField` — owns the menu labels and the
/// natural default direction when switching *to* a field (shell concern, like
/// `LibraryFilter` wraps `ItemKind`).
enum LibrarySortField: CaseIterable, Identifiable {
  case dateAdded, lastPracticed, title

  var id: Self { self }

  init(_ core: SortField) {
    switch core {
    case .dateAdded: self = .dateAdded
    case .lastPracticed: self = .lastPracticed
    case .title: self = .title
    }
  }

  var core: SortField {
    switch self {
    case .dateAdded: .dateAdded
    case .lastPracticed: .lastPracticed
    case .title: .title
    }
  }

  var label: String {
    switch self {
    case .dateAdded: "Date Added"
    case .lastPracticed: "Last Practiced"
    case .title: "Title"
    }
  }

  /// Direction applied when the user switches to this field.
  var naturalDefault: SortDirection {
    switch self {
    case .dateAdded, .lastPracticed: .descending
    case .title: .ascending
    }
  }
}

/// Native pull-down sort control (Files/Mail idiom). Tapping the active field
/// flips direction; tapping another switches at its natural default.
struct LibrarySortMenu: View {
  let current: LibrarySort
  let onChange: (LibrarySort) -> Void

  var body: some View {
    Menu {
      ForEach(LibrarySortField.allCases) { field in
        Button {
          onChange(next(for: field))
        } label: {
          if field.core == current.field {
            Label(field.label, systemImage: directionSymbol)
          } else {
            Text(field.label)
          }
        }
      }
    } label: {
      Image(systemName: "arrow.up.arrow.down")
        .font(IntradaFont.tab)
        .foregroundStyle(IntradaColor.inkFaint)
        .padding(8)
    }
    .accessibilityLabel("Sort")
    .accessibilityValue("\(LibrarySortField(current.field).label), \(directionAccessibility)")
  }

  private var directionSymbol: String {
    current.direction == .ascending ? "chevron.up" : "chevron.down"
  }

  private var directionAccessibility: String {
    current.direction == .ascending ? "ascending" : "descending"
  }

  private func next(for field: LibrarySortField) -> LibrarySort {
    if field.core == current.field {
      let flipped: SortDirection = current.direction == .ascending ? .descending : .ascending
      return LibrarySort(field: current.field, direction: flipped)
    }
    return LibrarySort(field: field.core, direction: field.naturalDefault)
  }
}

#if DEBUG
  #Preview {
    ZStack {
      PaperBackground()
      LibrarySortMenu(
        current: LibrarySort(field: .dateAdded, direction: .descending),
        onChange: { _ in })
    }
  }
#endif
```

> Confirm the generated Swift spelling of the enum cases (`.dateAdded` vs `.DateAdded`) against `SharedTypes.swift` after Task 5 — UniFFI/facet de-capitalizes, but verify. Adjust if needed.

- [ ] **Step 2: Build to verify it compiles**

Build the Intrada scheme (Xcode build or `mcp__xcode__BuildProject`). Expected: compiles; the `#Preview` renders the icon.

- [ ] **Step 3: Commit**

```bash
git add ios/Intrada/Views/Components/LibrarySortMenu.swift
git commit -m "feat(ios): LibrarySortMenu native pull-down sort control"
```

---

## Task 8: Wire the menu into `LibraryScreen` + restore at launch

**Files:**
- Modify: `ios/Intrada/Views/Screens/LibraryScreen.swift:8-30` (binding + filter row layout)
- Modify: `ios/Intrada/Views/RootView.swift:32-42` (call restore)

- [ ] **Step 1: Add the sort binding + place the menu**

In `LibraryScreen.swift`, add a binding that reads `viewModel.activeSort` and writes via `.setSort` (mirrors the existing `filterBinding`):

```swift
  private var sortBinding: Binding<LibrarySort> {
    Binding(
      get: { store.viewModel?.activeSort ?? LibrarySort(field: .dateAdded, direction: .descending) },
      set: { store.send(.setSort($0)) })
  }
```

Change the filter row to an `HStack` with the pills left (scrollable) and the menu pinned right:

```swift
      VStack(spacing: 0) {
        HStack(spacing: 8) {
          LibraryFilterTabs(selection: filterBinding)
            .frame(maxWidth: .infinity, alignment: .leading)
          LibrarySortMenu(current: sortBinding.wrappedValue, onChange: { sortBinding.wrappedValue = $0 })
        }
        .padding(.horizontal, 16)
        .padding(.top, 12)
        .padding(.bottom, 14)
        content
      }
```

- [ ] **Step 2: Call restore at launch**

In `RootView.swift`, in the `.task` non-seed branch (after `startApp`):

```swift
      if seedSampleData {
        store.send(.loadSampleData)
      } else {
        store.send(.startApp(apiBaseUrl: apiBaseURL, localFirst: true))
        store.restorePersistedSort()
      }
```

- [ ] **Step 3: Build + drive the preview to verify**

Build and launch on the simulator (`just ios-run` with `SEED=1` for populated data, or run the Populated preview). Verify:
- The `arrow.up.arrow.down` icon sits at the right of the filter row.
- Tapping it opens a menu with Date Added / Last Practiced / Title; the active field shows a chevron.
- Picking a field reorders the list; re-tapping the active field flips direction.
- Force-quit and relaunch (non-seed): the list reopens in the last-chosen order.

Expected: all behaviours confirmed. (Per CLAUDE.md UI-verification: drive the preview; if the running app is unreachable, hand off these exact steps + expected results to the user.)

- [ ] **Step 4: Commit**

```bash
git add ios/Intrada/Views/Screens/LibraryScreen.swift ios/Intrada/Views/RootView.swift
git commit -m "feat(ios): add sort menu to library toolbar + restore at launch"
```

---

## Task 9: Snapshot test + accessibility

**Files:**
- Modify: `ios/IntradaTests/ScreenSnapshotTests.swift`

- [ ] **Step 1: Add a snapshot of the populated library (now with the sort control)**

The existing `testLibraryScreenPopulated` already snapshots `LibraryScreen` with `.previewLibrary`; the sort icon now appears in it. Re-record that reference, and verify VoiceOver labels manually (the menu button announces "Sort" + current value).

Run the snapshot suite in **record** mode once to capture the new control, then in assert mode:
`xcodebuild test -scheme Intrada -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:IntradaTests/ScreenSnapshotTests`
Expected: with references re-recorded, PASS.

> Snapshot determinism: record on the pinned simulator/runtime the CI uses (macos-26 / Xcode 26.5 per project CI notes) or the reference won't match in CI. Keep the new/updated PNG lean (see snapshot hygiene memory).

- [ ] **Step 2: Commit**

```bash
git add ios/IntradaTests/ScreenSnapshotTests.swift ios/IntradaTests/__Snapshots__
git commit -m "test(ios): re-record library snapshot with sort control"
```

---

## Task 10: Full verification + docs

**Files:**
- Modify: `docs/roadmap.md` (close/advance the item), `specs/library-sort.md` (only if anything diverged)

- [ ] **Step 1: Rust gates (run locally before pushing — CLAUDE.md)**

Run:
```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
Expected: all PASS. (Whole workspace so `intrada-web`/`intrada-api` are confirmed to still compile against the new `ItemPracticeSummary` field.)

- [ ] **Step 2: iOS gates**

Run the full `IntradaTests` suite (unit + snapshot) on the pinned simulator. Expected: PASS.

- [ ] **Step 3: Update the roadmap**

Mark the library-sort roadmap item done in `docs/roadmap.md`. If the UI diverged from the spec's open questions (chevron treatment, locale-aware title), note the resolution in `specs/library-sort.md`.

- [ ] **Step 4: Commit**

```bash
git add docs/roadmap.md specs/library-sort.md
git commit -m "docs: mark library sort done; record UI decisions"
```

- [ ] **Step 5: Self-review + PR**

Use `superpowers:requesting-code-review` (Tier 3) with "comment-policy violations are Blockers, not Nits". Open tracked GitHub issues for any deferred items *before* posting the self-review comment; end the comment with `Deferred items tracked: #N` (or `none`). Open the PR (never push to main); verify CI + Codecov patch-coverage against the spec's Testing section.

---

## Self-Review (plan vs spec)

- **Spec coverage:** §1 UI → Tasks 7–8; §2 core sort → Tasks 1, 3; §3 last-practiced → Task 2; §4 persistence → Tasks 4, 6; §5 key decisions → all; §6 deliberately-not-doing → respected (no Priority/Key sort, one global sort, no crux_kv); testing → Tasks 1–4, 6, 9, 10. No gaps.
- **Type consistency:** `LibrarySort { field, direction }`, `SortField { DateAdded, LastPracticed, Title }`, `SortDirection { Descending, Ascending }`, `ItemPracticeSummary.last_practiced_at: Option<String>`, `Event::SetSort`, `AppEffect::SaveLibrarySort`, `Store.sortDefaultsKey`, `Store.restorePersistedSort()`, `LibrarySortMenu`/`LibrarySortField` — names used identically across tasks.
- **Placeholders:** none — every code step shows the code. Two NOTEs flag version-specific API surfaces (crux `AppTester` introspection, generated case spelling) to confirm against the codebase, with a stated fallback; these are verification cues, not unfilled blanks.
- **Offline-first invariants:** no network on the sort path (1), no GRDB schema/migration (summary is a derived view type), small singleton via UserDefaults (8), reconciliation N/A. Core tested; both `local_first` and online modes exercised via the mode-agnostic `view()` path.
