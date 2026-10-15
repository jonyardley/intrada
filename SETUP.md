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
`release-play.yml` to Play internal testing (section 4).

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
| `ANDROID_UPLOAD_KEYSTORE_BASE64` | Upload keystore | Signing the Play bundle and the tester APK |
| `ANDROID_UPLOAD_KEYSTORE_PASSWORD` | Upload keystore | Signing |
| `ANDROID_UPLOAD_KEY_ALIAS` | Upload keystore | Signing (`upload`) |
| `ANDROID_UPLOAD_KEY_PASSWORD` | Upload keystore | Signing |
| `PLAY_SERVICE_ACCOUNT_JSON` | Google Cloud and Play Console | Uploading to Play internal testing |
| `SENTRY_DSN_ANDROID` | Sentry | Crash reporting in Play builds; a tagged run fails without it |
| `GRADLE_ENCRYPTION_KEY` | Gradle | Encrypting the Gradle cache in Android CI and the Play release |

Repo variables: `SENTRY_PROJECT_ANDROID` (the Sentry project slug, for the
Play release and its native symbols) and, only until the first rollout,
`PLAY_RELEASE_STATUS=draft`.

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

## 4. Native Android to Play internal testing

`.github/workflows/release-play.yml` runs on Ubuntu on the same `v*` tag as
TestFlight, or by hand (#2499). It builds a signed bundle and APK, keeps both
as run artifacts, and uploads the bundle to the internal track through the
`android internal` lane in `fastlane/Fastfile`. The version code is the
workflow's run number and the version name comes from the tag. On a tag it also
creates the Sentry release and uploads the native symbols (#2517).

One key signs everything: the upload key is also the app signing key Play
holds, so a Play build installs over a sideloaded APK and a tester keeps their
notebook.

**One-time setup, in order** (done for intrada in October 2026):

1. **Upload key.** Make it with Android Studio's `keytool`, since the Mac's
   own has no Java behind it, and keep it outside the repo:
   ```bash
   "/Applications/Android Studio.app/Contents/jbr/Contents/Home/bin/keytool" \
     -genkeypair -keystore upload.jks -alias upload -keyalg RSA -keysize 2048 -validity 10000
   ```
2. **Signing secrets** (table above):
   ```bash
   base64 -i upload.jks | gh secret set ANDROID_UPLOAD_KEYSTORE_BASE64
   gh secret set ANDROID_UPLOAD_KEY_ALIAS --body upload
   gh secret set ANDROID_UPLOAD_KEYSTORE_PASSWORD
   gh secret set ANDROID_UPLOAD_KEY_PASSWORD
   ```
   Plus `SENTRY_DSN_ANDROID` (with `printf %s`, no trailing newline) and the
   `SENTRY_PROJECT_ANDROID` variable.
3. **Play Console app.** Create it with package `com.intrada.android`. The API
   cannot do this.
4. **Play App Signing with your own key, before any bundle goes up.** Internal
   testing → Create new release → Change app signing key → Export and upload a
   key from Java keystore. Run the `pepk.jar` command it shows with Android
   Studio's `java` and upload the zip. Check: App integrity → App signing shows
   the same SHA-256 for the app signing key and the upload key, and it matches
   `keytool -printcert -jarfile app-release.aab` on a CI bundle.
5. **First bundle by hand.** Run the workflow with upload switched off:
   `gh workflow run release-play.yml -f upload=false`. Download the bundle from
   the run, upload it to the internal release, add a testers list, and roll it
   out. Taking it from the workflow keeps its version code below every later
   run. Complete the advertising ID declaration (Policy → App content) with
   *No* if Play asks.
6. **Service account.** In Google Cloud: enable the **Google Play Android
   Developer API**, create a service account, add a JSON key, and store it:
   `gh secret set PLAY_SERVICE_ACCOUNT_JSON < key.json`, then delete the file.
   No Google Cloud roles are needed.
7. **Its Play permissions.** Play Console → Users and permissions → invite the
   service account's email. Tick **View app information and download bulk
   reports (read-only)** as well as **Release apps to testing tracks** and
   **Manage testing tracks**. Without the first, every upload fails with
   *The caller does not have permission* and nothing says which permission is
   missing.
8. **Run it**: `gh workflow run release-play.yml`, or push a `v*` tag. Testers
   join from the opt-in link on the internal testing track's Testers tab.

## 5. Local Development

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
