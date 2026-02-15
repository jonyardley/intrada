# Research: Improve CI/CD Pipeline

## R1: Single Workflow vs Separate Workflows

**Decision**: Merge `deploy.yml` into `ci.yml` as a conditional deploy job with `needs` on all CI jobs.

**Rationale**:
- `needs:` guarantees deploy never runs unless all CI jobs pass — the core safety requirement
- Artifact sharing works natively within a single workflow run (no cross-workflow hacks)
- One workflow run in the GitHub UI shows the complete pipeline picture
- The `if: github.event_name == 'push' && github.ref == 'refs/heads/main'` condition cleanly skips deploy on PRs

**Alternatives considered**:
- `workflow_run` trigger: Requires explicit conclusion check (`if: github.event.workflow_run.conclusion == 'success'`), artifacts need cross-workflow parameters (`run-id`, `github-token`), two separate UI entries. More complex, more fragile.
- Reusable workflows (`workflow_call`): Overkill for a single CI pipeline with one deploy target. Adds indirection without benefit.

## R2: Debug vs Release Build

**Decision**: Always build release (`trunk build --release`) in the `wasm-build` CI job.

**Rationale**:
- E2E tests work identically with release builds — Playwright tests interact with the DOM, not WASM internals
- Eliminates the current duplicate build (debug in CI, release in deploy)
- Catches release-mode-only issues (integer overflow panics, conditional compilation) before merge
- Time impact is modest (~30-60 seconds) for a project of this size — not a meaningful PR feedback regression

**Alternatives considered**:
- Debug for PRs, release for main only: Saves ~30-60s on PRs but reintroduces complexity (conditional build mode) and misses release-mode-only bugs on PRs
- Keep separate builds: Status quo — wastes compute and risks "works in CI, fails in deploy" mismatches

## R3: Artifact Passing

**Decision**: Use `actions/upload-artifact@v4` / `actions/download-artifact@v4` within the same workflow run.

**Rationale**: Works natively across jobs in a single workflow. The existing `wasm-build` → `e2e` pattern already proves this. The new `deploy` job follows the identical pattern. No `run-id`, no `github-token`, no cross-workflow parameters needed.

## R4: Cloudflare Workers Deployment from Pre-built Artifact

**Decision**: Download the artifact to `crates/intrada-web/dist/` (the path `wrangler.toml` already references) and run `wrangler-action` from the repo root.

**Rationale**:
- `wrangler.toml` specifies `directory = "crates/intrada-web/dist"` — placing the artifact there means zero config changes
- The deploy job only needs `actions/checkout@v4` (for `wrangler.toml` + `worker.js`) and the artifact download
- No Rust toolchain, no trunk, no Tailwind CLI required in the deploy job — dramatically simpler and faster
