#!/usr/bin/env bash
# The Sentry release name is a contract across two files that nothing else
# reads together (#1611): the app composes it at runtime, and the release lane
# composes it off the uploaded binary, both from the same three Info.plist
# fields. Swap a separator on one side and crashes arrive filed under a release
# nobody created, which reads as Sentry being odd rather than as a one-character
# edit.
#
# The lane is read all the way through: its variables are resolved back to the
# keys they were read from, so renaming one or reindenting the block is fine.
# The app is read for shape and fields only, deliberately. Resolving its
# interpolated names back to keys the same way was tried, and it failed on an
# ordinary tidy (hoisting the plist reads into locals), which is the false
# failure #1611 says is worse than no check at all. What the app composes is
# already pinned by ios/IntradaTests/SentryReleaseTests.swift; what nothing
# else reads is whether the two files still agree.
#
# So one gap is knowingly left: the app swapping which of its two version keys
# feeds which argument passes here, because both keys are still read and the
# literal is unchanged. Every other way the two can drift apart fails.
#
# The Android app and the Play lane hold the same contract (#2565), read the
# same way: Reporting.kt for shape and fields (its composition is pinned by
# ReportingTest.kt), release-play.yml resolved back to where Gradle takes
# each field from.
#
# The file paths are overridable so the self-test can point the same parser
# at fixtures that disagree (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

swift_file="${RELEASE_NAME_SWIFT:-ios/Intrada/Core/SentryRelease.swift}"
workflow_file="${RELEASE_NAME_WORKFLOW:-.github/workflows/release-testflight.yml}"
kotlin_file="${RELEASE_NAME_KOTLIN:-android/app/src/main/kotlin/com/intrada/android/core/Reporting.kt}"
play_file="${RELEASE_NAME_PLAY_WORKFLOW:-.github/workflows/release-play.yml}"

contract="CFBundleIdentifier@CFBundleShortVersionString+CFBundleVersion"
android_contract="packageName@versionName+versionCode"

app_file="$swift_file"
lane_file="$workflow_file"
pair_contract="$contract"

fail() {
  printf '\nBlocked: %s\n\n' "$1" >&2
  cat <<EOF >&2
The release name must be composed the same way in both:

  $app_file
  $lane_file

as the app's identifier, then its version, then its build number, joined by
an at sign and a plus ($pair_contract).

If the format is changing deliberately, change both sides and the contract
line in scripts/check-release-name.sh together.

EOF
  exit 1
}

for f in "$swift_file" "$workflow_file" "$kotlin_file" "$play_file"; do
  [ -f "$f" ] || fail "$f is missing, so the release name contract cannot be read."
done

# ── The contract, read once per platform ────────────────────────────────────

contract_re='^([A-Za-z]+)([^A-Za-z])([A-Za-z]+)([^A-Za-z])([A-Za-z]+)$'

read_contract() {
  [[ $1 =~ $contract_re ]] || fail "the contract line $1 in this script is not three fields and two separators."
  contract_sep1="${BASH_REMATCH[2]}"
  contract_sep2="${BASH_REMATCH[4]}"
  contract_keys=$(printf '%s\n%s\n%s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[3]}" "${BASH_REMATCH[5]}" | sort | tr '\n' ' ')
}

read_contract "$contract"

# ── The app side: shape and fields ──────────────────────────────────────────

# Flattened, so a reformat that wraps the arguments differently still reads.
swift_flat=$(tr '\n\t' '  ' <"$swift_file" | tr -s ' ')

swift_literal_re='"\\\(([A-Za-z0-9_]+)\)([^"\\]*)\\\(([A-Za-z0-9_]+)\)([^"\\]*)\\\(([A-Za-z0-9_]+)\)"'
swift_hits=$({ printf '%s' "$swift_flat" | grep -oE "$swift_literal_re" || true; } | wc -l | tr -d ' ')
if [ "$swift_hits" != "1" ]; then
  fail "$swift_file holds $swift_hits string literals composing three fields, not exactly one."
fi

[[ $swift_flat =~ $swift_literal_re ]] || fail "$swift_file composes no three-field release name."
swift_sep1="${BASH_REMATCH[2]}"
swift_sep2="${BASH_REMATCH[4]}"

if [ "$swift_sep1" != "$contract_sep1" ] || [ "$swift_sep2" != "$contract_sep2" ]; then
  fail "$swift_file joins the three fields with '$swift_sep1' and '$swift_sep2', not '$contract_sep1' and '$contract_sep2'."
fi

# bundle.bundleIdentifier is Foundation's accessor for CFBundleIdentifier, so
# it counts as reading that key.
swift_keys=$({ printf '%s' "$swift_flat" | grep -oE 'bundleIdentifier|"CF[A-Za-z]+"' || true; } |
  tr -d '"' | sed 's/^bundleIdentifier$/CFBundleIdentifier/' | sort -u | tr '\n' ' ')

if [ "$swift_keys" != "$contract_keys" ]; then
  fail "$swift_file reads the identity fields [ ${swift_keys}], not [ ${contract_keys}]."
fi

# ── The release lane side: resolved in full ─────────────────────────────────

# Whole lines only, so a `note_release=` elsewhere in a 200 line workflow is
# not read as a second attempt at the same assignment.
lane_assignment() {
  local file="$1" lines hits
  lines=$({ grep -E '^[[:space:]]*release="[^"]*"[[:space:]]*$' "$file" || true; } |
    sed 's/^[[:space:]]*//; s/[[:space:]]*$//')
  hits=$(printf '%s\n' "$lines" | grep -c . || true)
  if [ "$hits" != "1" ]; then
    fail "$file holds $hits lines assigning the release name, not exactly one."
  fi
  printf '%s\n' "$lines" | head -1
}

lane_assign=$(lane_assignment "$workflow_file")
lane_re='^release="\$\{?([A-Za-z0-9_]+)\}?([^$]*)\$\{?([A-Za-z0-9_]+)\}?([^$]*)\$\{?([A-Za-z0-9_]+)\}?"$'
[[ $lane_assign =~ $lane_re ]] || fail "$workflow_file composes the release name from something other than three shell variables: $lane_assign"
lane_parts=("${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}" "${BASH_REMATCH[3]}" "${BASH_REMATCH[4]}" "${BASH_REMATCH[5]}")

lane_key_for() {
  local var="$1" key
  key=$(grep -E "^[[:space:]]*$var=" "$workflow_file" | head -1 | grep -oE 'CF[A-Za-z]+' | head -1 || true)
  if [ -z "$key" ]; then
    fail "$workflow_file interpolates \$$var into the release name but reads no Info.plist key into it."
  fi
  printf '%s' "$key"
}

# One substitution per assignment: an assignment takes only the last one's
# status, so a combined line would swallow an early resolution failure.
lane_k1=$(lane_key_for "${lane_parts[0]}")
lane_k2=$(lane_key_for "${lane_parts[2]}")
lane_k3=$(lane_key_for "${lane_parts[4]}")
lane_name="${lane_k1}${lane_parts[1]}${lane_k2}${lane_parts[3]}${lane_k3}"

# ── The comparison ──────────────────────────────────────────────────────────

if [ "$lane_name" != "$contract" ]; then
  fail "the lane composes $lane_name, which the app does not. The app composes the pinned $contract."
fi

echo "✓ release name: both iPhone sides compose $contract"

# ── Android: the app side, shape and fields ─────────────────────────────────

app_file="$kotlin_file"
lane_file="$play_file"
pair_contract="$android_contract"
read_contract "$android_contract"

kotlin_flat=$(tr '\n\t' '  ' <"$kotlin_file" | tr -s ' ')

kotlin_literal_re='"\$\{?([A-Za-z0-9_]+)\}?([^"$]*)\$\{?([A-Za-z0-9_]+)\}?([^"$]*)\$\{?([A-Za-z0-9_]+)\}?"'
kotlin_hits=$({ printf '%s' "$kotlin_flat" | grep -oE "$kotlin_literal_re" || true; } | wc -l | tr -d ' ')
if [ "$kotlin_hits" != "1" ]; then
  fail "$kotlin_file holds $kotlin_hits string templates composing three fields, not exactly one."
fi

[[ $kotlin_flat =~ $kotlin_literal_re ]] || fail "$kotlin_file composes no three-field release name."
kotlin_sep1="${BASH_REMATCH[2]}"
kotlin_sep2="${BASH_REMATCH[4]}"

if [ "$kotlin_sep1" != "$contract_sep1" ] || [ "$kotlin_sep2" != "$contract_sep2" ]; then
  fail "$kotlin_file joins the three fields with '$kotlin_sep1' and '$kotlin_sep2', not '$contract_sep1' and '$contract_sep2'."
fi

# longVersionCode is PackageInfo's accessor for the version code from API 28.
kotlin_keys=$({ printf '%s' "$kotlin_flat" | grep -oE '\.(packageName|versionName|longVersionCode|versionCode)([^A-Za-z0-9_]|$)' || true; } |
  sed -E 's/[^A-Za-z]//g; s/^longVersionCode$/versionCode/' | sort -u | tr '\n' ' ')

if [ "$kotlin_keys" != "$contract_keys" ]; then
  fail "$kotlin_file reads the identity fields [ ${kotlin_keys}], not [ ${contract_keys}]."
fi

# ── Android: the Play lane side, resolved in full ───────────────────────────

play_assign=$(lane_assignment "$play_file")
play_var='\$\{?([A-Za-z0-9_]+)(#[^}]*)?\}?'
play_re="^release=\"${play_var}([^\$]*)${play_var}([^\$]*)${play_var}\"\$"
[[ $play_assign =~ $play_re ]] || fail "$play_file composes the release name from something other than three shell variables: $play_assign"
play_vars=("${BASH_REMATCH[1]}" "${BASH_REMATCH[4]}" "${BASH_REMATCH[7]}")
play_strips=("${BASH_REMATCH[2]}" "${BASH_REMATCH[5]}" "${BASH_REMATCH[8]}")
play_seps=("${BASH_REMATCH[3]}" "${BASH_REMATCH[6]}")

env_expr() {
  { grep -E "^[[:space:]]*$1:[[:space:]]*\\$\\{\\{.*\\}\\}" "$play_file" || true; } | head -1 |
    sed -E 's/^[^$]*\$\{\{[[:space:]]*//; s/[[:space:]]*\}\}.*$//'
}

# Gradle builds with the tag as its version name, less this prefix.
gradle_name_strip=$({ grep -oE 'ANDROID_VERSION_NAME="\$\{ANDROID_VERSION_NAME(#[^}]*)?\}"' "$play_file" || true; } |
  head -1 | sed -E 's/^ANDROID_VERSION_NAME="\$\{ANDROID_VERSION_NAME//; s/\}"$//')
gradle_name_expr=$(env_expr ANDROID_VERSION_NAME)
gradle_code_expr=$(env_expr ANDROID_VERSION_CODE)
[ -n "$gradle_name_expr" ] || fail "$play_file sets no ANDROID_VERSION_NAME for Gradle, so the version name cannot be resolved."
[ -n "$gradle_code_expr" ] || fail "$play_file sets no ANDROID_VERSION_CODE for Gradle, so the version code cannot be resolved."

play_field_for() {
  local var="$1" strip="$2" line expr
  line=$({ grep -E "^[[:space:]]*$var=" "$play_file" || true; } | head -1)
  if [ -n "$line" ]; then
    case "$line" in
      *applicationId*) printf 'packageName' ;;
      *) fail "$play_file interpolates \$$var into the release name but reads no applicationId into it." ;;
    esac
    return
  fi
  expr=$(env_expr "$var")
  if [ -z "$expr" ] && [ "$var" = "GITHUB_REF_NAME" ]; then
    expr="github.ref_name"
  fi
  [ -n "$expr" ] || fail "$play_file interpolates \$$var into the release name but sets it from nothing Gradle reads."
  if [ "$expr" = "$gradle_code_expr" ]; then
    printf 'versionCode'
  elif [[ $gradle_name_expr == *"$expr"* ]]; then
    if [ "$strip" != "$gradle_name_strip" ]; then
      fail "$play_file strips '$strip' from \$$var in the release name, but Gradle's version name strips '$gradle_name_strip'."
    fi
    printf 'versionName'
  else
    fail "$play_file interpolates \$$var ($expr) into the release name, which is neither the version name nor the version code Gradle builds with."
  fi
}

play_k1=$(play_field_for "${play_vars[0]}" "${play_strips[0]}")
play_k2=$(play_field_for "${play_vars[1]}" "${play_strips[1]}")
play_k3=$(play_field_for "${play_vars[2]}" "${play_strips[2]}")
play_name="${play_k1}${play_seps[0]}${play_k2}${play_seps[1]}${play_k3}"

if [ "$play_name" != "$android_contract" ]; then
  fail "the Play lane composes $play_name, which the app does not. The app composes the pinned $android_contract."
fi

echo "✓ release name: both Android sides compose $android_contract"
exit 0
