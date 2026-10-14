# Intrada: Setup & Configuration

This document covers the external accounts, secrets, and configuration needed
to develop and ship Intrada. The app is a native SwiftUI iOS app, offline-first
with no server.

## 1. Sentry (Error reporting + APM)

DSNs are public (Sentry's security model expects them embedded in client
code), but the Swift SDK reads the value via an env var so the project can be
swapped without code changes.

| Surface | Sentry platform | DSN delivery |
|---------|----------------|--------------|
| Native iOS | Swift | `SENTRY_DSN_NATIVE` env var, baked in via `xcodegen` at build time (see [`docs/reference.md`, Environment variables](docs/reference.md#environment-variables)) |

Each event is tagged with `environment = development | production` (determined
at runtime), and `release = $GIT_SHA` when set at build time. Performance
tracing is sampled at 10% (`traces_sample_rate = 0.1`).

### Set the native iOS DSN locally

Add to your `.env` file at the repo root:

```
SENTRY_DSN_NATIVE=https://...@...ingest.de.sentry.io/...
```

`just ios` / `just ios-run` pick it up via `set dotenv-load`. Unset in CI, so
test/smoke runs send nothing.

### Verifying

Trigger a test event (e.g. force a panic in a debug build) and confirm it
appears in the Sentry project's Issues tab.

## 2. CI/CD Pipeline (GitHub Actions)

`.github/workflows/ci.yml` handles CI:

```
push to PR:   path filter, then only the jobs its paths touch:
              Rust: test, MSRV, clippy, fmt
              iOS: build and tests on the self-hosted Mac, reported through a
                hosted check; snapshot hygiene
              Android: build, unit and snapshot tests; UI tests on an
                emulator; snapshot hygiene
              always: security and hygiene, no em or en dashes, comment
                density (unless labelled comments-justified)
push to main: the Rust and Android jobs and security and hygiene every
              time, the iOS jobs when iOS paths changed, and an alert when
              main goes red
```

The apps ship separately: iOS through TestFlight (see §3 below), Android
through `release-play.yml` to Play internal testing, whose setup is not yet
written up (#2631).

### GitHub Actions secrets

| Secret | Service | Required for |
|--------|---------|-------------|
| `ASC_KEY_ID` | App Store Connect | TestFlight (native iOS) |
| `ASC_ISSUER_ID` | App Store Connect | TestFlight (native iOS) |
| `ASC_KEY_CONTENT_BASE64` | App Store Connect | TestFlight (native iOS) |
| `MATCH_GIT_URL` | fastlane match | TestFlight (native iOS) |
| `MATCH_GIT_BASIC_AUTHORIZATION` | fastlane match | TestFlight (native iOS) |
| `MATCH_PASSWORD` | fastlane match | TestFlight (native iOS) |
| `SENTRY_AUTH_TOKEN` | Sentry | Release tracking for TestFlight |
| `SENTRY_DSN_NATIVE` | Sentry | Crash reporting in TestFlight builds; a tagged run fails without it |

Set at: **GitHub repo → Settings → Secrets and variables → Actions**

## 3. Native iOS to TestFlight

The native SwiftUI app ships to TestFlight via
`.github/workflows/release-testflight.yml`, which runs on `workflow_dispatch`
or a `v*` tag, never per PR (#784, #984). Signing uses fastlane **match**; an
App Store Connect API key handles auth and upload. The workflow and its
`beta` lane in `fastlane/Fastfile`:

- builds a Release core, then flips the *generated* project to Manual signing
  with the match profile. `project.yml` holds no signing keys, so the
  simulator loop keeps Automatic signing.
- sets the build number to `github.run_number` and the version from the tag
  (`v0.3.0` gives `0.3.0`, which must be `X.Y.Z`); untagged runs keep
  `project.yml`'s version.
- refuses a tag when `SENTRY_DSN_NATIVE` is missing, and after the upload
  creates the Sentry release under the name the app reports (#1553).
- uploads for internal testing only.

**One-time setup, in order** (each step depends on the previous):

1. **App Store Connect**: accept the Developer Program License Agreement
   (Business → Agreements shows *Active*).
2. **Register the bundle id** `com.intrada.native` in Certificates, Identifiers
   & Profiles → Identifiers. No special capabilities needed.
3. **Create the app record**: Apps → + → New App (iOS, the bundle id, any SKU).
   *The API key cannot do this; it is one manual click.*
4. **Create the API key**: Users and Access → Integrations → **Team Keys** →
   role **App Manager**. Save the Key ID, Issuer ID, and the `.p8` (one-time
   download).
5. **Create a private certs repo**, e.g. `jonyardley/intrada-certificates`, and
   choose a strong `MATCH_PASSWORD`.
6. **Bootstrap match** (local, Ruby ≥ 3; system Ruby 2.6 is too old, use
   `rbenv`):
   ```bash
   bundle install              # then commit the generated Gemfile.lock
   MATCH_GIT_URL=<certs-repo> MATCH_PASSWORD=<pw> bundle exec fastlane match appstore
   ```
   Authenticate with your Apple ID when prompted. This generates + encrypts +
   pushes the Apple Distribution cert + App Store profile to the certs repo.
7. **Add the GitHub Actions secrets** (table above):
   ```bash
   base64 -i AuthKey_<KEYID>.p8 | pbcopy        # → ASC_KEY_CONTENT_BASE64
   printf 'USER:GITHUB_PAT' | base64 | pbcopy   # → MATCH_GIT_BASIC_AUTHORIZATION (repo read access)
   ```
8. **Run it**: in Actions, open the TestFlight release workflow and choose *Run workflow* (or push a
   `v*` tag). After processing, add yourself to an Internal Testing group in the
   app's TestFlight tab and install via the TestFlight app.

Local parity (after the one-time setup): `just testflight`.

## 4. Local Development

### Prerequisites

Install Rust first (mise's `cargo:` backend builds with it):

- Rust via [rustup](https://rustup.rs); `rust-toolchain.toml` pins the version
  and rustup installs it on first build

Then the rest of the dev toolchain (just, xcodegen, oxipng, typos, Ruby for
fastlane, pinned cargo tools like cargo-swift 0.9.0) is declared in
`mise.toml`. Install [mise](https://mise.jdx.dev) once and let it provision
everything:

```bash
brew install mise
mise install
```

Everything mise does not cover is listed in the README's
[prerequisites](README.md#prerequisites).

### Optional: sccache for faster fresh-worktree builds

`mise install` provisions the `sccache` binary, but it stays inactive until
you opt in: it isn't wired into `mise.toml`'s committed config, so nobody
gets it by surprise. Measured ~30% faster `just check` in a fresh worktree
with `target/` cold and the cache warm (#1206); no measurable overhead when
the cache is empty. Opt in per-machine:

```bash
cat > mise.local.toml <<'EOF'
[env]
RUSTC_WRAPPER = "sccache"
EOF
```

`mise.local.toml` is gitignored and merges over `mise.toml` automatically;
run `mise install` again if you added it after your first install. Default
cache size is 10G at `~/Library/Caches/Mozilla.sccache`; check usage with
`sccache --show-stats`. CI stays on Swatinem/rust-cache, not sccache, as the
target-dir cache it already runs beats a cold sccache cache without paying
for remote storage.

### One-time git config

```bash
# Hide the whole-tree swift format reformat commit from git blame
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

### Quick start

```bash
just ios          # iPhone, in Xcode
just android-run  # Android, on the emulator
```

## Checklist

Use this when setting up from scratch:

- [ ] Sentry native iOS project created
- [ ] `SENTRY_DSN_NATIVE` set locally for crash reporting (optional)
- [ ] Test event sent, visible in the Sentry project
- [ ] TestFlight signing bootstrapped (see §3) if shipping to testers
