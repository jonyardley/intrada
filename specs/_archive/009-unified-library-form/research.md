# Research: Unified Library Item Form

**Feature**: 009-unified-library-form
**Date**: 2026-02-14

## Research Topics

### 1. Leptos Tab Component Pattern

**Decision**: Build a custom `TypeTabs` component using native HTML/ARIA tab semantics within Leptos's reactive model.

**Rationale**: Leptos 0.7 CSR does not have a built-in tab widget library. The tab bar for this feature is minimal (exactly 2 tabs, static labels, simple active/inactive states). A custom component is the simplest approach and avoids adding a third-party dependency for a trivial UI element.

**Alternatives considered**:
- **Third-party component library (e.g., leptos-use, leptonic)**: Rejected — these libraries target later Leptos versions and add unnecessary dependency weight for a two-tab bar. The project already has no UI component library dependency.
- **Radix-style headless tabs port**: Rejected — over-engineered for exactly two static tabs. No dynamic tab addition/removal needed.

**Implementation pattern**:
- Two `<button>` elements inside a `<div role="tablist">`
- Each button: `role="tab"`, `aria-selected={is_active}`, `aria-controls="tabpanel-{type}"`, `tabindex={if active "0" else "-1"}`
- Keyboard handling: `on:keydown` captures ArrowLeft/ArrowRight to switch focus between tabs
- Active tab styling: indigo-600 bg, white text, bottom border (consistent with project's Primary button color)
- Inactive tab styling (interactive): white bg, slate-300 border, hover:bg-slate-50
- Inactive tab styling (display-only): slate-100 bg, slate-400 text, cursor-default, no hover effect

### 2. Conditional Field Rendering in Leptos

**Decision**: Use Leptos's reactive `{move || if ... { Some(view!) } else { None }}` pattern for conditionally showing/hiding the Category field.

**Rationale**: This is the idiomatic Leptos pattern. The `category` signal persists regardless of whether the field is rendered, preserving the value when switching tabs (FR-005). No `on:mount`/`on:cleanup` side effects are needed — the signal simply holds the value.

**Alternatives considered**:
- **CSS display:none**: Rejected — the hidden field would still be in the DOM, which could confuse screen readers and adds unnecessary DOM nodes. Also requires more complex ARIA management.
- **`<Show>` component**: Acceptable alternative to the `move || if` pattern. Both work equivalently in Leptos 0.7 CSR. The `if` pattern is used for consistency with the existing codebase (e.g., detail.rs uses this pattern for optional fields).

### 3. Reactive Composer Label Change

**Decision**: Use a derived signal for the Composer field's label and required prop based on `active_tab`.

**Rationale**: The Composer field is always visible but its label changes between "Composer *" (Piece) and "Composer" (Exercise). Rather than rendering two separate TextField components and toggling visibility, use a single TextField with reactive props:
- `label` is computed from `active_tab`: `move || if active_tab.get() == ItemType::Piece { "Composer *" } else { "Composer" }`
- `required` is computed from `active_tab`: `move || active_tab.get() == ItemType::Piece`

**Challenge**: The existing `TextField` component takes `label: &'static str` and `required: bool` (not signals). This will need to be addressed:
- **Option A**: Change TextField to accept `MaybeSignal<&'static str>` for label — but this changes the component API for all callsites.
- **Option B**: Render two TextFields for composer and conditionally show one based on active_tab — slightly redundant but zero API changes needed.
- **Option C**: Keep label as static and update the `*` indicator separately. Use `"Composer"` as the label with a separate `" *"` span that conditionally renders.

**Selected**: Option B — render two TextFields. This avoids modifying the reusable TextField component API, keeps the change scope minimal, and is already a pattern the codebase supports. The two TextFields share the same `composer` signal via `bind:value`, so switching between them is seamless.

### 4. Unified Validation Function Design

**Decision**: Replace `validate_piece_form()` and `validate_exercise_form()` with a single `validate_library_form()` that takes an `ItemType` discriminator.

**Rationale**: The two existing functions share ~80% of their logic (title, notes, bpm, tempo_marking, tags validation are identical). The only differences are:
- Piece: composer is required
- Exercise: composer is optional, category is validated

A single function with a type parameter eliminates this duplication.

**Alternatives considered**:
- **Keep both functions and call from unified form**: Rejected — maintaining two near-identical functions when the form is already unified would be inconsistent.
- **Trait-based validation**: Rejected — over-engineered for two variants with minimal differences.

### 5. Edit Form Route Unification

**Decision**: Unify edit routes from `/pieces/:id/edit` and `/exercises/:id/edit` to `/library/:id/edit`.

**Rationale**: Since the edit form determines the item type from the ViewModel data (not from the URL), having type-specific routes is unnecessary. The unified route is simpler and consistent with the existing `/library/:id` detail route pattern.

**Impact**: The detail view's edit button href changes from a type-conditional expression to a simple `format!("/library/{}/edit", id)`. This simplifies the detail view code.

**Alternatives considered**:
- **Keep separate edit routes but point to unified component**: Rejected — adds route duplication for no functional benefit. The component would be identical regardless of which route loaded it.

### 6. Tab Switching and Error Clearing

**Decision**: Clear all validation errors when the user switches between tabs (FR-007).

**Rationale**: Validation errors from one tab configuration may not be relevant after switching. For example, a "Composer is required" error on the Piece tab is not applicable on the Exercise tab. Leaving stale errors would confuse users.

**Implementation**: In the tab switch handler (TypeTabs on_change callback), immediately call `errors.set(HashMap::new())` before or after updating `active_tab`.

**Alternatives considered**:
- **Re-validate on tab switch**: Rejected — would show errors before the user has a chance to interact with the form in the new tab configuration. Overly aggressive.
- **Clear only type-specific errors**: Rejected — more complex to implement and the simple "clear all" approach is standard UX for form mode switches.
