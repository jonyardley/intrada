# Research: Practice Analytics Dashboard

**Feature**: 023-analytics-dashboard
**Date**: 2026-02-17

## Decision 1: Where to compute analytics

**Decision**: All analytics computation in `intrada-core/src/analytics.rs` as pure functions.

**Rationale**: Constitution Principle V requires the core to contain zero I/O. Analytics aggregation (streak, weekly totals, daily totals, item rankings, score trends) is pure computation over `Vec<PracticeSession>` — it belongs in core. The web shell only renders the pre-computed `AnalyticsView` from the ViewModel.

**Alternatives considered**:
- **Server-side API endpoint**: Would reduce client computation but adds API complexity, a new database query, and breaks the "no new persistence" constraint. Also, the session list is already fetched client-side.
- **Computation in the view layer**: Would violate Architecture Integrity — shells must not contain business logic. Also harder to unit test.

## Decision 2: Line chart rendering approach

**Decision**: Pure inline SVG via Leptos `view!` macro. No charting library.

**Rationale**: The codebase already renders inline SVG (see `bottom_tab_bar.rs`). A 28-point line chart is simple geometry — calculate x/y coordinates from data, emit `<polyline>` and `<circle>` elements. This adds zero dependencies, keeps the WASM bundle small, and integrates naturally with Tailwind CSS classes.

**Alternatives considered**:
- **plotly.rs**: Heavy dependency, designed for complex interactive charts. Overkill for a single 28-point line.
- **web-sys Canvas API**: More performant for complex rendering but harder to style with Tailwind, requires imperative DOM access (shell-only), and harder to test.
- **External JS charting library via wasm-bindgen**: Would require FFI bindings, increase bundle size, and introduce a non-Rust dependency.

## Decision 3: Data flow for analytics

**Decision**: Compute analytics from `model.sessions` inside the core's `update()` when `SessionsLoaded` fires, store result in `ViewModel.analytics`.

**Rationale**: Sessions are already loaded into the model when the app initialises. Computing analytics at load time (not on every render) is efficient and follows the existing pattern where `DataLoaded` triggers view model construction. The analytics view model is computed once and stored, not recomputed on each access.

**Alternatives considered**:
- **Lazy computation on route navigation**: Would require a new event (e.g., `AnalyticsRequested`) and deferred computation. More complex, and the data is already available — no benefit to deferring.
- **Separate fetch for analytics**: Would require a new API endpoint returning pre-aggregated data. Unnecessary since all session data is already client-side.

## Decision 4: "Today" for streak and weekly calculations

**Decision**: Pass "today" as a parameter to analytics functions (e.g., `compute_analytics(sessions, today: NaiveDate)`).

**Rationale**: Makes the pure functions deterministic and testable — tests can pass any date as "today" without mocking system time. The shell provides `chrono::Utc::now().date_naive()` when dispatching the event. This follows the core purity principle: no system calls in core.

**Alternatives considered**:
- **Use `Utc::now()` inside core**: Violates purity — system clock access is a side effect. Would make tests non-deterministic.
- **Hard-code UTC date**: Would work but hides the dependency. Explicit parameter is clearer.

## Decision 5: SVG chart dimensions and responsiveness

**Decision**: Fixed viewBox (e.g., `0 0 600 200`) with `width="100%"` and `preserveAspectRatio`. Chart scales to container width via CSS.

**Rationale**: SVG viewBox provides resolution-independent scaling. The container (Card component) handles responsive width via Tailwind. This is the standard approach for responsive SVG charts and requires no resize event listeners.

**Alternatives considered**:
- **Dynamic pixel dimensions via web-sys**: Would require reading container width from DOM — shell-only concern, more complex, fragile on resize.
- **CSS-only scaling with percentage widths**: SVG elements don't natively support percentage-based coordinates. viewBox is the correct SVG mechanism.
