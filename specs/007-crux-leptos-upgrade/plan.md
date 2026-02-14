# Implementation Plan: Crux & Leptos Upgrade

**Branch**: `007-crux-leptos-upgrade` | **Date**: 2026-02-14 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/007-crux-leptos-upgrade/spec.md`

## Summary

Upgrade Leptos from 0.7 to 0.8.x (latest 0.8.15) in the web shell crate, adopt the new `bind:value` idiomatic pattern for form two-way binding, and verify all existing functionality is preserved. crux_core remains at 0.17.0-rc2 (already the latest published version). This is a low-risk upgrade — Leptos 0.8 introduces no breaking changes for CSR-only apps, and the primary code improvement is replacing verbose `prop:value` + `on:input` patterns with the new `bind:value` directive.

## Technical Context

**Language/Version**: Rust stable (1.75+, 2021 edition)
**Primary Dependencies**: crux_core 0.17.0-rc2 (unchanged), leptos 0.7 → 0.8 (upgrade), send_wrapper 0.6 (unchanged)
**Storage**: N/A (in-memory stub data; no persistence changes)
**Testing**: cargo test (82 existing tests), cargo clippy, trunk build (WASM)
**Target Platform**: WASM (wasm32-unknown-unknown) via trunk 0.21.x + native (CLI)
**Project Type**: Workspace — pure core + platform shells (CLI, web)
**Performance Goals**: WASM binary size must not increase >20% (NFR-002)
**Constraints**: Zero compiler/clippy warnings, all CI gates pass
**Scale/Scope**: 10 component files + 6 view files in intrada-web; 2 Cargo.toml files to update

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Code Quality — Clarity over cleverness | ✅ Pass | `bind:value` simplifies 2-line patterns to 1 line; no new complexity introduced |
| I. Code Quality — Single Responsibility | ✅ Pass | No new modules; existing module boundaries unchanged |
| I. Code Quality — Consistent Style | ✅ Pass | All changes follow rustfmt + clippy; adopts idiomatic Leptos 0.8 patterns |
| I. Code Quality — No Dead Code | ✅ Pass | `event_target_value` imports removed where `bind:value` replaces them |
| I. Code Quality — Explicit over Implicit | ✅ Pass | `bind:value` is an explicit framework directive; no hidden side effects |
| I. Code Quality — Type Safety | ✅ Pass | No `any` types; all existing type constraints preserved |
| II. Testing — Coverage | ✅ Pass | All 82 existing tests must pass; test modifications allowed per clarification |
| II. Testing — Independence | ✅ Pass | No test execution order changes |
| II. Testing — Meaningful Assertions | ✅ Pass | Tests verify behaviour, not implementation |
| III. UX Consistency — Design System | ✅ Pass | No visual changes; CSS/Tailwind classes untouched |
| III. UX Consistency — Interaction Patterns | ✅ Pass | Form binding behaviour preserved |
| III. UX Consistency — Accessibility | ✅ Pass | ARIA attributes preserved (FR-011) |
| IV. Performance — Measurement | ✅ Pass | WASM binary size measured before/after (NFR-002) |

**Gate result**: ✅ All principles pass. No violations to justify.

## Project Structure

### Documentation (this feature)

```text
specs/007-crux-leptos-upgrade/
├── plan.md              # This file
├── research.md          # Phase 0 output — migration research
├── quickstart.md        # Phase 1 output — upgrade guide
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── intrada-core/           # Pure Crux core — NO CHANGES (crux_core 0.17.0-rc2 unchanged)
│   ├── Cargo.toml
│   └── src/
│       ├── app.rs          # App trait, Effect/Event/Command — unchanged
│       ├── model.rs        # Model/ViewModel — unchanged
│       ├── domain/         # Piece/Exercise handlers — unchanged
│       ├── validation.rs   # Validation rules — unchanged
│       └── error.rs        # Error types — unchanged
│
├── intrada-web/            # Leptos web shell — UPGRADE TARGET
│   ├── Cargo.toml          # leptos version bump 0.7 → 0.8
│   └── src/
│       ├── main.rs         # mount_to_body — verify compatible
│       ├── app.rs          # Root component — verify compatible
│       ├── core_bridge.rs  # Effect processing — verify compatible
│       ├── types.rs        # SharedCore type — verify compatible
│       ├── helpers.rs      # event_target_value usage — may simplify
│       ├── validation.rs   # Client-side validation — unchanged
│       ├── data.rs         # Stub data — unchanged
│       ├── components/     # 10 component files — adopt bind:value where applicable
│       │   ├── text_field.rs    # MODIFY: bind:value + aria preservation
│       │   ├── text_area.rs     # MODIFY: bind:value + aria preservation
│       │   ├── button.rs        # Verify compatible
│       │   ├── card.rs          # Verify compatible
│       │   ├── back_link.rs     # Verify compatible
│       │   ├── page_heading.rs  # Verify compatible
│       │   ├── field_label.rs   # Verify compatible
│       │   ├── type_badge.rs    # Verify compatible
│       │   ├── app_header.rs    # Verify compatible
│       │   ├── app_footer.rs    # Verify compatible
│       │   ├── form_field_error.rs  # Verify compatible
│       │   ├── library_item_card.rs # Verify compatible
│       │   └── mod.rs          # Verify compatible
│       └── views/          # 6 view files — adopt bind:value in forms
│           ├── add_piece.rs     # MODIFY: bind:value in form inputs
│           ├── add_exercise.rs  # MODIFY: bind:value in form inputs
│           ├── edit_piece.rs    # MODIFY: bind:value in form inputs
│           ├── edit_exercise.rs # MODIFY: bind:value in form inputs
│           ├── library_list.rs  # Verify compatible (search input)
│           ├── detail.rs        # Verify compatible (no form inputs)
│           └── mod.rs           # No change
│
├── intrada-cli/            # CLI shell — NO CHANGES
│   └── ...
│
└── Cargo.toml              # Workspace manifest — no change (crux_core stays 0.17.0-rc2)
```

**Structure Decision**: Existing workspace structure preserved. Changes scoped entirely to `crates/intrada-web/` (Cargo.toml + source files). No new files created, no files deleted.

## Implementation Strategy

### Phase 1: Version Bump & Compile

1. Update `leptos` version in `crates/intrada-web/Cargo.toml` from `"0.7"` to `"0.8"`
2. Run `cargo update -p leptos` to resolve new dependency tree
3. Build workspace: `cargo build --workspace`
4. Fix any compilation errors (expected: none based on research)

### Phase 2: Idiomatic Pattern Migration

1. **text_field.rs**: Replace `prop:value` + `on:input` + `event_target_value` with `bind:value`
   - Preserve `aria-describedby` and `aria-invalid` attributes
   - Preserve `required` attribute
   - Preserve `placeholder` attribute
2. **text_area.rs**: Same `bind:value` migration
   - Preserve `aria-describedby` and `aria-invalid` attributes
3. **View form files** (add_piece, add_exercise, edit_piece, edit_exercise): If any inline `prop:value` + `on:input` patterns exist outside TextField/TextArea components, migrate those too
4. **library_list.rs**: Check search input for `prop:value` + `on:input` pattern; migrate if present

### Phase 3: Verification

1. `cargo fmt --all` — formatting
2. `cargo clippy --workspace -- -D warnings` — zero warnings
3. `cargo test --workspace` — all tests pass
4. `trunk build` — WASM build succeeds
5. Measure WASM binary size: compare before/after (must be <120% of baseline)
6. Manual smoke test: serve with `trunk serve`, verify all views

### Key Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Adopt `bind:value`? | Yes | Eliminates boilerplate; is the 0.8 recommended pattern for two-way binding |
| Adopt `on:input:target`? | No | `bind:value` makes it unnecessary; simpler migration |
| Adopt `erase_components`? | No (out of scope) | Build tooling optimization, not a dependency upgrade concern |
| Upgrade crux_core? | No change needed | Already on latest published version (0.17.0-rc2) |
| Upgrade send_wrapper? | No change needed | Project's own usage; unrelated to Leptos 0.8 changes |
| Upgrade trunk? | Only if required | Out of scope unless Leptos 0.8 requires it |

## Complexity Tracking

> No constitution violations detected. Table not applicable.
