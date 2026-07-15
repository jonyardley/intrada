# Scoring 0–10 + overall session score — Implementation Plan (Phase 1 / #1008)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move practice scoring from a 1–5 scale to 1–10 (with `None` = unrated), rescale existing recorded scores ×2, and add an optional overall session score — all in `intrada-core` + the native iOS app.

**Architecture:** The score scale is governed by two core constants (`MIN_SCORE`/`MAX_SCORE`); the per-entry handler and validation already read them, so the data-layer change is small. The overall session score is a new `Option<u8>` carried through `SummarySession` → `PracticeSession` → `PracticeSessionView`/`SummaryView`, set by a new `UpdateSessionScore` event. On device, GRDB gains a `session_score` column and a one-time migration that doubles each stored entry score. The Swift bindings are regenerated, and the summary UI gains a 1–10 entry scorer plus an overall scorer.

**Tech Stack:** Rust (crux_core, serde, bincode FFI), SwiftUI, GRDB (SQLite), swift-snapshot-testing.

## Global Constraints

- **Scope: `intrada-core` + native iOS only.** Do **not** touch `intrada-api` (Turso) or the Leptos web shell — web/API parity is deferred (tracked on #1008).
- **A rated score is `1..=10`; unrated is `None`, never `0`.** `MIN_SCORE = 1`, `MAX_SCORE = 10`. The ring (#1009) renders `None`/unrated as an en-dash; `0` is never a settable score.
- **Historical scores rescale ×2** (a stored `3` → `6`), clamped to `10`. One-time, forward-only GRDB migration. **Never edit a shipped migration** — append new ones.
- **Bincode FFI bridge:** new fields use `#[serde(default)]`; append the new `SessionEvent` variant (don't reorder). Bridge-crossing types get an `assert_round_trips` test (#846).
- **Generated Swift bindings are a build artifact** — regenerate via `just ios-gen`; never hand-edit `ios/generated/`.
- **TDD:** write the failing test, watch it fail, implement minimally, watch it pass, commit. `cargo fmt --check` + `cargo clippy -- -D warnings` must pass before any push.
- Commit messages end with: `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.

## File Structure

**Core (`crates/intrada-core/src/`):**
- `validation.rs` — `MIN_SCORE`/`MAX_SCORE` constants + `validate_score` (range only).
- `domain/session.rs` — `SetlistEntry`/`SummarySession`/`PracticeSession`, `SessionEvent` (+ new `UpdateSessionScore`), `transition_to_summary`, `SaveSession` handler, tests.
- `model.rs` — `SummaryView`, `PracticeSessionView` (+ `session_score`), and the view builder.
- `domain/types.rs` — `assert_round_trips` session test.

**iOS (`ios/`):**
- `Intrada/Core/LibraryStore.swift` — `session` schema, migrations (v4 add column, v5 rescale), `saveSession`, session load mapping.
- `Intrada/Views/Screens/SessionSummaryScreen.swift` — per-entry scorer (1→10) + new overall scorer.
- `Intrada/Views/Components/MasteryMeter.swift` — interim /10 correctness (Phase 2 replaces it).
- `IntradaTests/ScreenSnapshotTests.swift` + `__Snapshots__/` — re-record summary snapshots.
- `IntradaTests/` — new GRDB migration upgrade-path test.

---

### Task 1: Score scale → 1–10 (core constants + validation)

**Files:**
- Modify: `crates/intrada-core/src/validation.rs:14-15` (constants) and the `validate_score` tests
- Modify (tests): `crates/intrada-core/src/domain/session.rs:2631` (`test_update_entry_score_boundary_values`)

**Interfaces:**
- Produces: `validation::MIN_SCORE = 1`, `validation::MAX_SCORE = 10` (consumed by the existing `UpdateEntryScore` handler at `session.rs:974` and `validate_score`).

- [ ] **Step 1: Write the failing test** — add to `validation.rs` tests module:

```rust
#[test]
fn validate_score_accepts_full_0_to_10_band() {
    assert!(validate_score(&Some(1)).is_ok());
    assert!(validate_score(&Some(10)).is_ok());
    assert!(validate_score(&None).is_ok());
    // 0 is "unrated" (None), never a settable score; 11 is out of range.
    assert!(validate_score(&Some(0)).is_err());
    assert!(validate_score(&Some(11)).is_err());
}
```

- [ ] **Step 2: Run it, verify it fails**

Run: `cargo test -p intrada-core validate_score_accepts_full_0_to_10_band`
Expected: FAIL — `Some(10)` currently rejected (MAX_SCORE is 5).

- [ ] **Step 3: Update the constants** at `validation.rs:14-15`:

```rust
pub const MIN_SCORE: u8 = 1;
pub const MAX_SCORE: u8 = 10;
```

- [ ] **Step 4: Fix the existing boundary test** at `session.rs:2631` — change the "maximum valid" arm from `Some(5)` to `Some(10)`:

```rust
    // Score 10 — maximum valid
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateEntryScore {
            entry_id: entry_id.clone(),
            score: Some(10),
        }),
    );

    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.entries[0].score, Some(10));
    }
```

- [ ] **Step 5: Run the suite, verify green**

Run: `cargo test -p intrada-core score`
Expected: PASS (new band test + boundary test).

- [ ] **Step 6: Commit**

```bash
git add crates/intrada-core/src/validation.rs crates/intrada-core/src/domain/session.rs
git commit -m "feat(core): widen practice score scale to 1–10"
```

---

### Task 2: Overall session score — model field + event (core)

**Files:**
- Modify: `crates/intrada-core/src/domain/session.rs` — `SummarySession` struct, `PracticeSession` struct (`:86-96`), `transition_to_summary` (`:376-404`), `SessionEvent` enum (`:152-269`), the event handler block, `SaveSession` handler (`:1044-1084`)
- Test: same file's `#[cfg(test)] mod tests`

**Interfaces:**
- Produces: `SessionEvent::UpdateSessionScore { score: Option<u8> }`; `PracticeSession.session_score: Option<u8>`; `SummarySession.session_score: Option<u8>`.

- [ ] **Step 1: Write the failing test** (in `session.rs` tests):

```rust
#[test]
fn test_update_session_score_sets_and_validates() {
    let mut model = model_with_summary();

    update(
        &mut model,
        Event::Session(SessionEvent::UpdateSessionScore { score: Some(8) }),
    );
    assert!(model.last_error.is_none());
    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.session_score, Some(8));
    } else {
        panic!("Expected Summary state");
    }

    // Out of range is rejected, leaving the prior value intact.
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateSessionScore { score: Some(11) }),
    );
    if let SessionStatus::Summary(ref s) = model.session_status {
        assert_eq!(s.session_score, Some(8));
    }
}
```

- [ ] **Step 2: Run it, verify it fails**

Run: `cargo test -p intrada-core test_update_session_score_sets_and_validates`
Expected: FAIL — `UpdateSessionScore` variant and `session_score` field don't exist (compile error).

- [ ] **Step 3: Add the field to `SummarySession`** (find the struct in `session.rs`; it currently ends with `completion_status`). Add:

```rust
    #[serde(default)]
    pub session_score: Option<u8>,
```

- [ ] **Step 4: Add the field to `PracticeSession`** (`:86-96`), after `completion_status`:

```rust
    #[serde(default)]
    pub session_score: Option<u8>,
```

- [ ] **Step 5: Initialise it in `transition_to_summary`** (`:376-404`) — in the returned `SummarySession { ... }`, add `session_score: None,`.

- [ ] **Step 6: Append the event variant** to `SessionEvent` (next to `UpdateSessionNotes`, `:152-269`):

```rust
    UpdateSessionScore { score: Option<u8> },
```

- [ ] **Step 7: Handle it** — add a match arm beside `UpdateEntryScore`/`UpdateSessionNotes` in the session update fn:

```rust
SessionEvent::UpdateSessionScore { score } => {
    if let Some(s) = score {
        if !(validation::MIN_SCORE..=validation::MAX_SCORE).contains(&s) {
            return crux_core::render::render();
        }
    }
    let SessionStatus::Summary(ref mut summary) = model.session_status else {
        model.last_error = Some("Not in summary state".to_string());
        return crux_core::render::render();
    };
    summary.session_score = score;
    model.last_error = None;
    crux_core::render::render()
}
```

- [ ] **Step 8: Carry it into the saved session** — in the `SaveSession` handler (`:1044-1084`), add to the `PracticeSession { ... }` literal:

```rust
        session_score: summary.session_score,
```

- [ ] **Step 9: Run the test, verify green**

Run: `cargo test -p intrada-core test_update_session_score_sets_and_validates`
Expected: PASS.

- [ ] **Step 10: Compile the whole workspace** (shared core type touched):

Run: `cargo test -p intrada-core && cargo clippy -p intrada-core -- -D warnings`
Expected: PASS (any other constructors of `PracticeSession`/`SummarySession` now need the field — fix them to `session_score: None` if the compiler flags them).

- [ ] **Step 11: Commit**

```bash
git add crates/intrada-core/src/domain/session.rs
git commit -m "feat(core): add overall session score + UpdateSessionScore event"
```

---

### Task 3: Expose `session_score` in the ViewModel (core)

**Files:**
- Modify: `crates/intrada-core/src/model.rs` — `SummaryView` (`:362-368`), `PracticeSessionView` (`:275-285`)
- Modify: the view builder in `app.rs`/`model.rs` that constructs `SummaryView` from `SummarySession` and `PracticeSessionView` from `PracticeSession`
- Test: `model.rs` (or `app.rs`) tests

**Interfaces:**
- Consumes: `SummarySession.session_score`, `PracticeSession.session_score` (Task 2).
- Produces: `SummaryView.session_score: Option<u8>`, `PracticeSessionView.session_score: Option<u8>` (consumed by Swift in Task 7).

- [ ] **Step 1: Write the failing test** — drive a session to summary, set the score, assert the view exposes it:

```rust
#[test]
fn summary_view_exposes_session_score() {
    let mut model = model_with_summary();
    update(
        &mut model,
        Event::Session(SessionEvent::UpdateSessionScore { score: Some(7) }),
    );
    let view = view(&model); // the crate's ViewModel builder
    assert_eq!(view.summary.unwrap().session_score, Some(7));
}
```

(Use the crate's existing pattern for building the `ViewModel` in tests — match how other `view(&model)` / `app.view()` tests are written in this module.)

- [ ] **Step 2: Run it, verify it fails**

Run: `cargo test -p intrada-core summary_view_exposes_session_score`
Expected: FAIL — `SummaryView` has no `session_score` field.

- [ ] **Step 3: Add the field to `SummaryView`** (`:362-368`):

```rust
    pub session_score: Option<u8>,
```

- [ ] **Step 4: Add the field to `PracticeSessionView`** (`:275-285`):

```rust
    pub session_score: Option<u8>,
```

- [ ] **Step 5: Populate both** in their builders — set `session_score: summary.session_score` (and `session.session_score` for the persisted-session view).

- [ ] **Step 6: Run the test, verify green**

Run: `cargo test -p intrada-core summary_view_exposes_session_score`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/intrada-core/src/model.rs crates/intrada-core/src/app.rs
git commit -m "feat(core): expose session_score on summary + session views"
```

---

### Task 4: Round-trip coverage for the new field (core)

**Files:**
- Modify: `crates/intrada-core/src/domain/types.rs:432-468` (the existing session round-trip test)

**Interfaces:**
- Consumes: `PracticeSession.session_score` (Task 2).

- [ ] **Step 1: Extend the round-trip test** — in `save_session_persistence_op_round_trips_on_ffi_bincode_wire`, add `session_score: Some(8),` to the `PracticeSession { ... }` literal so a populated value crosses the bincode wire.

- [ ] **Step 2: Run it, verify green**

Run: `cargo test -p intrada-core save_session_persistence_op_round_trips_on_ffi_bincode_wire`
Expected: PASS (proves `session_score` survives the FFI bincode round-trip, #846).

- [ ] **Step 3: Commit**

```bash
git add crates/intrada-core/src/domain/types.rs
git commit -m "test(core): round-trip session_score on the FFI bincode wire"
```

---

### Task 5: Regenerate Swift bindings

**Files:**
- Generated (do not hand-edit): `ios/generated/SharedTypes/Sources/SharedTypes/SharedTypes.swift`

- [ ] **Step 1: Regenerate**

Run: `just ios-gen`
Expected: success; `SharedTypes.swift` now contains `case updateSessionScore(score: UInt8?)` in `SessionEvent` and a `sessionScore: UInt8?` field on `SummaryView`/`PracticeSessionView`.

- [ ] **Step 2: Sanity-check the diff**

Run: `git diff --stat ios/generated`
Expected: only generated changes for the new field/variant.

- [ ] **Step 3: Commit**

```bash
git add ios/generated
git commit -m "build(ios): regenerate bindings for session_score + UpdateSessionScore"
```

---

### Task 6: GRDB — add `session_score` column + rescale existing scores (iOS)

**Files:**
- Modify: `ios/Intrada/Core/LibraryStore.swift` — migrations (after `v3_session`, `:159-175`), `saveSession` (`:102-124`), the session load/row-mapping
- Test: `ios/IntradaTests/LibraryStoreMigrationTests.swift` (create)

**Interfaces:**
- Produces: `session.session_score` column; existing entry scores doubled (≤10).

- [ ] **Step 1: Write the failing upgrade-path test** — `ios/IntradaTests/LibraryStoreMigrationTests.swift`:

```swift
import GRDB
import XCTest
@testable import Intrada

final class LibraryStoreMigrationTests: XCTestCase {
  func testV5RescalesEntryScoresAndAddsSessionScore() throws {
    let queue = try DatabaseQueue()  // in-memory

    // Migrate only up to v3 (the pre-rescale schema), then seed a session
    // whose single entry scored 3 on the old 1–5 scale.
    try LibraryStore.migrator.migrate(queue, upTo: "v3_session")
    try queue.write { db in
      let entriesJSON = #"[{"id":"e1","itemId":"i1","itemTitle":"Scales","itemType":"exercise","position":0,"durationSecs":60,"status":"completed","score":3}]"#
      try db.execute(sql: """
        INSERT INTO session (id, started_at, completed_at, total_duration_secs,
          completion_status, session_notes, session_intention, entries, updated_at, deleted_at)
        VALUES ('s1','2026-01-01T00:00:00Z','2026-01-01T00:01:00Z',60,'completed',NULL,NULL,?, '2026-01-01T00:00:00Z',NULL)
        """, arguments: [entriesJSON])
    }

    // Run the remaining migrations (v4 add column, v5 rescale).
    try LibraryStore.migrator.migrate(queue)

    try queue.read { db in
      let row = try Row.fetchOne(db, sql: "SELECT entries, session_score FROM session WHERE id='s1'")!
      let entries: String = row["entries"]
      XCTAssertTrue(entries.contains("\"score\":6"), "old score 3 should rescale ×2 to 6")
      XCTAssertNil(row["session_score"] as String?, "session_score column exists, null for old rows")
    }
  }
}
```

- [ ] **Step 2: Run it, verify it fails**

Run (quit Xcode first — see `docs/ios-testing.md`): `just ios-test` (or the equivalent `xcodebuild test` for `LibraryStoreMigrationTests`).
Expected: FAIL — `LibraryStore.migrator` isn't accessible and migrations `v4`/`v5` don't exist.

- [ ] **Step 3: Make the migrator testable** — if `migrator` is a private local in `LibraryStore`, hoist it to a `static let migrator: DatabaseMigrator` (internal) built by a `static func makeMigrator()`, and have the init use `Self.migrator`. (Mirror the existing registration order; do not change v1–v3 bodies.)

- [ ] **Step 4: Append migration v4 (add column)** after `v3_session`:

```swift
migrator.registerMigration("v4_session_score") { db in
  try db.execute(sql: "ALTER TABLE session ADD COLUMN session_score INTEGER")
}
```

- [ ] **Step 5: Append migration v5 (rescale entry scores ×2)**:

```swift
migrator.registerMigration("v5_rescale_entry_scores") { db in
  let rows = try Row.fetchAll(db, sql: "SELECT id, entries FROM session")
  for row in rows {
    let id: String = row["id"]
    let json: String = row["entries"]
    guard var dtos = try? JSONDecoder().decode([StoredEntry].self, from: Data(json.utf8))
    else { continue }
    for i in dtos.indices {
      if let s = dtos[i].score { dtos[i].score = min(10, s &* 2) }
    }
    guard let data = try? JSONEncoder().encode(dtos),
          let rescaled = String(data: data, encoding: .utf8) else { continue }
    try db.execute(sql: "UPDATE session SET entries = ? WHERE id = ?", arguments: [rescaled, id])
  }
}
```

(`StoredEntry` is the existing private DTO at `LibraryStore.swift:263-280`; it must be visible to the migration closure — it already lives in the same file.)

- [ ] **Step 6: Persist & load `session_score`** — in `saveSession` (`:102-124`) add `session_score` to the column list, the `VALUES` placeholders, the `ON CONFLICT` set, and the `arguments` (`session.sessionScore.map(Int.init) as Any`). Then update the session **load** query (the `SELECT ... FROM session` that hydrates `PracticeSession`) to select `session_score` and map it into `PracticeSession(... sessionScore: row["session_score"])`.

- [ ] **Step 7: Run the migration test, verify green**

Run: `just ios-test` (filtered to `LibraryStoreMigrationTests`).
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add ios/Intrada/Core/LibraryStore.swift ios/IntradaTests/LibraryStoreMigrationTests.swift
git commit -m "feat(ios): rescale stored scores ×2 + persist session_score (GRDB v4/v5)"
```

---

### Task 7: Summary UI — 1–10 entry scorer + overall session scorer (iOS)

**Files:**
- Modify: `ios/Intrada/Views/Screens/SessionSummaryScreen.swift` — `scoreRow` (`:120-143`), insert an overall scorer between `noteField` and `controls`

**Interfaces:**
- Consumes: `SummaryView.session_score` (Task 3), `SessionEvent.updateSessionScore` (Task 5).

- [ ] **Step 1: Widen the per-entry scorer** — in `scoreRow` (`:124`) change the range and the a11y label; wrap so 10 dots fit the row width:

```swift
  return HStack(spacing: 5) {
    ForEach(1...10, id: \.self) { value in
```
and (`:142`):
```swift
    .accessibilityValue(score == 0 ? "not scored" : "\(score) of 10")
```

(If 10 dots are too wide at the current 18pt size, drop the dot to ~15pt and `spacing: 4`; verify on the simulator. This row is replaced by the ring tap-target in Phase 2/#1009, so keep changes minimal.)

- [ ] **Step 2: Add an overall-session scorer** — a new private view, inserted in `body` between `noteField` and `controls`:

```swift
  private func sessionScoreRow(_ summary: SummaryView) -> some View {
    let score = summary.sessionScore.map(Int.init) ?? 0
    return VStack(alignment: .leading, spacing: IntradaSpacing.controlGap) {
      Text("Overall")
        .font(IntradaFont.fieldLabel)
        .foregroundStyle(IntradaColor.inkSecondary)
      HStack(spacing: 5) {
        ForEach(1...10, id: \.self) { value in
          Button {
            let next: UInt8? = summary.sessionScore == UInt8(value) ? nil : UInt8(value)
            store.send(.session(.updateSessionScore(score: next)))
          } label: {
            Circle()
              .fill(score >= value ? AnyShapeStyle(IntradaColor.accent) : AnyShapeStyle(.clear))
              .frame(width: 18, height: 18)
              .overlay(Circle().stroke(IntradaColor.divider, lineWidth: 1.5)
                .opacity(score >= value ? 0 : 1))
          }
          .buttonStyle(.plain)
        }
      }
      .accessibilityElement(children: .ignore)
      .accessibilityLabel("Overall session score")
      .accessibilityValue(score == 0 ? "not scored" : "\(score) of 10")
    }
  }
```

(Use the real `IntradaFont`/`IntradaColor`/`IntradaSpacing` tokens that the file already imports; match the surrounding card styling — wrap in `.cardSurface()` like `noteField` if that's the existing pattern.)

- [ ] **Step 3: Wire it into `body`** — add `sessionScoreRow(summary)` to the `VStack` between `noteField` and `controls`.

- [ ] **Step 4: Build & exercise on the simulator** (per `docs/ios-testing.md`): launch `just ios-run`, drive a session to the summary, confirm the 1–10 per-entry dots and the overall scorer both set/clear and that a failed core write would surface (re-read `viewModel` after `send`). Capture a screenshot.

- [ ] **Step 5: Commit**

```bash
git add ios/Intrada/Views/Screens/SessionSummaryScreen.swift
git commit -m "feat(ios): 1–10 entry scorer + overall session score on the summary"
```

---

### Task 8: MasteryMeter interim correctness on the 0–10 scale (iOS)

> Throwaway-but-correct: #1009 replaces `MasteryMeter` with the ring next phase. This keeps Phase 1 self-consistent so library cards don't read "7 / 5" in the interim.

**Files:**
- Modify: `ios/Intrada/Views/Components/MasteryMeter.swift`

- [ ] **Step 1: Update the doc + caption + a11y for /10, keep the 5 bars proportional.** Change `steps` semantics so the meter maps a 0–10 `level` across its 5 bars, and the caption/label read out of 10:

```swift
  private func filled(_ index: Int) -> Bool {
    guard let level else { return false }
    // 5 bars across a 0–10 scale: each bar ≈ 2 points.
    return index < Int((Double(level) / 2.0).rounded())
  }

  private var caption: String {
    guard let level else { return "—" }
    return "\(level) / 10"
  }
```
and the accessibility label:
```swift
      level == nil ? "Not yet practised" : "Mastery \(level ?? 0) of 10")
```

- [ ] **Step 2: Update the preview** (`#Preview`) to iterate `1...10` (or a representative 1,3,6,8,10) so the preview reflects the new scale.

- [ ] **Step 3: Build to confirm it compiles** (`just ios` or a build of the Intrada scheme).

- [ ] **Step 4: Commit**

```bash
git add ios/Intrada/Views/Components/MasteryMeter.swift
git commit -m "fix(ios): MasteryMeter reads on the 0–10 scale (interim, pre-ring)"
```

---

### Task 9: Re-record + optimize snapshots (iOS)

**Files:**
- Modify: `ios/IntradaTests/__Snapshots__/ScreenSnapshotTests/testSessionSummaryCompleted.1.png`, `testSessionSummaryEndedEarly.1.png`, and any `LibraryItemCard`/library snapshot whose `MasteryMeter` changed
- Possibly modify: `ios/IntradaTests/ScreenSnapshotTests.swift` (preview fixtures `previewSummary` to carry a `sessionScore`)

- [ ] **Step 1: Give the summary preview a session score** — update the `Store.previewSummary` fixture so the recorded snapshot shows the overall scorer populated (e.g. `sessionScore: 8`). Match the existing fixture-construction pattern.

- [ ] **Step 2: Re-record** the affected snapshots (set `isRecording = true` per the project's snapshot convention, run, then revert), per `docs/ios-testing.md` and the Snapshot test hygiene rules in `CLAUDE.md`.

- [ ] **Step 3: Optimize + check**

Run: `just ios-snapshots-optimize && just ios-snapshots-check`
Expected: references shrink (alpha dropped), hygiene job passes (no orphans, under size ceiling).

- [ ] **Step 4: Run the snapshot suite green**

Run: `just ios-test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ios/IntradaTests
git commit -m "test(ios): re-record summary snapshots for the 1–10 scale + overall score"
```

---

### Task 10: Final gates + PR

- [ ] **Step 1: Core gates**

Run: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
Expected: all PASS.

- [ ] **Step 2: iOS build + tests** (quit Xcode first)

Run: `just ios-test`
Expected: PASS.

- [ ] **Step 3: Open the PR via the `ship` skill** (not raw `gh pr create`). PR body: scope = core + iOS (web/API deferred), closes #1008; Coverage line; note the ×2 rescale + the 1→10 scorer + the new overall score; flag MasteryMeter as interim (replaced by #1009). Post the `ship` self-review as a `gh pr comment`; end it with `Deferred items tracked: web/API parity on #1008`.

---

## Self-Review

**Spec coverage (#1008):**
- Per-item 1–5 → 0–10 (rated 1–10, None=unrated) → Task 1. ✓
- Overall session score (new field) → Tasks 2, 3, 7. ✓
- ×2 rescale of historical scores → Task 6 (v5). ✓
- Score input UI + labels → Task 7; interim display → Task 8. ✓
- Bridge round-trip + bindings → Tasks 4, 5. ✓
- Migration upgrade-path test → Task 6. ✓
- Core + iOS only, web/API deferred → Global Constraints + Task 10. ✓

**Open follow-ups (not this plan):** #1009 (ring replaces MasteryMeter — deletes the Task 8 interim) and the linked-exercises feature build on top.

**Type consistency:** `session_score: Option<u8>` (Rust) ↔ `sessionScore: UInt8?` (Swift) across `SummarySession`/`PracticeSession`/`SummaryView`/`PracticeSessionView`; event `UpdateSessionScore { score: Option<u8> }` ↔ `.updateSessionScore(score: UInt8?)`. Consistent throughout.
