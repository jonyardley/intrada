#!/usr/bin/env bash
# Flag em dashes and en dashes added to prose or comments, not string literals,
# and double dashes and American spellings added to Markdown prose (#2100).
# See docs/style-guide.md: house style is plain British English with none.
# This gates the rule on changed lines only, the same diff-scoped way
# check-comment-density.sh works, so it binds new content without rewriting a
# tree that already carries hundreds of historical dashes. Bypass a genuinely
# justified case (a vendored notice, quoted material) with SKIP_DASH_CHECK=1.

set -euo pipefail

if [ "${SKIP_DASH_CHECK:-}" = "1" ]; then
  exit 0
fi

# CI passes the PR base via DASH_CHECK_BASE (HEAD is detached on a PR checkout).
# Locally the pre-push hook leaves it unset and we derive origin/main.
base="${DASH_CHECK_BASE:-}"
if [ -z "$base" ]; then
  branch=$(git symbolic-ref --short HEAD 2>/dev/null || true)
  if [ -z "$branch" ]; then
    exit 0
  fi
  case "$branch" in
    main|master) exit 0 ;;
  esac
  base="origin/main"
fi

if ! git rev-parse --verify "$base" >/dev/null 2>&1; then
  exit 0
fi

range="$base...HEAD"

# em dash U+2014, en dash U+2013, written as raw UTF-8 bytes via ANSI-C quoting
# (bash 3.2 has no \u) so the literal glyphs never appear in this file.
em=$'\xe2\x80\x94'
en=$'\xe2\x80\x93'
dash_re="$em|$en"

# Prose and comment surfaces only. The label-separator exception (#1231) lives
# here as an exemption: the three structured docs that use it as house style are
# excluded, along with the archived specs kept only for reference.
files=$(git diff "$range" --name-only -- \
  '*.md' '*.rs' '*.swift' '*.sh' '*.py' '*.yml' '*.yaml' \
  ':(exclude)CLAUDE.md' \
  ':(exclude)docs/roadmap.md' \
  ':(exclude)design/CLAUDE.md' \
  ':(exclude)specs/_archive/**' \
  2>/dev/null || true)

if [ -z "$files" ]; then
  exit 0
fi

found=0
report=""
while IFS= read -r f; do
  [ -n "$f" ] || continue
  added=$(git diff "$range" -- "$f" | grep -E '^\+' | grep -vE '^\+\+\+' || true)
  case "$f" in
    # A string literal in code is data, often the core's own output (the dash
    # it shows for an empty duration), and changing it moves snapshot references.
    *.swift | *.rs)
      hits=$(printf '%s\n' "$added" | perl -ne '
        my $code = $_;
        $code =~ s/"(?:[^"\\]|\\.)*"//g unless $code =~ m{^\+\s*//};
        print if $code =~ /\xe2\x80[\x93\x94]/;
      ')
      ;;
    *) hits=$(printf '%s\n' "$added" | grep -E "$dash_re" || true) ;;
  esac
  if [ -n "$hits" ]; then
    found=1
    report="$report"$'\n'"  $f:"$'\n'"$(printf '%s\n' "$hits" | sed 's/^/    /')"
  fi
done <<EOF
$files
EOF

# ── Double dashes and American spellings in Markdown prose ──
# Judged against the file as committed rather than the diff, so a line inside a
# fenced block is known to be code even when the fence itself did not change.
# Only words that are unambiguously wrong here; code spans and link targets are
# exempt, since an API name such as `Color` is spelt the way its owner spells it.
americanisms='analyze|analyzed|analyzes|analyzing|behavior|behaviors|center|centered|centers|color|colors|defense|favor|favors|favorite|fulfillment|maximize|minimize|optimize|optimized|optimizing|organize|organized|organizing|organization|organizations|prioritize|prioritized|prioritizing|recognize|recognized|recognizing|standardize|standardized|summarize|summarized|summarizing|traveled|traveling'

prose_files=$(git diff "$range" --name-only --diff-filter=d -- '*.md' \
  ':(exclude)specs/_archive/**' 2>/dev/null || true)

while IFS= read -r f; do
  [ -n "$f" ] || continue
  added_lines=$(git diff -U0 "$range" -- "$f" | perl -ne '
    print join(" ", $1 .. $1 + (defined $2 ? $2 : 1) - 1), " "
      if /^@@ -\S+ \+(\d+)(?:,(\d+))? @@/;
  ')
  [ -n "$added_lines" ] || continue
  hits=$(git show "HEAD:$f" | ADDED="$added_lines" WORDS="$americanisms" perl -ne '
    BEGIN { %added = map { $_ => 1 } split " ", $ENV{ADDED}; $fence = 0 }
    if (/^\s*(?:```|~~~)/) { $fence = !$fence; next }
    next if $fence || !$added{$.};
    my $prose = $_;
    $prose =~ s/`[^`]*`//g;
    $prose =~ s/<!--|-->//g;
    $prose =~ s/\]\([^)]*\)/]/g;
    next if $prose =~ /^\s*\|?[\s:|-]+\|?\s*$/;
    print "+$_" if $prose =~ /(?<!-)--(?!-)/ || $prose =~ /\b(?:$ENV{WORDS})\b/i;
  ')
  if [ -n "$hits" ]; then
    found=1
    report="$report"$'\n'"  $f:"$'\n'"$(printf '%s\n' "$hits" | sed 's/^/    /')"
  fi
done <<EOF
$prose_files
EOF

if [ "$found" = "1" ]; then
  cat <<EOF >&2

Blocked: this branch adds em or en dashes to prose or comments, or double
dashes or American spellings to Markdown prose.
$report

House style is plain British English with no dashes of either kind. Replace a
dash with a comma, a colon, a full stop, or a rephrase, and use the British
spelling. A command or identifier belongs in a code span.

If a case is genuinely justified, bypass with:

  SKIP_DASH_CHECK=1 git push

EOF
  exit 1
fi

exit 0
