# Intrada: Setup & Configuration

The external accounts, secrets and configuration needed to develop and ship
intrada's two offline-first apps. There is no server.

## 1. Sentry (Error reporting + APM)

DSNs are public by Sentry's design, but the iOS app reads its DSN from
`SENTRY_DSN_NATIVE`, baked in by `xcodegen` at build time
([environment variables](docs/reference.md#environment-variables)), so the
project can change without a code change.

Each event carries `environment` (development or production) and, when set at
build time, `release = $GIT_SHA`. Tracing samples 10%.

### Set the native iOS DSN locally

Add to `.env` at the repo root:

```
SENTRY_DSN_NATIVE=https://...@...ingest.de.sentry.io/...
```

`just ios` and `just ios-run` read it through `set dotenv-load`. CI leaves it
unset, so test runs send nothing. To verify, force a panic in a debug build and
look for it in the Sentry project's Issues tab.

## 2. CI/CD Pipeline (GitHub Actions)

`.github/workflows/ci.yml` runs on every PR the jobs its paths touch (Rust,
iOS on the self-hosted Mac, Android), plus security and hygiene every time. A
push to main reruns them and alerts when main goes red.

iOS ships through TestFlight (section 3). Android ships through
`release-play.yml` to Play internal testing, whose setup is not yet written up
(#2631).

### GitHub Actions secrets

| Secret | Service | Required for |
|--------|---------|-------------|
| `ASC_KEY_ID` | App Store Connect | TestFlight |
| `ASC_ISSUER_ID` | App Store Connect | TestFlight |
| `ASC_KEY_CONTENT_BASE64` | App Store Connect | TestFlight |
| `MATCH_GIT_URL` | fastlane match | TestFlight |
| `MATCH_GIT_BASIC_AUTHORIZATION` | fastlane match | TestFlight |
| `MATCH_PASSWORD` | fastlane match | TestFlight |
| `SENTRY_AUTH_TOKEN` | Sentry | Release tracking for TestFlight |
| `SENTRY_DSN_NATIVE` | Sentry | Crash reporting in TestFlight builds; a tagged run fails without it |

Set at: **GitHub repo → Settings → Secrets and variables → Actions**

## 3. Native iOS to TestFlight

`.github/workflows/release-testflight.yml` runs on `workflow_dispatch` or a
`v*` tag, never per PR (#784, #984). Signing uses fastlane **match**; an App
Store Connect API key handles auth and upload. The workflow and its `beta` lane
in `fastlane/Fastfile`:

- builds a Release core, then flips the *generated* project to Manual signing
  with the match profile, so the simulator keeps Automatic signing.
- sets the build number to `github.run_number` and the version from the tag
  (`v0.3.0` gives `0.3.0`); untagged runs keep `project.yml`'s version.
- refuses a tag when `SENTRY_DSN_NATIVE` is missing, and after the upload
  creates the Sentry release under the name the app reports (#1553).
- uploads for internal testing only.

**One-time setup, in order:**

1. **App Store Connect**: accept the Developer Program License Agreement
   (Business → Agreements shows *Active*).
2. **Register the bundle id** `com.intrada.native` in Certificates, Identifiers
   & Profiles → Identifiers. No special capabilities needed.
3. **Create the app record**: Apps → + → New App (iOS, the bundle id, any SKU).
   The API key cannot do this.
4. **Create the API key**: Users and Access → Integrations → **Team Keys** →
   role **App Manager**. Save the Key ID, Issuer ID, and the `.p8` (one-time
   download).
5. **Create a private certs repo**, e.g. `jonyardley/intrada-certificates`, and
   choose a strong `MATCH_PASSWORD`.
6. **Bootstrap match** locally, with the Ruby mise installs:
   ```bash
   bundle install              # then commit the generated Gemfile.lock
   MATCH_GIT_URL=<certs-repo> MATCH_PASSWORD=<pw> bundle exec fastlane match appstore
   ```
   Sign in with your Apple ID when asked. This pushes the encrypted
   distribution certificate and App Store profile to the certs repo.
7. **Add the GitHub Actions secrets** (table above):
   ```bash
   base64 -i AuthKey_<KEYID>.p8 | pbcopy        # → ASC_KEY_CONTENT_BASE64
   printf 'USER:GITHUB_PAT' | base64 | pbcopy   # → MATCH_GIT_BASIC_AUTHORIZATION (repo read access)
   ```
8. **Run it**: run the TestFlight release workflow in Actions, or push a `v*`
   tag. Then add yourself to an Internal Testing group in the app's TestFlight
   tab.

After the setup, `just testflight` does the same locally.

## 4. Local Development

Install Rust through [rustup](https://rustup.rs) first, since mise's `cargo:`
backend builds with it; `rust-toolchain.toml` pins the version. Then
`brew install mise && mise install` provisions everything `mise.toml` pins.
Everything else is in the README's [prerequisites](README.md#prerequisites).

### Optional: sccache for faster fresh-worktree builds

`mise install` provisions `sccache` but leaves it off. It made `just check`
about 30% faster in a fresh worktree (#1206). Opt in per machine:

```bash
cat > mise.local.toml <<'EOF'
[env]
RUSTC_WRAPPER = "sccache"
EOF
```

`mise.local.toml` is gitignored and merges over `mise.toml`; run `mise install`
again after adding it. CI stays on Swatinem/rust-cache.

### One-time git config

```bash
# Hide the whole-tree swift format commit from git blame
git config blame.ignoreRevsFile .git-blame-ignore-revs
```
