#!/usr/bin/env bash
# A build dispatched by hand takes its version from ios/project.yml, not from a
# tag, so a version left behind the newest release uploads under a release
# nobody cut and files its crashes there (#1961). v0.11.0 was tagged while the
# file still said 0.10.0.
#
# The rule is "not behind", not "equal": the version is bumped on main in its
# own PR before the tag is cut on that commit, so main is legitimately ahead of
# the newest tag between the two. Only tags reachable from HEAD count, so a tag
# cut on another branch never fails this one.
#
# The project path is overridable so the self-test can point it at fixtures
# (scripts/tests/hygiene-checks-test.sh).

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

project_file="${APP_VERSION_PROJECT:-ios/project.yml}"

fail() {
  printf '\nBlocked: %s\n\n' "$1" >&2
  exit 1
}

[ -f "$project_file" ] || fail "$project_file is missing, so the app version cannot be read."

version_re='^[0-9]+\.[0-9]+\.[0-9]+$'

version=$({ grep -E '^[[:space:]]*MARKETING_VERSION:' "$project_file" || true; } |
  head -1 | sed -E 's/^[[:space:]]*MARKETING_VERSION:[[:space:]]*//; s/["[:space:]]//g')
[[ $version =~ $version_re ]] || fail "$project_file sets MARKETING_VERSION to '$version', not X.Y.Z."

tag=$(git tag --merged HEAD --list 'v*' --sort=-v:refname |
  { grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' || true; } | head -1)
[ -n "$tag" ] || fail "no vX.Y.Z tag is reachable from HEAD. A shallow clone hides them: fetch the tags (git fetch --tags) or check out with full history."

newest="${tag#v}"
lowest=$(printf '%s\n%s\n' "$version" "$newest" | sort -t. -k1,1n -k2,2n -k3,3n | head -1)

if [ "$version" != "$newest" ] && [ "$lowest" = "$version" ]; then
  fail "$project_file sets MARKETING_VERSION to $version, behind the newest release $tag. Bump it to at least $newest before the next build."
fi

echo "✓ app version: $version is not behind the newest release $tag"
exit 0
