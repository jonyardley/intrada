#!/usr/bin/env bash
# What this branch changes, so local gates can skip work a change cannot
# affect (#2734). CI does not source this: it runs its own list directly.

# Paths changed since the merge base with origin/main, worktree and untracked
# included; fails on no origin/main, a detached head or main ("run everything").
diff_scope_paths() {
  local branch base tracked untracked
  branch=$(git symbolic-ref --quiet --short HEAD 2>/dev/null) || return 1
  [ "$branch" != "main" ] || return 1
  base=$(git merge-base origin/main HEAD 2>/dev/null) || return 1
  tracked=$(git diff -z --name-only --no-renames "$base" -- | tr '\0' '\n'; exit "${PIPESTATUS[0]}") || return 1
  untracked=$(git ls-files -z --others --exclude-standard | tr '\0' '\n'; exit "${PIPESTATUS[0]}") || return 1
  printf '%s\n%s\n' "$tracked" "$untracked"
}

# Hygiene "name:command" entries to run: scripts/tests/ self-tests drop out, named
# on one stderr "skipped:" line, unless scripts/, the justfile or .github/ changed.
diff_scope_select_checks() {
  local paths entry kept="" skipped=""
  if ! paths=$(diff_scope_paths) || grep -qE '^(scripts/|justfile$|\.github/)' <<<"$paths"; then
    printf '%s\n' "$@"
    return 0
  fi
  for entry in "$@"; do
    case "${entry#*:}" in
      *scripts/tests/*) skipped="${skipped:+$skipped, }${entry%%:*}" ;;
      *) kept="$kept$entry"$'\n' ;;
    esac
  done
  if [ -z "$kept" ]; then
    printf '%s\n' "$@"
    return 0
  fi
  printf '%s' "$kept"
  if [ -n "$skipped" ]; then
    echo "skipped: no change under scripts/, the justfile or .github/, so not running $skipped" >&2
  fi
}

# CI's Android settings, read from ci.yml so the local gates cannot drift from
# them (#2877). Succeeds when the `changes` job says Android is paused.
ci_android_paused() {
  grep -qE '^[[:space:]]*android_paused:[[:space:]]*"true"' .github/workflows/ci.yml
}

# Succeeds when a path on stdin matches CI's `android` path filter, or when
# that list cannot be read ("run everything").
ci_android_changed() {
  local patterns pattern changed
  patterns=$(awk '
    /^[[:space:]]*filters:[[:space:]]*\|/ { infilters = 1; next }
    infilters && /^[[:space:]]*android:[[:space:]]*$/ { inandroid = 1; next }
    inandroid && /^[[:space:]]*- / { sub(/^[[:space:]]*- /, ""); gsub(/\x27|"/, ""); print; next }
    inandroid { exit }' .github/workflows/ci.yml)
  [ -n "$patterns" ] || return 0
  while IFS= read -r changed; do
    [ -n "$changed" ] || continue
    while IFS= read -r pattern; do
      # shellcheck disable=SC2053 # the pattern is a glob on purpose
      [[ $changed == ${pattern//\*\*/*} ]] && return 0
    done <<<"$patterns"
  done
  return 1
}

# How the branch's diff is reviewed (#2922): "none" when every path is
# documentation, else "reviewer", plus each bridge change the reviewer pairs on.
# Fails closed: an unreadable or empty diff, or markdown outside the roots
# below, is reviewed. Files that steer agents are not documentation.
diff_scope_review_route() {
  local paths p docs=1 bridge
  paths=$(diff_scope_paths | grep -v '^$') || paths=""
  [ -n "$paths" ] || docs=0
  while IFS= read -r p; do
    case "$p" in
      .claude/* | */.claude/* | CLAUDE.md | */CLAUDE.md | AGENTS.md | */AGENTS.md) docs=0 ;;
      docs/* | specs/* | design/*.md | README.md) ;;
      *) docs=0 ;;
    esac
  done <<<"$paths"
  if [ "$docs" = 1 ]; then
    echo "none: documentation only"
    return 0
  fi
  bridge=$(diff_scope_bridge_changes | paste -sd, - | sed 's/,/, /g')
  echo "reviewer${bridge:+, sensitive: $bridge}"
}

# Each bridge change in the branch's diff, one per line: a changed line inside
# an item deriving Facet (what crosses the bridge, as check-bridge-tests.sh
# finds it), a BLOB_VERSION line, a store migration or the FFI crate's source.
# Reads the same files as diff_scope_paths; an untracked file is all added.
diff_scope_bridge_changes() {
  local base f
  base=$(git merge-base origin/main HEAD 2>/dev/null) || { echo "an unreadable diff"; return 0; }
  while IFS= read -r -d '' f; do
    case "$f" in crates/*.rs) ;; *) continue ;; esac
    if git ls-files --error-unmatch -- "$f" >/dev/null 2>&1 || git cat-file -e "$base:$f" 2>/dev/null; then
      git diff -U0 --no-renames "$base" -- "$f"
    else
      git diff -U0 --no-index -- /dev/null "$f"
    fi | BASE="$base" perl -e '
      sub spans {
        my (@out, $start, $want, $name, $depth);
        for my $i (0 .. $#_) {
          local $_ = $_[$i];
          if (!defined $name && /^\s*#\[/) { $start //= $i + 1; $want = 1 if /facet::Facet\b|effect\(facet_typegen\)/; next }
          if (!defined $name && $want && /^\s*pub\s+(?:struct|enum)\s+(\w+)/) { $name = $1; $depth = 0 }
          if (defined $name) {
            $depth += () = /\{/g; $depth -= () = /\}/g;
            if ($depth <= 0 && /[;}]\s*$/) { push @out, [$start, $i + 1, $name]; undef $name; undef $want; undef $start }
            next;
          }
          undef $start unless /^\s*(\/\/|$)/; undef $want unless defined $start;
        }
        return \@out;
      }
      sub hit { my ($spans, $n) = @_; for (@$spans) { return $_->[2] if $n >= $_->[0] && $n <= $_->[1] } undef }
      my $file = shift;
      exit 0 if $file =~ m{(^|/)tests?(/|\.rs$)|_tests\.rs$};
      my (@old, @new);
      if (open my $g, "-|", "git", "show", "$ENV{BASE}:$file") { @old = <$g>; close $g }
      if (open my $f, "<", $file) { @new = <$f> }
      my ($old, $new) = (spans(@old), spans(@new));
      my ($o, $n, %seen);
      while (<STDIN>) {
        if (/^@@ -(\d+)(?:,\d+)? \+(\d+)/) { ($o, $n) = ($1, $2); next }
        next unless defined $n && /^[-+](?![-+]{2} )/;
        my $at = /^-/ ? $o++ : $n++;
        my $type = hit(/^-/ ? $old : $new, $at);
        if ($type) { $seen{"t$type"}++ or print "$type in $file\n" }
        elsif (/BLOB_VERSION/) { $seen{b}++ or print "a blob pin in $file\n" }
        elsif ($file eq "crates/intrada-store/src/migrations.rs") { $seen{m}++ or print "a store migration\n" }
        elsif ($file =~ m{^crates/intrada-ffi/src/}) { $seen{f}++ or print "the FFI bridge in $file\n" }
      }
    ' "$f"
  done < <(git -c core.quotePath=false diff -z --name-only --no-renames "$base" --; git ls-files -z --others --exclude-standard)
}
