#!/usr/bin/env bash
# Prove the two bash gates still bite (#1597, #1611). A gate nobody has watched
# fail is not a gate: each check below is run against an input built to break
# it, and against the near misses that must stay green, so the pass is evidence
# and not just silence. Every case here should fail if a line of the check it
# covers is deleted; a case that passes either way is worse than no case.

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

passed=0
failures=0

expect() {
  local want="$1" name="$2"
  shift 2
  local out status
  set +e
  out=$("$@" 2>&1)
  status=$?
  set -e
  if [ "$status" = "$want" ]; then
    passed=$((passed + 1))
  else
    failures=$((failures + 1))
    printf '  ✗ %s: expected exit %s, got %s\n' "$name" "$want" "$status" >&2
    printf '%s\n' "$out" | sed 's/^/      /' >&2
  fi
}

# ── The link check ──────────────────────────────────────────────────────────

sandbox="$work/links"
mkdir -p "$sandbox/docs/deep"
cd "$sandbox"
git init -q -b main .
git config user.email "test@example.com"
git config user.name "Hygiene self-test"
git config commit.gpgsign false

printf '# Target\n' >docs/target.md
printf '# Spare\n' >docs/spare.md
printf '# Read me\n\nSee [the target](docs/target.md).\n' >README.md
printf 'See [what went](../nowhere-at-all.md).\n' >docs/stale.md
for i in $(seq 1 50); do printf 'line %s of prose.\n' "$i"; done >docs/long.md
git add -A
git commit -qm "base"

# A remote-tracking ref, so the checks that leave LINK_CHECK_BASE unset
# exercise the branch detection and the origin/main fallback rather than
# exiting early on an unresolvable base.
git update-ref refs/remotes/origin/main main
git checkout -qb branch

link_check() {
  LINK_CHECK_BASE=main bash "$repo_root/scripts/check-links.sh"
}

link_check_bare() {
  bash "$repo_root/scripts/check-links.sh"
}

write_page() {
  printf '%s\n' "$@" >docs/deep/page.md
  git add docs/deep/page.md
  git commit -qm "page" --allow-empty
}

write_page '# Page' 'See [the target](../target.md).'
expect 0 "a relative link resolved from the linking file's own directory" link_check

write_page '# Page' 'See [the target](/docs/target.md).'
expect 0 "a repo-root path" link_check

write_page '# Page' 'See [the target](../target.md#a-heading) and [this page](#local).'
expect 0 "fragments on a real file, and an anchor with no path" link_check

write_page '# Page' 'See [the site](https://example.invalid/nothing) and [mail](mailto:a@b.c).'
expect 0 "external URLs, which are somebody else's server" link_check

write_page '# Page' 'Wrapped prose that runs on for a while and then' 'carries [the target](../target.md) onto a second line.'
expect 0 "a valid link after a reflow" link_check

write_page '# Page' '```markdown' '[an example](docs/does-not-exist.md)' '```'
expect 0 "a dangling link inside a fenced code block" link_check

write_page '# Page' 'Inline `[an example](docs/does-not-exist.md)` in a code span.'
expect 0 "a dangling link inside an inline code span" link_check

write_page '# Page' 'See [the target](../gone.md).'
expect 1 "a dangling relative link" link_check

write_page '# Page' 'See [the target](/docs/gone.md).'
expect 1 "a dangling repo-root path" link_check

write_page '# Page' '![a diagram](../gone.png)'
expect 1 "a dangling image" link_check

write_page '# Page' 'See [the target][ref].' '' '[ref]: ../gone.md'
expect 1 "a dangling reference definition" link_check

write_page '# Page' 'See [the target](../gone.md).'
expect 1 "the base derived from the branch name rather than the environment" link_check_bare
expect 0 "the documented bypass, on the same state" env SKIP_LINK_CHECK=1 bash "$repo_root/scripts/check-links.sh"

# A link far down a file, after an earlier hunk: the only case that notices if
# the hunk-header arithmetic drifts by a line.
write_page '# Page' 'Nothing broken on this page.'
awk 'NR == 5 { print "line 5, edited."; next } { print }' docs/long.md >"$work/long" && mv "$work/long" docs/long.md
awk 'NR == 40 { print "See [the target](gone-from-here.md)." } { print }' docs/long.md >"$work/long" && mv "$work/long" docs/long.md
git add docs/long.md
git commit -qm "edit line 5 and add a link at line 40"
expect 1 "a dangling link 40 lines down, after an earlier hunk" link_check

git checkout -q main -- docs/long.md
git commit -qm "put the long file back"
expect 0 "the long file restored" link_check

# The working tree, not HEAD: a link is written before it is committed, and an
# uncommitted edit above one must not renumber what gets read.
printf 'See [the target](../gone.md).\n' >>docs/deep/page.md
expect 1 "a dangling link that is not committed yet" link_check
git checkout -q -- docs/deep/page.md
expect 0 "the same file once the link is taken out again" link_check

printf 'a new first line\nand another\nand a third\n' | cat - docs/stale.md >"$work/stale" && mv "$work/stale" docs/stale.md
git add docs/stale.md
git commit -qm "prepend three lines above an old dangling link"
expect 0 "lines added above a link that was already broken and was not touched" link_check
git checkout -q main -- docs/stale.md
expect 0 "those lines taken out again in the working tree" link_check
git checkout -q HEAD -- docs/stale.md

# Files the branch removes, which the changed-lines half cannot see.
git rm -q docs/spare.md
git commit -qm "remove a file nothing links to"
expect 0 "removing a file no link points at" link_check

git mv docs/target.md docs/renamed.md
git commit -qm "rename the linked file"
expect 1 "a rename that leaves an untouched file pointing at the old path" link_check

git mv docs/renamed.md docs/target.md
git commit -qm "put the name back"
expect 0 "the rename undone" link_check

# Where the check runs at all: on main it does not, on a branch cut from the
# same commit it does, and on a detached HEAD it cannot tell, so it does not.
git checkout -q main
printf '\nAnd [what went](docs/vanished.md).\n' >>README.md
git add README.md
git commit -qm "a dangling link on main"
expect 0 "a dangling link on main, which this gate does not police" link_check_bare
git checkout -qb sidebranch
expect 1 "the same commit on a branch, which it does" link_check_bare
git checkout -q --detach
expect 0 "a detached HEAD, where there is no branch to derive a base from" link_check_bare

cd "$repo_root"

# ── The release name check ──────────────────────────────────────────────────

swift_fixture() {
  cat >"$1" <<EOF
enum SentryRelease {
  static func name(bundleId: String?, shortVersion: String?, buildNumber: String?) -> String? {
    guard let bundleId, let shortVersion, let buildNumber else { return nil }
    return "\\(bundleId)$2\\(shortVersion)$3\\(buildNumber)"
  }

  static func name(for bundle: Bundle) -> String? {
    name(
      bundleId: bundle.bundleIdentifier,
      shortVersion: bundle.object(forInfoDictionaryKey: "$4") as? String,
      buildNumber: bundle.object(forInfoDictionaryKey: "$5") as? String)
  }
}
EOF
}

lane_fixture() {
  cat >"$1" <<EOF
      - name: Create the Sentry release for this build
        run: |
          key() { /usr/libexec/PlistBuddy -c "Print :\$1" "\$plist"; }
          identifier="\$(key CFBundleIdentifier)"
          short_version="\$(key $4)"
          build_number="\$(key $5)"
          release="\$identifier$2\$short_version$3\$build_number"
          echo "Sentry release: \$release"
EOF
}

release_check() {
  RELEASE_NAME_SWIFT="$1" RELEASE_NAME_WORKFLOW="$2" \
    bash "$repo_root/scripts/check-release-name.sh"
}

good_swift="$work/Good.swift"
good_lane="$work/good-lane.yml"
swift_fixture "$good_swift" "@" "+" CFBundleShortVersionString CFBundleVersion
lane_fixture "$good_lane" "@" "+" CFBundleShortVersionString CFBundleVersion
expect 0 "two sides that agree" release_check "$good_swift" "$good_lane"

swapped_swift="$work/Swapped.swift"
swift_fixture "$swapped_swift" "+" "@" CFBundleShortVersionString CFBundleVersion
expect 1 "the app swapping its separators" release_check "$swapped_swift" "$good_lane"

swapped_lane="$work/swapped-lane.yml"
lane_fixture "$swapped_lane" "+" "@" CFBundleShortVersionString CFBundleVersion
expect 1 "the lane swapping its separators" release_check "$good_swift" "$swapped_lane"

expect 1 "both sides agreeing on a format that is not the contract" \
  release_check "$swapped_swift" "$swapped_lane"

reordered_lane="$work/reordered-lane.yml"
lane_fixture "$reordered_lane" "@" "+" CFBundleVersion CFBundleShortVersionString
expect 1 "the lane reading the version fields in the other order" release_check "$good_swift" "$reordered_lane"

wrong_field_swift="$work/WrongField.swift"
swift_fixture "$wrong_field_swift" "@" "+" CFBundleShortVersionString CFBundleName
expect 1 "the app reading a field that is not in the contract" release_check "$wrong_field_swift" "$good_lane"

reflowed_swift="$work/Reflowed.swift"
cat >"$reflowed_swift" <<'EOF'
enum SentryRelease {
  // A reformat: the arguments wrap differently and a comment sits in the middle.
  static func name(
    bundleId: String?,
    shortVersion: String?,
    buildNumber: String?
  ) -> String? {
    guard let bundleId, let shortVersion, let buildNumber else { return nil }
    return "\(bundleId)@\(shortVersion)+\(buildNumber)"
  }

  static func name(for bundle: Bundle) -> String? {
    name(
      bundleId: bundle.bundleIdentifier,
      shortVersion: bundle.object(
        forInfoDictionaryKey: "CFBundleShortVersionString") as? String,
      buildNumber: bundle.object(
        forInfoDictionaryKey: "CFBundleVersion") as? String)
  }
}
EOF
expect 0 "the app after a reformat" release_check "$reflowed_swift" "$good_lane"

hoisted_swift="$work/Hoisted.swift"
cat >"$hoisted_swift" <<'EOF'
enum SentryRelease {
  static func name(bundleId: String?, shortVersion: String?, buildNumber: String?) -> String? {
    guard let bundleId, let shortVersion, let buildNumber else { return nil }
    return "\(bundleId)@\(shortVersion)+\(buildNumber)"
  }

  static func name(for bundle: Bundle) -> String? {
    let short = bundle.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
    let build = bundle.object(forInfoDictionaryKey: "CFBundleVersion") as? String
    return name(bundleId: bundle.bundleIdentifier, shortVersion: short, buildNumber: build)
  }
}
EOF
expect 0 "the app after hoisting the plist reads into locals" release_check "$hoisted_swift" "$good_lane"

renamed_vars_lane="$work/renamed-vars-lane.yml"
cat >"$renamed_vars_lane" <<'EOF'
      - name: Create the Sentry release for this build
        run: |
              key() { /usr/libexec/PlistBuddy -c "Print :$1" "$plist"; }
              app_id="$(key CFBundleIdentifier)"
              marketing="$(key CFBundleShortVersionString)"
              build="$(key CFBundleVersion)"
              release="${app_id}@${marketing}+${build}"
              echo "created $release"
EOF
expect 0 "the lane after renaming its shell variables and reindenting" \
  release_check "$good_swift" "$renamed_vars_lane"

second_mention_lane="$work/second-mention-lane.yml"
cat >"$second_mention_lane" <<'EOF'
      - name: Create the Sentry release for this build
        run: |
          key() { /usr/libexec/PlistBuddy -c "Print :$1" "$plist"; }
          identifier="$(key CFBundleIdentifier)"
          short_version="$(key CFBundleShortVersionString)"
          build_number="$(key CFBundleVersion)"
          release="$identifier@$short_version+$build_number"
          sentry-cli releases finalize "$release" && note_release="$release"
EOF
expect 0 "the lane mentioning the release again further down" \
  release_check "$good_swift" "$second_mention_lane"

expect 0 "the files this repo actually ships" bash "$repo_root/scripts/check-release-name.sh"

# ── The faint ink check ─────────────────────────────────────────────────────

faint="$work/faint"
mkdir -p "$faint/Views/Components" "$faint/DesignSystem" "$faint/Views/Screens"
faint_check() {
  FAINT_INK_ROOT="$faint" bash "$repo_root/scripts/check-faint-ink.sh"
}

printf 'static let inkFaint = Color(hex: 0xA99C8C)\nlet x = IntradaColor.inkFaint\n' >"$faint/DesignSystem/Theme.swift"
printf 'Image(systemName: "x").foregroundStyle(IntradaColor.inkFaintIcon)\n' >"$faint/Views/Glyph.swift"
printf 'Rectangle().fill(IntradaColor.inkFainter)\n' >"$faint/Views/Tick.swift"
printf 'Text("b")  // not inkFaint: it fails AA\n' >"$faint/Views/Note.swift"
expect 0 "the token itself, a glyph token, the fainter token and a comment" faint_check

printf 'Text(meta).foregroundStyle(IntradaColor.inkFaint)\n' >"$faint/Views/Row.swift"
expect 1 "faint ink on a meta line" faint_check
rm "$faint/Views/Row.swift"

printf 'Text(meta)\n  .foregroundStyle(\n    IntradaColor.inkFaint\n  )\n' >"$faint/Views/Wrapped.swift"
expect 1 "faint ink with the modifier wrapped over lines" faint_check
rm "$faint/Views/Wrapped.swift"

printf 'var tint: Color = IntradaColor.inkFaint\n' >"$faint/Views/Components/SectionHeader.swift"
expect 1 "faint ink on a section title" faint_check
rm "$faint/Views/Components/SectionHeader.swift"

expect 2 "a root that is not there" env FAINT_INK_ROOT="$work/nowhere" bash "$repo_root/scripts/check-faint-ink.sh"

expect 0 "the screens this repo actually ships" bash "$repo_root/scripts/check-faint-ink.sh"

# ── The theme value check ───────────────────────────────────────────────────

themed="$work/themed"
themed_allow="$work/themed-allow.txt"
mkdir -p "$themed/DesignSystem" "$themed/Views"
: >"$themed_allow"
theme_check() {
  THEME_VALUES_ROOT="$themed" THEME_VALUES_ALLOWLIST="$themed_allow" \
    bash "$repo_root/scripts/check-theme-values.sh"
}
printf 'static let ink = Color(hex: 0x2A2622)\nstatic let card: CGFloat = 16\nlet x = Text("a").padding(16).foregroundStyle(.red)\n' >"$themed/DesignSystem/Theme.swift"
printf 'VStack(spacing: IntradaSpacing.card) {}\n  .padding(.horizontal, IntradaSpacing.card)\n  .foregroundStyle(IntradaColor.ink)\n  .background(Color.clear)\nLinearGradient(colors: [IntradaColor.shadow, .clear], startPoint: .top, endPoint: .bottom)\nHStack(spacing: 0) { Spacer(minLength: 0) }.padding(0)\n  .padding(EdgeInsets(top: 0, leading: IntradaSpacing.card, bottom: 0, trailing: IntradaSpacing.card))\n' >"$themed/Views/Row.swift"
expect 0 "theme tokens, zero, clear, and raw values inside the theme file" theme_check

theme_case() {
  local want="$1" name="$2" file="$3" body="$4"
  printf '%s\n' "$body" >"$file"
  expect "$want" "$name" theme_check
  rm "$file"
}
theme_case 1 "a typed colour on iPhone" "$themed/Views/Tint.swift" 'Text("a").foregroundStyle(Color(red: 0.9, green: 0.1, blue: 0.1))'
theme_case 1 "a named system colour on iPhone" "$themed/Views/Tint.swift" 'Text("a").foregroundStyle(.orange)'
theme_case 1 "a typed padding on iPhone" "$themed/Views/Gap.swift" 'Text("a").padding(.vertical, 10)'
theme_case 1 "a typed stack spacing on iPhone" "$themed/Views/Gap.swift" 'VStack(alignment: .leading, spacing: 6) {}'
theme_case 1 "typed edge insets on iPhone" "$themed/Views/Gap.swift" 'Text("a").padding(.init(top: 4, leading: 0, bottom: 0, trailing: 0))'
theme_case 1 "typed EdgeInsets on iPhone" "$themed/Views/Gap.swift" 'Text("a").padding(EdgeInsets(top: 0, leading: 12, bottom: 0, trailing: 0))'
theme_case 1 "a typed spacer length on iPhone" "$themed/Views/Gap.swift" 'HStack { Spacer(minLength: 12) }'
theme_case 1 "a named shadow colour on iPhone" "$themed/Views/Tint.swift" 'Card().shadow(color: .black, radius: 4)'
theme_case 1 "a named colour in a gradient on iPhone" "$themed/Views/Tint.swift" 'LinearGradient(colors: [IntradaColor.ink, .white], startPoint: .top, endPoint: .bottom)'
theme_case 1 "the accent colour on iPhone" "$themed/Views/Tint.swift" 'Text("a").foregroundStyle(.accentColor)'
theme_case 1 "a tertiary style on iPhone" "$themed/Views/Tint.swift" 'Text("a").foregroundStyle(.tertiary)'

gap_file="$themed/Views/Gap.swift"
printf 'HStack(spacing: 2) {}\n' >"$gap_file"
printf '%s\tspacing: 2\t1\t#2706\n' "$gap_file" >"$themed_allow"
expect 0 "a typed spacing on the allowlist at its count" theme_check

printf 'HStack(spacing: 2) {}\nText("a").padding(.vertical, 10)\n' >"$gap_file"
expect 1 "an allowed value in the file does not cover a different one" theme_check

printf 'HStack(spacing: 2) {}\nVStack(spacing: 2) {}\n' >"$gap_file"
expect 1 "an allowed value used more often than its count" theme_check

printf '%s\tspacing: 2\t3\t#2706\n' "$gap_file" >"$themed_allow"
expect 1 "an allowed value used less often than its count" theme_check

rm "$gap_file"
expect 1 "an allowlist entry that matches nothing" theme_check

printf '%s\tspacing: 2\t1\n' "$gap_file" >"$themed_allow"
expect 2 "an allowlist entry with no issue" theme_check
printf '%s\tspacing: 2\t1\tpredates the check\n' "$gap_file" >"$themed_allow"
expect 2 "an allowlist entry with a reason but no issue" theme_check
: >"$themed_allow"

expect 2 "a root that is not there" env THEME_VALUES_ROOT="$work/nowhere" bash "$repo_root/scripts/check-theme-values.sh"

expect 0 "the screens this repo actually ships" bash "$repo_root/scripts/check-theme-values.sh"

# ── The design token check ──────────────────────────────────────────────────

tokens="$work/tokens"
mkdir -p "$tokens"
tokens_check() {
  DESIGN_TOKENS_THEME="$tokens/Theme.swift" DESIGN_TOKENS_PAGE="$tokens/page.html" \
    bash "$repo_root/scripts/check-design-tokens.sh"
}
write_theme() {
  printf '%s\n' \
    '  static let paperTop = Color(hex: 0xF7F4EF)' \
    '  static let marker = Color(hex: 0xFFE9A3)' \
    '  static let masteryTrack = Color(hex: 0xEDE8DC)' \
    '  static func marker(_ colour: HighlighterColour) -> Color {' \
    '    case .butter: marker' \
    '    case .coral: Color(hex: 0xFFB4A2)' \
    "$@" >"$tokens/Theme.swift"
}
write_tokens_page() {
  local butter="$1" coral="$2"
  printf '%s\n' \
    '<style>' \
    '  :root {' \
    '    --paperTop: #F7F4EF;' \
    "    --butter: $butter;" \
    '    --track: #EDE8DC;' \
    '  }' \
    '</style>' \
    '<span class="go" style="background:#FFE9A3;">Start</span>' \
    "<span class=\"go\" style=\"background:$coral;\">Start</span>" >"$tokens/page.html"
}

write_theme
write_tokens_page '#FFE9A3' '#FFB4A2'
expect 0 "every colour shown, two through an alias and one only as a swatch" tokens_check

write_theme '  static let iconInk = Color(hex: 0x3B2A1E)'
expect 1 "a token colour the page never shows" tokens_check

write_theme
write_tokens_page '#FFE9A3' '#FFB4A9'
expect 1 "a marker pastel the page never shows" tokens_check

write_tokens_page '#FFE9A4' '#FFB4A2'
expect 1 "an aliased property with a stale hex" tokens_check

expect 2 "a page that is not there" env DESIGN_TOKENS_PAGE="$work/nowhere.html" bash "$repo_root/scripts/check-design-tokens.sh"

expect 0 "the theme and page this repo actually ships" bash "$repo_root/scripts/check-design-tokens.sh"

# ── The component catalogue check ───────────────────────────────────────────

cat_root="$work/catalogue"
components_check() {
  COMPONENTS_ROOT="$cat_root" COMPONENTS_PENDING="${pending_names-Badge}" \
    bash "$repo_root/scripts/check-components.sh"
}
reset_catalogue() {
  rm -rf "$cat_root"
  mkdir -p "$cat_root/design/components/Pills" "$cat_root/ios/Views" "$cat_root/ios/IntradaTests/__Snapshots__/C"
  printf '| Component | What |\n|---|---|\n| [Pills](Pills/README.md) | choose |\n| Badge | kind |\n' \
    >"$cat_root/design/components/README.md"
  printf 'struct Pills<Option: Hashable>: View {}\n' >"$cat_root/ios/Views/Pills.swift"
  : >"$cat_root/ios/IntradaTests/__Snapshots__/C/testPills.1.png"
  write_component_page
}
write_component_page() {
  printf '%s\n' '---' 'name: Pills' "${@:-ios_symbol: Pills}" '---' '' '# Pills' >"$cat_root/design/components/Pills/README.md"
}
ios_keys=('ios_symbol: Pills' 'ios_file: ios/Views/Pills.swift' 'ios_snapshots: C/testPills.1.png')

reset_catalogue
write_component_page "${ios_keys[@]}"
expect 0 "a page with a generic symbol and a row still to come" components_check

pending_names="" expect 1 "a row in the index with no page" components_check

pending_names="Badge Pills" expect 1 "a name left on the pending list once its page exists" components_check

pending_names="Badge Ghost" expect 1 "a pending name that is not in the index" components_check

printf '| Component | What |\n|---|---|\n| [Pills](Pils/README.md) | choose |\n| Badge | kind |\n' \
  >"$cat_root/design/components/README.md"
expect 1 "an index link to the wrong page" components_check
reset_catalogue

write_component_page "${ios_keys[@]}"
mkdir -p "$cat_root/design/components/Stray"
printf '%s\n' '---' 'name: Stray' "${ios_keys[@]}" '---' >"$cat_root/design/components/Stray/README.md"
expect 1 "a page that is not in the index" components_check
rm -rf "$cat_root/design/components/Stray"

write_component_page "${ios_keys[@]}"
sed -i.bak 's/^name: Pills$/name: Pill/' "$cat_root/design/components/Pills/README.md"
expect 1 "a page whose name is not its folder" components_check

write_component_page 'ios_symbol: Pills' 'ios_file: ios/Views/Pills.swift, ios/Views/Gone.swift' 'ios_snapshots: C/testPills.1.png'
expect 1 "an iOS file that is not there" components_check

printf 'struct Pills<Option: Hashable>: View {}\n' >"$cat_root/elsewhere.swift"
write_component_page 'ios_symbol: Pills' 'ios_file: elsewhere.swift' 'ios_snapshots: C/testPills.1.png'
expect 1 "an iOS file outside ios/" components_check

write_component_page 'ios_symbol: ,' 'ios_file: ,' 'ios_snapshots: ,'
expect 1 "iOS keys that name nothing" components_check

printf 'struct Other: View {} // Pills\n' >"$cat_root/ios/Views/Pills.swift"
write_component_page "${ios_keys[@]}"
expect 1 "an iOS symbol only mentioned, never declared" components_check
reset_catalogue

rm "$cat_root/ios/IntradaTests/__Snapshots__/C/testPills.1.png"
write_component_page "${ios_keys[@]}"
expect 1 "an iOS snapshot that is not there" components_check
reset_catalogue

expect 2 "an index that is not there" env COMPONENTS_ROOT="$work/nowhere" bash "$repo_root/scripts/check-components.sh"

expect 0 "the catalogue this repo actually ships" bash "$repo_root/scripts/check-components.sh"

# ── The haptics check ───────────────────────────────────────────────────────

haptics="$work/haptics"
mkdir -p "$haptics/Core" "$haptics/Views"
haptics_check() {
  HAPTICS_ROOT="$haptics" bash "$repo_root/scripts/check-haptics.sh"
}

printf 'UISelectionFeedbackGenerator().selectionChanged()\n' >"$haptics/Core/Store+Feedback.swift"
printf 'Haptic.selection.play()  // not UISelectionFeedbackGenerator\n' >"$haptics/Views/Tap.swift"
expect 0 "the helper itself, a call through it and a comment" haptics_check

printf 'UINotificationFeedbackGenerator().notificationOccurred(.success)\n' >"$haptics/Views/Save.swift"
expect 1 "a success haptic built by hand" haptics_check
rm "$haptics/Views/Save.swift"

printf '.sensoryFeedback(.selection, trigger: revealed)\n' >"$haptics/Views/Reveal.swift"
expect 0 "a selection tick through the SwiftUI modifier" haptics_check

printf '.sensoryFeedback(.success, trigger: saved)\n' >"$haptics/Views/Saved.swift"
expect 1 "a success haptic through the SwiftUI modifier" haptics_check
rm "$haptics/Views/Saved.swift"

printf 'UIImpactFeedbackGenerator(style: .light).impactOccurred()\n' >"$haptics/Store+Feedback.swift"
expect 1 "a second file named like the helper, outside Core" haptics_check
rm "$haptics/Store+Feedback.swift"

expect 2 "a root that is not there" env HAPTICS_ROOT="$work/nowhere" bash "$repo_root/scripts/check-haptics.sh"

expect 0 "the screens this repo actually ships" bash "$repo_root/scripts/check-haptics.sh"

# ── The bridge-test check ───────────────────────────────────────────────────

bridge="$work/bridge"
mkdir -p "$bridge/core" "$bridge/tests"
bridge_check() {
  BRIDGE_CORE_ROOT="$bridge/core" BRIDGE_VIEWMODEL="$bridge/core/model.rs" \
    BRIDGE_TESTS_ROOT="$bridge/tests" BRIDGE_ALLOWLIST="$bridge/allow.txt" \
    bash "$repo_root/scripts/check-bridge-tests.sh"
}
facet='#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]'
cat >"$bridge/core/model.rs" <<EOF
$facet
pub struct ViewModel {
    pub up_next: Option<Alpha>,
    pub error_seq: u64,
}
$facet
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Alpha { One }
$facet
/// Waiting on a test.
pub struct Beta;
EOF
printf 'let a: Alpha = .one\nlet v = ViewModel()\n_ = view.upNext\n' >"$bridge/tests/SessionBridgeTests.swift"
printf '# frozen\nBeta\nViewModel.errorSeq  # never read\n' >"$bridge/allow.txt"
expect 0 "every type and field named or allowlisted" bridge_check

printf '%s\n// Boxing changes the wire.\n#[allow(dead_code)]\npub enum Gamma { A }\n' "$facet" >"$bridge/core/gamma.rs"
expect 1 "a new type behind a comment and an attribute, with no test" bridge_check
printf '// Gamma is covered elsewhere\n' >>"$bridge/tests/SessionBridgeTests.swift"
expect 1 "a new type named only in a comment" bridge_check
printf 'let g: Gamma = .a\n' >"$bridge/tests/StubBridge.swift"
expect 1 "a new type named only outside a LiveBridge test" bridge_check
printf 'let g: Gamma = .a\n' >"$bridge/tests/RowsBridge.swift"
expect 0 "a new type named in RowsBridge" bridge_check
rm "$bridge/core/gamma.rs" "$bridge/tests/StubBridge.swift" "$bridge/tests/RowsBridge.swift"

printf '#[cfg_attr(\n    feature = "facet_typegen",\n    derive(facet::Facet, Clone)\n)]\n#[serde(\n    rename_all = "camelCase"\n)]\npub struct Eta;\n' >"$bridge/core/eta.rs"
expect 1 "a new type behind multi-line attributes, with no test" bridge_check
printf '%s\npub(crate) struct Theta;\n' "$facet" >"$bridge/core/eta.rs"
expect 2 "a derive the parser cannot follow to a type" bridge_check
printf '#[effect(facet_typegen)]\npub enum Effect { Render }\n' >"$bridge/core/eta.rs"
expect 1 "the effect enum, with no test" bridge_check
rm "$bridge/core/eta.rs"

cp "$bridge/core/model.rs" "$bridge/model.orig"
perl -pi -e 's/^(    pub error_seq: u64,\n)/$1    pub notice_seq: u64,\n/' "$bridge/core/model.rs"
expect 1 "a new ViewModel field with no test" bridge_check
printf 'let n = noticeSeq\n_ = bridge.update(.noticeSeq(1))\n' >"$bridge/tests/LibraryBridgeTests.swift"
expect 1 "a new field named only as a variable or an event case" bridge_check
printf '_ = view.noticeSeq\n' >"$bridge/tests/LibraryBridgeTests.swift"
expect 0 "a new field read off the view" bridge_check
rm "$bridge/tests/LibraryBridgeTests.swift"
mv "$bridge/model.orig" "$bridge/core/model.rs"

printf 'let b = Beta()\n' >"$bridge/tests/LibraryBridgeTests.swift"
expect 1 "an allowlisted type that a test now names" bridge_check
rm "$bridge/tests/LibraryBridgeTests.swift"

cp "$bridge/allow.txt" "$bridge/allow.orig"
printf 'Delta\n' >>"$bridge/allow.txt"
expect 1 "an allowlisted type no longer on the bridge" bridge_check
mv "$bridge/allow.orig" "$bridge/allow.txt"

expect 0 "the fixture back where it started" bridge_check
expect 2 "a tests root that is not there" env BRIDGE_TESTS_ROOT="$work/nowhere" bash "$repo_root/scripts/check-bridge-tests.sh"
mv "$bridge/tests/SessionBridgeTests.swift" "$bridge/tests/SessionBridge.swift"
expect 2 "a tests root with no LiveBridge test" bridge_check
mv "$bridge/tests/SessionBridge.swift" "$bridge/tests/SessionBridgeTests.swift"

expect 0 "the bridge this repo actually ships" bash "$repo_root/scripts/check-bridge-tests.sh"

# ── The comment density check ───────────────────────────────────────────────

density="$work/density"
mkdir -p "$density"
cd "$density"
git init -q -b main .
git config user.email "test@example.com"
git config user.name "Hygiene self-test"
git config commit.gpgsign false
printf 'base\n' >README.md
git add -A
git commit -qm "base"
git checkout -qb feature

commented_source() {
  for i in $(seq 1 30); do printf 'val v%s = %s\n' "$i" "$i"; done
  for i in $(seq 1 10); do printf '// narrating line %s\n' "$i"; done
}
density_check() {
  COMMENT_CHECK_BASE=main bash "$repo_root/scripts/check-comment-density.sh"
}
commit_only() {
  git rm -rqf --ignore-unmatch src >/dev/null
  mkdir -p src
  commented_source >"src/$1"
  git add -A
  git commit -qm "$1"
}

commit_only Screen.txt
expect 0 "a heavily commented file of a kind the check does not read" density_check
commit_only Screen.swift
expect 1 "a heavily commented Swift file" density_check
expect 0 "the same file with the check bypassed" env SKIP_COMMENT_CHECK=1 COMMENT_CHECK_BASE=main \
  bash "$repo_root/scripts/check-comment-density.sh"

cd "$repo_root"

# ── The snapshot check ──────────────────────────────────────────────────────

snaps="$work/snaps"
mkdir -p "$snaps/ios/__Snapshots__/LibrarySnapshotTests"
snapshot_check() {
  SNAPSHOT_IOS_ROOT="$snaps/ios" bash "$repo_root/scripts/check-snapshots.sh"
}
printf 'func testLibrary() {}\n' >"$snaps/ios/LibrarySnapshotTests.swift"
printf 'png' >"$snaps/ios/__Snapshots__/LibrarySnapshotTests/testLibrary.1.png"
expect 0 "every reference named by a test and small" snapshot_check

head -c 200001 /dev/zero >"$snaps/ios/__Snapshots__/LibrarySnapshotTests/testLibrary.1.png"
expect 1 "an iOS reference over the ceiling" snapshot_check
head -c 200000 /dev/zero >"$snaps/ios/__Snapshots__/LibrarySnapshotTests/testLibrary.1.png"
expect 0 "an iOS reference exactly at the ceiling" snapshot_check
printf 'png' >"$snaps/ios/__Snapshots__/LibrarySnapshotTests/testLibrary.1.png"

printf 'png' >"$snaps/ios/__Snapshots__/LibrarySnapshotTests/testRetired.1.png"
expect 1 "an iOS reference no test names" snapshot_check
rm "$snaps/ios/__Snapshots__/LibrarySnapshotTests/testRetired.1.png"

expect 0 "the references this repo actually ships" bash "$repo_root/scripts/check-snapshots.sh"

# ── The dash check ──────────────────────────────────────────────────────────

em=$'\xe2\x80\x94'
en=$'\xe2\x80\x93'
sandbox="$work/dashes"
mkdir -p "$sandbox"
cd "$sandbox"
git init -q -b main .
git config user.email "test@example.com"
git config user.name "Hygiene self-test"
git config commit.gpgsign false
printf 'let a = 1\n' >base.swift
git add -A
git commit -qm "base"
git update-ref refs/remotes/origin/main main

dash_case() {
  local want="$1" name="$2" file="$3" line="$4"
  git checkout -qB "case-$passed-$failures" main
  printf '%s\n' "$line" >"$file"
  git add -A
  git commit -qm "case"
  expect "$want" "$name" bash "$repo_root/scripts/check-dashes.sh"
}

dash_case 1 "a dash in a Swift comment" a.swift "/// A piece $em for the snapshot."
dash_case 1 "an en dash in Rust prose" a.rs "// ii${en}V${en}i reads badly here"
dash_case 1 "a dash in Markdown" a.md "Plain prose $em with a dash."
dash_case 1 "a dash outside the string on a code line" a.swift "let x = \"ok\" $em 1"
dash_case 1 "a quoted dash inside a comment" a.swift "// show \"$em\" when empty"
dash_case 0 "a dash inside a Swift string literal" a.swift "durationDisplay: \"$em\","
dash_case 0 "an en dash inside a Rust string literal" a.rs "title: \"ii${en}V${en}i\".to_string(),"
dash_case 0 "an escaped quote before the dash" a.swift "let s = \"say \\\"$em\\\" now\""
dash_case 0 "no dash at all" a.swift "let plain = \"text\""

fence=$'```'
dash_case 1 "a double dash in Markdown prose" a.md "Plain prose -- with a dash."
dash_case 1 "an American spelling in Markdown" a.md "Pick a color for the tag."
dash_case 1 "an American spelling in capitals" a.md "Behavior of the timer."
dash_case 1 "prose after a fence closes" a.md "$fence"$'\nx --y\n'"$fence"$'\nThe color here.'
dash_case 0 "a double dash in a code span" a.md 'Run `cargo test -- --nocapture` first.'
dash_case 0 "an American spelling in a code span" a.md 'The `Color` token.'
dash_case 0 "a code span wrapped across lines" a.md $'Run `xcrun simctl\n--process SpringBoard` to see it.'
dash_case 1 "prose after a wrapped code span closes" a.md $'Run `xcrun simctl\n--process SpringBoard` in color.'
dash_case 1 "a blank line ends an unclosed code span" a.md $'A stray ` tick.\n\nThen -- a dash.'
dash_case 0 "a double dash in a fenced block" a.md "$fence"$'bash\ncargo test -- --nocapture\n'"$fence"
dash_case 0 "a table separator" a.md $'| a | b |\n|--|--|\n| 1 | 2 |'
dash_case 0 "an HTML comment" a.md "<!-- a note -->"
dash_case 0 "a horizontal rule" a.md "---"
dash_case 0 "an American spelling in a link target" a.md "See [the palette](docs/color.md)."
dash_case 0 "British spellings" a.md "The colour and behaviour of the centre."
dash_case 0 "a double dash in a Swift comment" a.swift "// run it with --verbose"
dash_case 0 "a blank line inside a fenced block" a.md "$fence"$'\n\nx -- y\n'"$fence"
dash_case 1 "prose after a closed span on the same line" a.md 'The `Color` token, in color.'
dash_case 1 "a stray tick before a fence" a.md $'A stray ` tick.\n'"$fence"$'\nx\n'"$fence"$'\nThen -- a dash.'
dash_case 0 "an Apple product name" a.md "Open Control Center and play."
dash_case 0 "a quoted citation" a.md "Deliberate Practice and Behavior. <!-- docs-check: quoted -->"
dash_case 1 "a non-ASCII file name" $'\xc3\xa9\xc3\xa9.md' "The color."

git checkout -qB "old-prose" main
printf 'The old color.\n' >old.md
git add -A
git commit -qm "old prose"
git update-ref refs/remotes/origin/main old-prose
git checkout -qB "new-prose" old-prose
printf 'The old color.\nThe new colour.\n' >old.md
git add -A
git commit -qm "new prose"
expect 0 "an American spelling on a line the branch did not add" bash "$repo_root/scripts/check-dashes.sh"
git checkout -qB "moved-prose" old-prose
git mv old.md moved.md
git commit -qm "move prose"
expect 0 "an American spelling in a file the branch only moved" bash "$repo_root/scripts/check-dashes.sh"
git update-ref refs/remotes/origin/main main

# ── The app version check ───────────────────────────────────────────────────

versions="$work/versions"
mkdir -p "$versions"
cd "$versions"
git init -q -b main .
git config user.email "test@example.com"
git config user.name "Hygiene self-test"
git config commit.gpgsign false
git config tag.gpgsign false

set_version() {
  printf 'settings:\n  base:\n    MARKETING_VERSION: "%s"\n' "$1" >project.yml
}

version_check() {
  APP_VERSION_PROJECT=project.yml bash "$repo_root/scripts/check-app-version.sh"
}

set_version 0.9.0
git add -A
git commit -qm "base"
expect 1 "no release tag reachable" version_check

git tag v0.9.0
expect 0 "the version equal to the newest tag" version_check

git tag v0.10.0
expect 1 "the version behind the newest tag" version_check

set_version 0.10.0
expect 0 "a two-digit minor compared as a number" version_check

git tag v0.11.0
expect 1 "behind a newer two-digit tag" version_check

set_version 0.11.0
expect 0 "the version bumped ahead of the tag being cut" version_check

git checkout -qb release-elsewhere
git commit -q --allow-empty -m "elsewhere"
git tag v0.12.0
git checkout -q main
expect 0 "a newer tag not reachable from HEAD" version_check

set_version "0.11"
expect 1 "a version that is not X.Y.Z" version_check

cd "$repo_root"
expect 0 "the version this repo actually ships" bash scripts/check-app-version.sh

# ── The self-test scope (#2734) ─────────────────────────────────────────────

sandbox="$work/scope"
mkdir -p "$sandbox/scripts/tests" "$sandbox/crates" "$sandbox/.github/workflows"
cd "$sandbox"
git init -q -b main .
git config user.email "test@example.com"
git config user.name "Hygiene self-test"
git config commit.gpgsign false
git config core.quotePath true
printf 'echo ok\n' >scripts/tests/demo-test.sh
printf 'on: push\n' >.github/workflows/ci.yml
printf 'fn main() {}\n' >crates/lib.rs
printf 'hygiene:\n' >justfile
git add -A
git commit -qm "base"
git update-ref refs/remotes/origin/main main
git checkout -qb branch

scope_entries=("typos:typos" "demo-test:bash scripts/tests/demo-test.sh")

# Exit 0 when the self-test entry is kept, with no "skipped:" line.
self_tests_run() {
  local out
  out=$(bash -c 'source "$1"; shift; diff_scope_select_checks "$@"' _ \
    "$repo_root/scripts/lib/diff-scope.sh" "${scope_entries[@]}" 2>&1)
  grep -qx 'demo-test:bash scripts/tests/demo-test.sh' <<<"$out" &&
    grep -qx 'typos:typos' <<<"$out" && ! grep -q '^skipped:' <<<"$out"
}

# Exit 0 when only the self-test is dropped, named on one "skipped:" line.
self_tests_skipped() {
  local out
  out=$(bash -c 'source "$1"; shift; diff_scope_select_checks "$@"' _ \
    "$repo_root/scripts/lib/diff-scope.sh" "${scope_entries[@]}" 2>&1)
  ! grep -q 'scripts/tests/' <<<"$out" && grep -qx 'typos:typos' <<<"$out" &&
    [ "$(grep -c '^skipped:.*demo-test' <<<"$out")" = 1 ]
}

printf '// edit\n' >>crates/lib.rs
expect 0 "a Rust-only edit skips the self-tests" self_tests_skipped

printf '# edit\n' >>scripts/tests/demo-test.sh
expect 0 "a script edit runs the self-tests" self_tests_run
git checkout -q -- scripts

printf 'check:\n' >>justfile
expect 0 "a justfile edit runs the self-tests" self_tests_run
git checkout -q -- justfile

printf 'echo new\n' >scripts/new-tool.sh
expect 0 "a new untracked script runs the self-tests" self_tests_run
rm scripts/new-tool.sh

printf 'echo new\n' >"scripts/tëst.sh"
expect 0 "a new untracked script with a non-ASCII name runs the self-tests" self_tests_run
git add "scripts/tëst.sh"
expect 0 "a staged script with a non-ASCII name runs the self-tests" self_tests_run
git rm -q --cached "scripts/tëst.sh"
rm "scripts/tëst.sh"

printf 'on: pull_request\n' >>.github/workflows/ci.yml
expect 0 "a workflow edit runs the self-tests" self_tests_run
git checkout -q -- .github

git mv scripts/tests/demo-test.sh crates/demo-test.sh
expect 0 "a script moved out of scripts/ runs the self-tests" self_tests_run
git mv crates/demo-test.sh scripts/tests/demo-test.sh

printf '// edit again\n' >>crates/lib.rs
expect 0 "a selection of only self-tests keeps them all, with no skipped line" \
  bash -c 'out=$(source "$1"; diff_scope_select_checks "demo-test:bash scripts/tests/demo-test.sh" 2>&1)
    [ "$out" = "demo-test:bash scripts/tests/demo-test.sh" ]' _ "$repo_root/scripts/lib/diff-scope.sh"
git checkout -q -- crates
printf '// edit\n' >>crates/lib.rs

git commit -qam "rust only"
expect 0 "a committed Rust-only change still skips" self_tests_skipped

git checkout -q --detach
expect 0 "a detached head runs the self-tests" self_tests_run
git checkout -q branch

git checkout -q main
printf '// on main\n' >>crates/lib.rs
expect 0 "on main the self-tests run" self_tests_run
git checkout -q -- crates
git checkout -q branch

git update-ref -d refs/remotes/origin/main
expect 0 "no origin/main runs the self-tests" self_tests_run

cd "$repo_root"
expect 0 "the hygiene recipe text names the scope helper on its checks (a text check)" \
  grep -q 'diff_scope_select_checks "${checks\[@\]}"' justfile

# ── The lessons cap ────────────────────────────────────────────────────────

lessons="$work/lessons.md"
lessons_archive="$work/lessons-archive.md"
printf '# Lessons archive\n\n## Archived\n\n- 2026-11-01, #1: an archived lesson.\n' >"$lessons_archive"
lessons_check() {
  LESSONS_FILE="$lessons" LESSONS_ARCHIVE="$lessons_archive" bash "$repo_root/scripts/check-lessons.sh"
}
write_lessons() {
  { printf '# Lessons\n\n- a rule above the heading\n- another rule\n\n## Lessons\n\n'
    for i in $(seq 1 "$1"); do printf -- '- 2026-11-12, #1: lesson %s\n' "$i"; done; } >"$lessons"
}
write_lessons 30
expect 0 "30 lessons, with rules above the heading not counted" lessons_check
write_lessons 31
expect 1 "31 lessons" lessons_check
lessons_over_names_archive() { grep -q "lessons-archive.md" <<<"$(lessons_check 2>&1 || true)"; }
expect 0 "31 lessons, the message naming the archive" lessons_over_names_archive
lessons_over_names_both_steps() {
  local out
  out=$(lessons_check 2>&1 || true)
  grep -q "whose issue has closed" <<<"$out" && grep -q "and no open issue" <<<"$out"
}
expect 0 "31 lessons, the message naming closed issues first and keeping open ones" lessons_over_names_both_steps
write_lessons 30
printf -- '- 2026-11-12, #1: lesson 30\n' >>"$lessons"
lessons_over_and_repeated() {
  local out
  out=$(lessons_check 2>&1 || true)
  grep -q "over the cap" <<<"$out" && grep -q "line 38 repeats line 37" <<<"$out"
}
expect 1 "31 lessons with one repeated" lessons_check
expect 0 "31 lessons with one repeated, both messages printed" lessons_over_and_repeated
{ printf '## Lessons\n\n'; for i in $(seq 1 25); do printf -- '- a: %s\n' "$i"; done
  printf '\n## Older\n\n'; for i in $(seq 1 25); do printf -- '- b: %s\n' "$i"; done; } >"$lessons"
expect 1 "50 lessons under two headings, all of which the hook prints" lessons_check
printf '# Lessons\n\n## Lessons \n\n- trailing space on the heading\n' >"$lessons"
expect 1 "no exact Lessons heading" lessons_check
rm "$lessons"
lessons_missing() { grep -q "no $lessons to read" <<<"$(lessons_check 2>&1 || true)"; }
expect 0 "no lessons file, named in the message" lessons_missing

# A union merge keeps both sides' lines (#2937), so a lesson can arrive twice.
write_lessons_with() {
  { printf '# Lessons\n\n- %s\n\n## Lessons\n\n' "$1"; shift
    printf -- '- 2026-11-12, #1: a distinct lesson.\n'
    for line in "$@"; do printf -- '- %s\n' "$line"; done; } >"$lessons"
}
dup='2026-11-12, #2: use `just claim` from the worktree, not $HOME [sic] (a.b)'
write_lessons_with "$dup." "$dup." "2026-11-12, #3: another lesson."
expect 0 "a rule above the heading repeating a lesson, and regex characters" lessons_check
write_lessons_with "a rule." "$dup." "$dup."
expect 1 "the same lesson twice" lessons_check
write_lessons_with "a rule." "$dup." "$dup (seen again 1)."
expect 1 "a lesson beside its bumped copy" lessons_check
write_lessons_with "a rule." "$dup (seen again 1)." "$dup (seen again 2)."
expect 1 "two branches bumping the same lesson" lessons_check
lessons_dup_message() {
  grep -q "lessons.md line 9 repeats line 8 .*keep one line" <<<"$(lessons_check 2>&1 || true)"
}
expect 0 "a repeated lesson, its lines and the fix in the message" lessons_dup_message
write_lessons_with "a rule." "2026-11-01, #1: an archived lesson."
expect 1 "a lesson that is also archived" lessons_check
lessons_archived_message() {
  grep -q "lessons.md line 8 is archived at .*lessons-archive.md line 5 with no higher .*delete it here" <<<"$(lessons_check 2>&1 || true)"
}
expect 0 "an archived lesson, both lines and the fix in the message" lessons_archived_message
write_lessons_with "a rule." "2026-11-01, #1: an archived lesson (seen again 1)."
expect 0 "an archived lesson copied back with its count raised" lessons_check
printf -- '- 2026-11-03, #5: a lesson seen before (seen again 2).\n' >>"$lessons_archive"
write_lessons_with "a rule." "2026-11-03, #5: a lesson seen before (seen again 2)."
expect 1 "an archived lesson brought back with the archive's count" lessons_check
write_lessons_with "a rule." "2026-11-03, #5: a lesson seen before (seen again 1)."
expect 1 "an archived lesson brought back with a lower count" lessons_check
write_lessons_with "a rule." "2026-11-03, #5: a lesson seen before (seen again 3)."
expect 0 "an archived lesson copied back above the archive's count" lessons_check
printf -- '- rule: .claude/rules/ios-ui.md 2026-11-02, #4: a lesson that became a rule.\n' >>"$lessons_archive"
write_lessons_with "a rule." "2026-11-02, #4: a lesson that became a rule."
expect 1 "a lesson archived as a rule" lessons_check
write_lessons_with "a rule." "2026-11-02, #4: a rule." "2026-11-01, #1: an archived lesson, said again."
expect 0 "lessons that only share an archived line's start or end" lessons_check

# A union merge of two edits to one rule line keeps both (#2943).
printf '# Lessons\n\n- a rule that\n  one is copied back here.\n- another rule\n    one is copied back here. \n\n## Lessons\n\n- 2026-11-12, #1: a lesson.\n' >"$lessons"
expect 1 "a rules line above the heading kept twice, indented differently" lessons_check
lessons_rule_dup_message() {
  grep -qF 'lessons.md line 6 repeats line 4 in the rules above "## Lessons": "one is copied back here."' <<<"$(lessons_check 2>&1 || true)"
}
expect 0 "a doubled rules line, both lines and the text in the message" lessons_rule_dup_message
cat >"$lessons" <<'EOF'
# Lessons

> A quoted header
>
> over two paragraphs.
>

```text
just check
```

```text
just pre-review
```

~~~
just usage
~~~

---

| a | b |
| --- | --- |
| c | d |
| --- | --- |

## Lessons

- 2026-11-12, #1: a lesson.
EOF
expect 0 "rules with code fences, bare quote markers and separators repeated" lessons_check

# ── Fixes that did not hold (#2968) ────────────────────────────────────────
fixes_say() {
  local out
  out=$(LESSONS_FILE="$lessons" LESSONS_ARCHIVE="$lessons_archive" bash "$repo_root/scripts/lesson-fixes.sh" 2>&1) || return 1
  grep -qF -- "$1" <<<"$out"
}
printf -- '- fix: docs/a.md, 2026-11-06, #6: a fix that held.\n' >>"$lessons_archive"
printf -- '- fix: docs/b.md, 2026-11-07, #7: a fix seen before it was made (seen again 2).\n' >>"$lessons_archive"
printf -- '- fix: docs/c.md, 2026-11-08, #8: a fix archived again after it failed.\n' >>"$lessons_archive"
printf -- '- issue: 2026-11-08, #8: a fix archived again after it failed (seen again 1).\n' >>"$lessons_archive"
write_lessons_with "a rule." "2026-11-06, #6: a fix that held, said differently (seen again 1)."
expect 0 "a fix archived again with a higher count counts; one that held and one counted before its fix do not" fixes_say "Fixes that did not hold: 1 of 3 fixes"
printf -- '- fix: docs/d.md, 2026-11-09, #9: a fix that did not hold.\n' >>"$lessons_archive"
write_lessons_with "a rule." "2026-11-09, #9: a fix that did not hold (seen again 1)."
expect 0 "a fixed lesson back in the lessons file counts" fixes_say "Fixes that did not hold: 2 of 4 fixes"
expect 0 "a fix that came back is named" fixes_say "  2026-11-09, #9: a fix that did not hold"
write_lessons_with "a rule." "2026-11-07, #7: a fix seen before it was made (seen again 2)."
expect 0 "a copy at the fixed line's own count does not count" fixes_say "Fixes that did not hold: 1 of 4 fixes"
write_lessons_with "a rule." "2026-11-07, #7: a fix seen before it was made (issue #70) (seen again 3)."
expect 0 "a fix back with an issue beside its count counts" fixes_say "Fixes that did not hold: 2 of 4 fixes"

mv "$lessons_archive" "$work/archive-away.md"
lessons_no_archive() { grep -q "no $lessons_archive to read" <<<"$(lessons_check 2>&1 || true)"; }
expect 0 "no archive, named in the message" lessons_no_archive
mv "$work/archive-away.md" "$lessons_archive"

# ── The grep pipe check ────────────────────────────────────────────────────
# Each fixture's pipe arrives through %s, so this file's own text never holds
# one for the check to refuse.

pipes_root="$work/grep-pipes"
pipes_check() {
  GREP_PIPES_ROOT="$pipes_root" bash "$repo_root/scripts/check-grep-pipes.sh"
}
pipes_case() {
  local want="$1" name="$2" body="$3"
  mkdir -p "$pipes_root/lib"
  : >"$pipes_root/clean.sh"
  : >"$pipes_root/lib/helper.sh"
  # shellcheck disable=SC2059 # the body is the format, its %s the pipe
  printf "$body" '|' '|' >"$pipes_root/${4:-clean.sh}"
  expect "$want" "$name" pipes_check
}
pipes_case 1 "a pipe into grep -q on one line" 'set -o pipefail\nprintf x %s grep -q x\n'
pipes_case 1 "a pipe split after the bar" 'printf x %s\n  grep -q x\n'
pipes_case 1 "a pipe split by a backslash" 'printf x %s \\\n  grep -q x\n'
pipes_case 1 "a pipe into grep -E -q" 'printf x %s grep -E -q x\n'
pipes_case 1 "a pipe into grep --quiet" 'printf x %s grep --quiet x\n'
pipes_case 1 "a pipe into grep --silent" 'printf x %s grep --silent x\n'
pipes_case 1 "a pipe and standard error into grep -q" 'printf x %s& grep -q x\n'
pipes_case 1 "a pipe into egrep -q" 'printf x %s egrep -q x\n'
pipes_case 1 "a pipe into command grep -q" 'printf x %s command grep -q x\n'
pipes_case 1 "a pipe into grep -q by its path" 'printf x %s /usr/bin/grep -q x\n'
pipes_case 1 "a pipe in a sourced helper under lib/" 'printf x %s grep -qF x\n' lib/helper.sh
pipes_case 0 "a here-string" 'grep -q x <<<"$text"\n'
pipes_case 0 "an or before grep -q" 'false %s%s grep -q x file\n'
pipes_case 0 "a commented pipe" '# printf x %s grep -q x\n'
pipes_case 0 "a pipe into a grep that prints" 'printf x %s grep -c x\n'
pipes_names_shape() { grep -q "here-string" <<<"$(pipes_check 2>&1 || true)"; }
printf 'printf x %s grep -q x\n' '|' >"$pipes_root/clean.sh"
expect 0 "a refusal names the here-string shape" pipes_names_shape
pipes_root="$work/no-such-root"
expect 2 "no root to read" pipes_check

# ── The self-tests' shared text match ──────────────────────────────────────

contains_run() { bash -c 'source "$1"; contains "$2" "$3"' _ "$repo_root/scripts/lib/contains.sh" "$@"; }
expect 0 "contains finds a needle in its haystack" contains_run "b" $'a\nb\nc'
expect 1 "contains misses a needle the haystack lacks" contains_run "d" $'a\nb\nc'
expect 2 "contains stops on an empty needle" contains_run "" "text"
expect 2 "contains stops on an empty needle in an empty haystack" contains_run "" ""
contains_names_needle() { grep -q "empty needle" <<<"$(contains_run "" "" 2>&1 || true)"; }
expect 0 "an empty needle is named in the message" contains_names_needle
expect 1 "no self-test helper matches its needle with a bare grep (a text check)" \
  grep -rqE 'grep -qF (-- )?"\$(needle|want_text)" <<<' "$repo_root/scripts/tests"

# ── The review route (#2922) ────────────────────────────────────────────────

route="$work/route"
mkdir -p "$route/docs" "$route/.claude/rules" "$route/crates/core/src" "$route/crates/intrada-store/src"
cd "$route"
git init -q -b main .
git config user.email "test@example.com"
git config user.name "Hygiene self-test"
git config commit.gpgsign false
printf '# Notes\n' >docs/notes.md
printf '# Rule\n' >.claude/rules/rule.md
printf '# Guide\n' >CLAUDE.md
printf '%s\n' '#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]' \
  'pub struct Shown {' '    pub a: u8,' '}' '' 'impl Shown {' '    fn twice(&self) -> u8 { self.a }' '}' \
  '' 'pub struct Kept {' '    pub b: u8,' '}' >crates/core/src/view.rs
printf 'const M: &[&str] = &["one"];\n' >crates/intrada-store/src/migrations.rs
git add -A
git commit -qm "base"
git update-ref refs/remotes/origin/main main
git checkout -qb branch

route_says() {
  local out
  out=$(bash -c 'source "$1"; diff_scope_review_route' _ "$repo_root/scripts/lib/diff-scope.sh" 2>&1)
  [[ $out == *"$1"* ]]
}

printf 'more\n' >>docs/notes.md
printf '# New\n' >docs/new.md
expect 0 "a documentation-only diff skips review" route_says "none: documentation only"
printf 'more\n' >>.claude/rules/rule.md
expect 0 "a rule under .claude/ takes /code-review inline" route_says "/code-review inline"
git checkout -q -- .claude
printf '{}\n' >.claude/rules/settings.json
expect 0 "a settings file under .claude/ takes reviewer" route_says "reviewer"
rm .claude/rules/settings.json
printf '# Rows\n' >docs/working-with-agents.md
expect 0 "the routing table is agent instructions" route_says "/code-review inline"
rm docs/working-with-agents.md
printf 'more\n' >>CLAUDE.md
expect 1 "CLAUDE.md is not documentation" route_says "none"
git checkout -q -- CLAUDE.md
sed -i.bak 's/self.a }/self.a * 2 }/; s/pub b: u8/pub b: u16/' crates/core/src/view.rs
expect 0 "a body or a plain type is not a bridge change" route_says "reviewer"
expect 1 "a body or a plain type names nothing sensitive" route_says "sensitive"
sed -i.bak 's/pub a: u8/pub a: u16/' crates/core/src/view.rs
expect 0 "a field of a Facet type is a bridge change" route_says "sensitive: Shown in crates/core/src/view.rs"
git checkout -q -- crates
sed -i.bak '1d' crates/core/src/view.rs
expect 0 "dropping the Facet derive is a bridge change" route_says "Shown in"
git checkout -q -- crates
printf 'const B: u32 = 1; // BLOB_VERSION\n' >>crates/core/src/view.rs
printf '// two\n' >>crates/intrada-store/src/migrations.rs
expect 0 "a blob pin is a bridge change" route_says "a blob pin in crates/core/src/view.rs"
expect 0 "a store change is a bridge change" route_says "the store in crates/intrada-store/src/migrations.rs"
git checkout -q -- crates
git clean -fdq
mkdir -p crates/intrada-ffi/src
printf 'fn f() {}\n' >crates/intrada-ffi/src/ffi.rs
printf '%s\n' '#[cfg_attr(' '    feature = "facet_typegen",' '    derive(facet::Facet)' ')]' \
  'pub struct Wrapped {' '    pub c: u8,' '} // trailing note' 'fn after() {}' >crates/core/src/wrapped.rs
cp crates/core/src/wrapped.rs crates/core/src/tests.rs
expect 0 "an FFI source file is a bridge change" route_says "the FFI bridge in crates/intrada-ffi/src/ffi.rs"
expect 0 "a Facet derive on a wrapped attribute is a bridge change" route_says "Wrapped in crates/core/src/wrapped.rs"
expect 1 "a test file is not a bridge change" route_says "crates/core/src/tests.rs"
git add -A
git commit -qm "wrapped"
rm -r crates/intrada-ffi crates/core/src/tests.rs
git commit -qam "drop"
git update-ref refs/remotes/origin/main HEAD
sed -i.bak 's/fn after() {}/fn after() { }/' crates/core/src/wrapped.rs
expect 1 "code after an item closed with a comment is not inside it" route_says "sensitive"
rm crates/core/src/wrapped.rs.bak
git checkout -q -- crates
git clean -fdq
mkdir -p tools/.claude
printf '# Agent\n' >tools/.claude/agent.md
expect 1 "a .claude/ folder anywhere is reviewed" route_says "none"
rm -r tools
printf '# Loose\n' >notes.md
expect 1 "markdown outside the documentation roots is reviewed" route_says "none"
rm notes.md
cp crates/core/src/view.rs "crates/core/src/new view.rs"
expect 0 "a new untracked file with a space is scanned as all added" route_says "Shown in crates/core/src/new view.rs"
rm "crates/core/src/new view.rs"
cp crates/core/src/view.rs 'crates/core/src/$(touch pwned).rs'
git add -A
git commit -qm "odd name"
sed -i.bak 's/pub a: u8/pub a: u16/' 'crates/core/src/$(touch pwned).rs'
expect 0 "a file name holding shell syntax is read as a name" route_says 'Shown in crates/core/src/$(touch pwned).rs'
expect 1 "a file name holding shell syntax runs nothing" test -e pwned
cd "$repo_root"

# ── Result ──────────────────────────────────────────────────────────────────

if [ "$failures" -gt 0 ]; then
  printf '\n✗ hygiene self-test: %s of %s cases failed\n' "$failures" "$((passed + failures))" >&2
  exit 1
fi

printf '✓ hygiene self-test: %s cases, every gate seen to fail and to pass\n' "$passed"
