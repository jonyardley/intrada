# Feature flags

Server-resolved per-user flags for gating in-progress features. Single
mechanism for web + iOS; same code path everywhere.

Introduced in #751 (gates Goals). Use this doc when adding flag #2 onwards.

## When to use a flag

- Shipping a multi-PR feature you want internal-test before everyone sees it.
- Holding back a route or nav surface from production users while it's still
  rough.
- Beta cohort that should see something the rest of the user base doesn't.

Don't use a flag for:

- One-day toggles (just merge or revert).
- Per-environment config (use env vars + `option_env!` for that).
- A/B tests or % rollouts — graduate to GrowthBook / similar instead.

## Adding a flag

### 1. Data model

[`crates/intrada-core/src/domain/features.rs`](../crates/intrada-core/src/domain/features.rs):

```rust
pub struct FeatureFlags {
    #[serde(default)]
    pub goals: bool,
    #[serde(default)]
    pub my_new_flag: bool, // ← add here
}
```

`#[serde(default)]` matters: older clients running before the new field
existed must still deserialise the response cleanly.

### 2. Server resolver

[`crates/intrada-api/src/routes/features.rs`](../crates/intrada-api/src/routes/features.rs):

```rust
async fn get_features(auth: AuthUser) -> Result<Json<FeatureFlags>, ApiError> {
    Ok(Json(FeatureFlags {
        goals: resolve("goals", &auth),
        my_new_flag: resolve("my_new_flag", &auth),  // ← add line
    }))
}
```

The `resolve(name, &auth)` helper:
- Reads env var `INTRADA_FEATURE_FLAG_<NAME>_ALLOWLIST` (comma-separated user IDs).
- `AuthSource::Disabled` (dev mode without Clerk) → all flags ON.
  Solo-developer friction is the constraint there; the allowlist exists
  for real users.
- Empty / unset env var → flag OFF for everyone in prod.

### 3. UI gates

#### Route gating

[`crates/intrada-web/src/app.rs`](../crates/intrada-web/src/app.rs):

```rust
<Route path=path!("/my-feature") view=|| view! {
    <AuthenticatedShell>
        <FeatureGate select=|f| f.my_new_flag>
            <MyFeatureView />
        </FeatureGate>
    </AuthenticatedShell>
} />
```

`FeatureGate` (in `components/feature_gate.rs`) renders children when on,
skeleton while loading, redirects to `/library` when off.

#### Inline / nav gating

Tab bars and inline links use `<Show>`:

```rust
let my_flag_enabled = Signal::derive(move ||
    view_model.with(|vm| vm.features.as_ref().is_some_and(|f| f.my_new_flag))
);

<Show when=move || my_flag_enabled.get()>
    <A href="/my-feature">…</A>
</Show>
```

`is_some_and` is correct: while features are still loading (`None`),
nothing renders — avoids flashing the surface to non-allowlisted users.

### 4. E2E mock

[`e2e/fixtures/api-mock.ts`](../e2e/fixtures/api-mock.ts): bump the
`/api/features` response to include the new field. The current mock
returns all flags `true`; that stays as the default.

### 5. Roll out

1. Land the PR. Production goals stays visible because no env var is set
   yet; everyone passes the allowlist by default? **No** — empty/unset
   env var means *nobody* matches, so the flag is **off** in prod until
   you set the env var. Plan for that:
   - If you want goals **visible** by default while you work on a new
     flag, only the new flag gets an empty allowlist (off by default);
     existing flags remain governed by their own env vars.
2. Add the env var on Fly.io: `fly secrets set INTRADA_FEATURE_FLAG_MY_NEW_FLAG_ALLOWLIST=<your-clerk-user-id> -a intrada-api`.
3. Verify on web (production) and iOS (sim + device).
4. To roll out to more users: append their IDs to the env var, restart
   the app: `fly apps restart intrada-api`.

## Removing a flag

When a feature graduates to everyone:

1. Delete the field from `FeatureFlags`.
2. Remove the gate from the UI (route / nav / inline).
3. Delete the resolver branch in `routes/features.rs`.
4. Unset the env var: `fly secrets unset INTRADA_FEATURE_FLAG_<NAME>_ALLOWLIST -a intrada-api`.

Older clients holding stale code don't break — `#[serde(default)]` on
new flags + ignored-unknown-fields default on serde means the response
shape is forwards-compatible either way.

## Why server-resolved (not compile-time)?

- iOS ships as a TestFlight build. Build-time flags can't be flipped
  without a new build + review cycle.
- Same UI codebase across web + iOS — one mechanism, one place to look.
- The flag fetch (`GET /api/features` on app start) costs one round-trip
  per app launch. That cost is bounded and predictable.

## Why env-var allowlist (not a DB table)?

- Cheap: zero schema changes, zero settings UI, one env var per flag.
- Operates on Clerk user IDs, which is the identity we already trust.
- Restart-required to apply changes (`fly apps restart …`) — acceptable
  for the rollout cadence we have today (~weekly at most).

When the env-var approach gets unwieldy (~3+ flags AND/OR you want
self-service "join the beta" UX), graduate to one of:

- **DB-backed flags table** — flags column on the users table, plus an
  account-settings UI for self-service. The client surface
  (`FeatureFlags` struct + view-model + UI gates) stays identical; only
  `get_features` changes.
- **Hosted SaaS** (GrowthBook, LaunchDarkly, PostHog, Statsig) — drop-in
  SDKs for web and iOS, real % rollouts, audit trail. Right when you
  have 10+ flags or need real A/B segmentation.

## Testing

- **Unit-test the allowlist matcher** alongside the env var (see
  `routes/features.rs::tests`).
- **Integration test the endpoint shape** so a missing field gets caught
  early (see `tests/features_test.rs`).
- **Integration test the wired-up resolver path** when adding the first
  multi-flag complexity (auth-enabled harness — deferred from #751
  for now).

## Files touched per new flag

- `crates/intrada-core/src/domain/features.rs` — add field
- `crates/intrada-api/src/routes/features.rs` — add env const + resolver line
- `crates/intrada-web/src/app.rs` — wrap routes in `<FeatureGate>`
- `crates/intrada-web/src/components/{bottom_tab_bar,app_header}.rs` — gate nav
- `e2e/fixtures/api-mock.ts` — extend mock response
- `docs/feature-flags.md` — this file, if conventions change

Adding flag #2 should land in one PR < 200 lines.
