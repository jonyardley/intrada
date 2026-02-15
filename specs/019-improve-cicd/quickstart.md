# Quickstart: Improve CI/CD Pipeline

## Prerequisites

- GitHub repository with CI secrets `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` configured
- A pull request and a merge to main to test both paths

## Verification Steps

### V1: PR Pipeline (no deploy)

1. Push a commit to a PR targeting main
2. Verify all CI jobs run in parallel: test, clippy, fmt, wasm-build, wasm-test
3. Verify e2e runs after wasm-build completes
4. Verify **no deploy job** appears in the workflow run
5. Verify the wasm-build job uses `trunk build --release`

### V2: Main Pipeline (deploy gated on CI)

1. Merge a PR to main
2. Verify all CI jobs run as in V1
3. Verify the deploy job appears but waits for all other jobs to complete
4. Verify deploy job succeeds and the site is updated at the Cloudflare Workers URL

### V3: Deploy Blocked on CI Failure

1. Push a commit to main that would fail a check (e.g., formatting violation)
2. Verify the failing job is reported clearly
3. Verify the deploy job is skipped (not run)

### V4: Artifact Reuse

1. In a successful main pipeline run, verify:
   - The `wasm-build` job uploads the `web-dist` artifact
   - The `e2e` job downloads and uses the same artifact
   - The `deploy` job downloads and deploys the same artifact
   - No second WASM build occurs anywhere in the pipeline

### V5: No `deploy.yml` Exists

1. Verify `.github/workflows/deploy.yml` has been deleted
2. Verify only `.github/workflows/ci.yml` exists in the workflows directory

### V6: Existing Tests Pass

1. `cargo test` — all 142+ tests pass
2. `cargo clippy -- -D warnings` — zero warnings
3. `cargo fmt --all -- --check` — no formatting issues
