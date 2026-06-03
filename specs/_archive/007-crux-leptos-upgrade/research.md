# Research: Crux & Leptos Upgrade

**Feature**: 007-crux-leptos-upgrade
**Date**: 2026-02-14

## 1. Leptos 0.7 → 0.8 Migration

### Decision: Upgrade from leptos 0.7 to 0.8 (latest 0.8.15)

**Rationale**: Leptos 0.8 is a small, incremental upgrade. The release notes describe changes as "technically semver-breaking but should not meaningfully affect user code." The major driver was Axum 0.8 support (irrelevant to CSR apps). For this CSR-only project, there are no show-stopping breaking changes — most existing code compiles as-is.

**Alternatives considered**:
- Stay on 0.7: Rejected — no benefit; 0.8 brings idiomatic improvements and bug fixes
- Wait for 0.9/1.0: Rejected — no timeline; 0.8 is current stable and actively maintained

### Breaking Changes (CSR-relevant only)

| Change | Impact on this project | Action required |
|--------|----------------------|-----------------|
| `LocalResource` no longer exposes `SendWrapper` in API | None — project does not use `LocalResource` | None |
| `LeptosOptions` / `ConfFile` lost `Default` impl | None — CSR mode does not use these types | None |
| `PossibleRouteMatch` made dyn-safe | None — project does not use advanced routing | None |
| Axum 0.8 reexported types | None — CSR mode does not use Axum | None |
| Server function error handling (`FromServerFnError`) | None — no server functions | None |

### New Idiomatic Patterns (recommended adoption)

| Pattern | Before (0.7) | After (0.8 idiomatic) | Files affected |
|---------|-------------|----------------------|----------------|
| Two-way binding | `prop:value=move \|\| v.get()` + `on:input=move \|ev\| { v.set(event_target_value(&ev)); }` | `bind:value=v` | text_field.rs, text_area.rs |
| Typed event targets | `event_target_value(&ev)` | `on:input:target=move \|ev\| { v.set(ev.target().value()); }` | text_field.rs, text_area.rs (if not using bind:value) |

### Unchanged APIs (verified compatible)

- `#[component]` macro, `#[prop(optional)]`, `#[prop(default)]`
- `RwSignal::new()`, `.get()`, `.set()`
- `Callback<ev::MouseEvent>`, `.run()`
- `Children` type (`Box<dyn FnOnce() -> AnyView + Send>`)
- `view!` macro syntax (all existing patterns)
- `on:click=move |ev| { ... }`, `on:input`, `on:submit`
- `leptos::mount::mount_to_body(App)`
- `leptos::prelude::*` (additive: now also exports `SignalSetter`, `TextProp`)
- `.into_any()` for type unification
- Conditional rendering: `move || { Some(view!) or None }`
- List rendering: `.map().collect::<Vec<_>>()`
- Generic components with `F: Fn(ev::MouseEvent) + 'static`
- Dynamic ARIA attributes (`aria-describedby`, `aria-invalid`)
- `send_wrapper = "0.6"` (project's own usage, unrelated to Leptos internals)

## 2. crux_core Version Status

### Decision: No version change required — already on latest published version

**Rationale**: The project already uses `crux_core = "0.17.0-rc2"`, which is the latest version published to crates.io as of 2026-02-14. No newer release exists. The 0.17.0-rc3 is in draft PR status and not yet published.

**Alternatives considered**:
- Pin to Git main branch: Rejected — unstable; PR #490 (rc3 prep) still in draft
- Downgrade to stable 0.16.2: Rejected — would require restoring deprecated Capability trait; 0.17 Command API is the project's established pattern
- Wait for 0.17.0 final: Not blocking — rc2 is stable enough; can upgrade when final ships

### API Stability Verification

All project patterns verified unchanged in 0.17.0-rc2:

| API | Status |
|-----|--------|
| `impl App for Intrada` (Event, Model, ViewModel, Effect) | Stable |
| `Command::notify_shell(op).into()` | Stable |
| `Command::all([...])` | Stable |
| `crux_core::render::render()` | Stable |
| `impl crux_core::Effect for Effect {}` | Stable |
| `From<Request<Op>> for Effect` impls | Stable |
| `impl Operation for StorageEffect { type Output = (); }` | Stable |
| `Core::<Intrada>::new()` | Stable |
| `core.process_event(event)` → `Vec<Effect>` | Stable |
| `core.view()` → `ViewModel` | Stable |

### Upcoming Changes (post-rc2, not yet published)

- `Core::map_with(app, model)` — new constructor alternative (non-breaking)
- Dependency bumps: `facet_generate` 0.13→0.14, `convert_case` 0.10→0.11
- New testing helpers: `expect_one_effect()`, `expect_one_event()` (non-breaking)

## 3. Secondary Dependency Assessment

| Dependency | Current | Action | Reason |
|-----------|---------|--------|--------|
| `send_wrapper` | 0.6 | Keep | Not affected by Leptos 0.8 changes |
| `wasm-bindgen` | (latest) | Verify compatibility | May need minor bump for Leptos 0.8 |
| `console_error_panic_hook` | (latest) | No change | Unrelated to framework upgrade |
| `getrandom` | 0.3 | Verify compatibility | WASM feature flag may need check |
| `serde` / `serde_json` | 1.x | No change | Stable, unrelated |
| `chrono` | 0.4 | No change | Unrelated |
| `ulid` | (latest) | No change | Unrelated |

## 4. Build Tooling

| Tool | Current | Action | Reason |
|------|---------|--------|--------|
| `trunk` | 0.21.x | Verify compatibility | Must support Leptos 0.8 WASM builds |
| `tailwindcss` | v4 standalone | No change | Unrelated to framework upgrade |

## 5. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Leptos 0.8 introduces subtle runtime behaviour change | Low | Medium | Manual smoke test all views post-upgrade |
| `bind:value` behaves differently from `prop:value` + `on:input` | Low | Low | Test form validation thoroughly |
| WASM binary size increases >20% | Very low | Medium | Measure before/after with `trunk build --release` |
| Transitive dependency conflict | Low | Low | Clean `cargo update` + verify lockfile |
