# Priority Items — Phase A (core + API + DB foundation) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `priority` flag to library items end-to-end (Crux core + Axum API + SQLite), reusing the existing item update pipeline — without touching the web UI and without removing Goals yet.

**Architecture:** `priority: bool` becomes a field on the `Item` domain type and an optional field on `UpdateItem`. Toggling reuses the existing `ItemEvent::Update → ItemUpdated` mutate-response path (no new event, endpoint, or HTTP function). The DB gets one appended column with a `DEFAULT 0`, read via positional indexing appended at the end of `SELECT_COLUMNS` so existing column indices are untouched.

**Tech Stack:** Rust (crux_core 0.18, crux_http), serde, axum 0.8, libsql (Turso/SQLite), tokio.

**Spec:** [`specs/priority-items.md`](../../../specs/priority-items.md). This plan implements the "Added" data-model section only; deletion of Goals and all UI are separate follow-up plans.

**Reuse decision (why no new event/endpoint):** A one-tap star toggle goes through `ItemEvent::Update { id, input: UpdateItem { priority: Some(b), ..Default::default() } }`. This reuses the validated optimistic-update path, the `PUT /api/items/{id}` route, and `update_item` DB function. Per CLAUDE.md "Reuse before creating." `CreateItem` is intentionally NOT given a `priority` field — new items default to `false`; you flag existing ones.

---

## File Structure

| File | Change | Responsibility |
|------|--------|----------------|
| `crates/intrada-core/src/domain/item.rs` | Modify | Add `priority: bool` to `Item`; apply it in the `Update` handler arm |
| `crates/intrada-core/src/domain/types.rs` | Modify | Add `priority: Option<bool>` to `UpdateItem` |
| `crates/intrada-core/src/http.rs` | Modify | Carry `priority` in the `UpdateItem` that `update_item` builds |
| `crates/intrada-core/src/model.rs` | Modify | Add `priority: bool` to `LibraryItemView` and map it |
| `crates/intrada-core/src/app.rs` | Modify | Tests; fix `Item { … }` literals broken by the new field |
| `crates/intrada-api/src/migrations.rs` | Modify | Append `add_priority_to_items` migration |
| `crates/intrada-api/src/db/items.rs` | Modify | `SELECT_COLUMNS`, `row_to_item`, `insert_item`, `update_item` |
| `crates/intrada-api/tests/items_test.rs` | Modify | Endpoint tests: default false, toggle true, cross-user isolation |

---

## Task 1: Core — add `priority` field to `Item` and expose it on the view model

**Files:**
- Modify: `crates/intrada-core/src/domain/item.rs` (struct around lines 29–41)
- Modify: `crates/intrada-core/src/model.rs` (`LibraryItemView` around lines 323–338, and its builder)
- Test: `crates/intrada-core/src/app.rs`

- [ ] **Step 1: Write the failing test**

Add to the tests module in `crates/intrada-core/src/app.rs` (alongside the existing `test_item_*` tests):

```rust
#[test]
fn test_new_item_defaults_to_not_priority() {
    let app = Intrada;
    let mut model = Model::test_default();

    let _cmd = app.update(
        Event::Item(ItemEvent::Add(crate::domain::types::CreateItem {
            title: "Prelude".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Bach".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        })),
        &mut model,
    );

    assert_eq!(model.items.len(), 1);
    assert!(!model.items[0].priority);

    let vm = app.view(&model);
    assert!(!vm.items[0].priority);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p intrada-core test_new_item_defaults_to_not_priority`
Expected: FAIL — compile error, `Item` has no field `priority` / `LibraryItemView` has no field `priority`.

- [ ] **Step 3: Add the field to `Item`**

In `crates/intrada-core/src/domain/item.rs`, add `priority` as the last field of `Item`:

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub kind: ItemKind,
    pub composer: Option<String>,
    pub key: Option<String>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub priority: bool,
}
```

The `#[serde(default)]` lets older JSON (without the field) still deserialize as `false`.

- [ ] **Step 4: Set `priority: false` where `Item` is built on Add**

Find where `ItemEvent::Add` constructs the optimistic `Item` in `crates/intrada-core/src/domain/item.rs` (the `Add(input)` handler arm). Add `priority: false,` to that `Item { … }` literal.

- [ ] **Step 5: Add the field to `LibraryItemView` and map it**

In `crates/intrada-core/src/model.rs`, add to `LibraryItemView` (after `latest_achieved_tempo`):

```rust
    pub priority: bool,
```

Then locate where `LibraryItemView { … }` is constructed (grep `LibraryItemView {` across `crates/intrada-core/src/`). Add to that literal:

```rust
        priority: item.priority,
```

(`item` is the `&Item` being mapped; adjust the binding name to match the surrounding code.)

- [ ] **Step 6: Fix the broken `Item { … }` literals**

Adding a field breaks every `Item { … }` literal in core tests. Build and fix each one the compiler reports:

Run: `cargo build -p intrada-core --tests`
For each `missing field priority` error, add `priority: false,` to that literal. These are mechanical and identical.

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test -p intrada-core test_new_item_defaults_to_not_priority`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add crates/intrada-core/src/domain/item.rs crates/intrada-core/src/model.rs crates/intrada-core/src/app.rs
git commit -m "feat(core): add priority field to Item and LibraryItemView"
```

---

## Task 2: Core — toggle priority through the existing `Update` event

**Files:**
- Modify: `crates/intrada-core/src/domain/types.rs` (`UpdateItem` around lines 69–96)
- Modify: `crates/intrada-core/src/domain/item.rs` (`Update` handler arm around lines 83–120)
- Test: `crates/intrada-core/src/app.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/intrada-core/src/app.rs` tests:

```rust
#[test]
fn test_update_sets_item_priority() {
    let app = Intrada;
    let now = chrono::Utc::now();
    let mut model = Model {
        items: vec![Item {
            id: "p1".to_string(),
            title: "Etude".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            priority: false,
        }],
        ..Model::test_default()
    };

    let _cmd = app.update(
        Event::Item(ItemEvent::Update {
            id: "p1".to_string(),
            input: crate::domain::types::UpdateItem {
                priority: Some(true),
                ..Default::default()
            },
        }),
        &mut model,
    );

    assert!(model.last_error.is_none());
    assert!(model.items[0].priority);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p intrada-core test_update_sets_item_priority`
Expected: FAIL — `UpdateItem` has no field `priority`.

- [ ] **Step 3: Add `priority` to `UpdateItem`**

In `crates/intrada-core/src/domain/types.rs`, add to `UpdateItem` (a plain `Option<bool>` — `priority` is not clearable, so no `double_option`):

```rust
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<bool>,
```

- [ ] **Step 4: Apply it in the `Update` handler arm**

In `crates/intrada-core/src/domain/item.rs`, in the `ItemEvent::Update` arm, after the existing `if let Some(tags) = input.tags { … }` block and before `item.updated_at = …`, add:

```rust
    if let Some(priority) = input.priority {
        item.priority = priority;
    }
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p intrada-core test_update_sets_item_priority`
Expected: PASS

- [ ] **Step 6: Run the full core suite**

Run: `cargo test -p intrada-core`
Expected: PASS (all existing tests still green).

- [ ] **Step 7: Commit**

```bash
git add crates/intrada-core/src/domain/types.rs crates/intrada-core/src/domain/item.rs crates/intrada-core/src/app.rs
git commit -m "feat(core): toggle item priority via UpdateItem"
```

---

## Task 3: Core — carry `priority` in the outbound HTTP update

**Files:**
- Modify: `crates/intrada-core/src/http.rs` (`update_item` around lines 98–119)

No new test — this is covered end-to-end by the API tests in Tasks 5–7. This step keeps the optimistic update and the server write consistent.

- [ ] **Step 1: Add `priority` to the `UpdateItem` built by `update_item`**

In `crates/intrada-core/src/http.rs`, in `update_item`, add `priority: Some(item.priority),` to the `UpdateItem { … }` literal:

```rust
    let update = UpdateItem {
        title: Some(item.title.clone()),
        composer: Some(item.composer.clone()),
        key: Some(item.key.clone()),
        tempo: Some(item.tempo.clone()),
        notes: Some(item.notes.clone()),
        tags: Some(item.tags.clone()),
        priority: Some(item.priority),
    };
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p intrada-core`
Expected: success.

- [ ] **Step 3: Commit**

```bash
git add crates/intrada-core/src/http.rs
git commit -m "feat(core): send priority in item update request"
```

---

## Task 4: API — migration to add the `priority` column

**Files:**
- Modify: `crates/intrada-api/src/migrations.rs` (`MIGRATIONS` array)

- [ ] **Step 1: Confirm the next migration number**

Read the end of the `MIGRATIONS` array in `crates/intrada-api/src/migrations.rs`. Find the highest existing `NNNN_` prefix and use `highest + 1` (at time of writing the highest is `0079`, so the new one is `0080`). Use the actual next number you find.

- [ ] **Step 2: Append the migration**

Add as the LAST entry of the `MIGRATIONS` array (use the number confirmed above):

```rust
    (
        "0080_add_priority_to_items",
        "ALTER TABLE items ADD COLUMN priority INTEGER NOT NULL DEFAULT 0;",
    ),
```

- [ ] **Step 3: Verify migrations run**

Run: `cargo test -p intrada-api --test items_test`
Expected: PASS — the existing item tests run migrations on a fresh SQLite db; this confirms the new statement is valid SQL and applies cleanly.

- [ ] **Step 4: Commit**

```bash
git add crates/intrada-api/src/migrations.rs
git commit -m "feat(api): add priority column to items table"
```

---

## Task 5: API — read `priority` from the row (default-false endpoint test)

**Files:**
- Modify: `crates/intrada-api/src/db/items.rs` (`SELECT_COLUMNS` line ~69, `row_to_item` lines ~35–67, `insert_item` return ~150)
- Test: `crates/intrada-api/tests/items_test.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/intrada-api/tests/items_test.rs`:

```rust
#[tokio::test]
async fn created_item_defaults_to_not_priority() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/items",
        json!({ "title": "Nocturne", "kind": "piece", "composer": "Chopin", "tags": [] }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let item: Item = common::json(&body);
    assert!(!item.priority);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p intrada-api --test items_test created_item_defaults_to_not_priority`
Expected: FAIL — `Item` has no field `priority` in scope (compile error from `intrada_core::Item`), OR the field is unread. (Core already has the field from Task 1, so the failure is in `db/items.rs` not populating it.)

- [ ] **Step 3: Append `priority` to `SELECT_COLUMNS`**

In `crates/intrada-api/src/db/items.rs`, append `priority` LAST so existing indices stay stable:

```rust
const SELECT_COLUMNS: &str =
    "id, kind, title, composer, key_signature, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at, priority";
```

- [ ] **Step 4: Read the new column in `row_to_item`**

In `row_to_item`, after reading `updated_at_str` (index 10), add index 11:

```rust
    let priority_int: i64 = col!(row, 11)?;
```

and add to the returned `Item { … }`:

```rust
        priority: priority_int != 0,
```

- [ ] **Step 5: Set `priority: false` in the `insert_item` return**

`insert_item` does not write the `priority` column (it relies on `DEFAULT 0`), so its returned `Item` must reflect that. Add `priority: false,` to the `Ok(Item { … })` literal at the end of `insert_item`.

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test -p intrada-api --test items_test created_item_defaults_to_not_priority`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/intrada-api/src/db/items.rs crates/intrada-api/tests/items_test.rs
git commit -m "feat(api): read item priority from items table"
```

---

## Task 6: API — persist `priority` on update (toggle endpoint test)

**Files:**
- Modify: `crates/intrada-api/src/db/items.rs` (`update_item` lines ~166–239)
- Test: `crates/intrada-api/tests/items_test.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/intrada-api/tests/items_test.rs`:

```rust
#[tokio::test]
async fn update_toggles_item_priority() {
    let (app, conn) = common::setup_test_app_with_conn(None, "http://localhost:3000").await;

    let (status, body) = common::post_json(
        app.clone(),
        "/api/items",
        json!({ "title": "Gymnopedie", "kind": "piece", "composer": "Satie", "tags": [] }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let created: Item = common::json(&body);

    let (status, body) = common::put_json(
        app,
        &format!("/api/items/{}", created.id),
        json!({ "priority": true }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Item = common::json(&body);
    assert!(updated.priority);

    // The other fields must survive the partial update.
    assert_eq!(updated.title, "Gymnopedie");
    let _ = conn;
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p intrada-api --test items_test update_toggles_item_priority`
Expected: FAIL — the response `priority` is `false` (update path ignores the field).

- [ ] **Step 3: Compute `priority` in `update_item`**

In `crates/intrada-api/src/db/items.rs` `update_item`, after the `let tags = input.tags.as_ref().unwrap_or(&current.tags);` line, add:

```rust
    let priority = input.priority.unwrap_or(current.priority);
```

- [ ] **Step 4: Write `priority` in the UPDATE statement**

Change the `UPDATE items SET …` statement to include `priority`, shifting the WHERE bindings. Replace the existing `conn.execute(...)` SQL + params with:

```rust
    conn.execute(
        "UPDATE items SET title = ?1, composer = ?2, key_signature = ?3, tempo_marking = ?4, tempo_bpm = ?5, notes = ?6, tags = ?7, updated_at = ?8, priority = ?9 WHERE id = ?10 AND user_id = ?11",
        libsql::params![
            title.as_str(),
            composer,
            key,
            tempo_marking.as_deref(),
            tempo_bpm,
            notes,
            tags_json.as_str(),
            now_str.as_str(),
            priority as i64,
            id,
            user_id
        ],
    )
    .await?;
```

- [ ] **Step 5: Return `priority` in the updated `Item`**

Add `priority,` to the `Ok(Some(Item { … }))` literal at the end of `update_item`.

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test -p intrada-api --test items_test update_toggles_item_priority`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add crates/intrada-api/src/db/items.rs crates/intrada-api/tests/items_test.rs
git commit -m "feat(api): persist item priority on update"
```

---

## Task 7: API — cross-user isolation test for the priority toggle

**Files:**
- Test: `crates/intrada-api/tests/items_test.rs`

This guards the CLAUDE.md requirement that DB writes test cross-user isolation. With auth disabled the harness uses a single fake user, so isolation is exercised at the DB-function level: a toggle scoped to a non-owning user must not change the row.

- [ ] **Step 1: Write the failing-then-passing test**

Add to `crates/intrada-api/tests/items_test.rs`. This calls the DB layer directly to assert the `user_id` scope on the UPDATE:

```rust
#[tokio::test]
async fn update_priority_is_scoped_to_owner() {
    use intrada_api::db::items;

    let (_app, conn) = common::setup_test_app_with_conn(None, "http://localhost:3000").await;

    let created = items::insert_item(
        &conn,
        "owner",
        &intrada_core::domain::types::CreateItem {
            title: "Arabesque".to_string(),
            kind: intrada_core::domain::item::ItemKind::Piece,
            composer: Some("Debussy".to_string()),
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
        },
    )
    .await
    .unwrap();

    let result = items::update_item(
        &conn,
        &created.id,
        "intruder",
        &intrada_core::domain::types::UpdateItem {
            priority: Some(true),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert!(result.is_none(), "non-owner update must not match the row");

    let still = items::get_item(&conn, &created.id, "owner").await.unwrap().unwrap();
    assert!(!still.priority, "owner's row must be unchanged");
}
```

> Confirm the exact public paths (`intrada_api::db::items::{insert_item, update_item, get_item}`) compile; if `db` is not re-exported from the crate root, adjust the `use` to the actual module path. `update_item` returns `Option<Item>` (`None` when no row matches the id+user_id) per `db/items.rs`.

- [ ] **Step 2: Run test**

Run: `cargo test -p intrada-api --test items_test update_priority_is_scoped_to_owner`
Expected: PASS (the `WHERE … AND user_id = ?` scope already enforces this; this test locks it in).

- [ ] **Step 3: Run the full API suite + clippy + fmt**

Run:
```bash
cargo test -p intrada-api
cargo clippy -p intrada-api -p intrada-core -- -D warnings
cargo fmt --check
```
Expected: all PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/intrada-api/tests/items_test.rs
git commit -m "test(api): cross-user isolation for item priority toggle"
```

---

## Final verification (before opening the PR)

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes for the touched crates
- [ ] `cargo test -p intrada-core` passes
- [ ] `cargo test -p intrada-api` passes
- [ ] Goals still build and work (this plan does not remove them)
- [ ] The `specs/priority-items.md` spec is included as the first commit on this branch (Tier-3 spec rides with Phase A per CLAUDE.md)

## Coverage (for the PR description)

Coverage: full — new core behaviour has unit tests (default-false, toggle-via-update); new API behaviour has endpoint tests (default-false, toggle, cross-user isolation). `migrations.rs` is in the Codecov ignore list (SQL strings).

## Out of scope (tracked for follow-up plans)

- **Plan 2** — atomic Goals rip-out across core + API + web; remove the `goals` feature flag.
- **Plan 3** — priority UI: star toggle, Library priority section, "Practise your priorities" (re-points the salvaged `order_goal_items_least_ready_first` ordering), Library-as-landing.
- **Plan 4** — Track "neglected priority" signal.
