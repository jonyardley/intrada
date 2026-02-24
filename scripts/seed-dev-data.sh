#!/usr/bin/env bash
# seed-dev-data.sh — Populate the intrada API with realistic music practice data.
#
# Usage:
#   bash scripts/seed-dev-data.sh              # seed into localhost:3001 (API)
#   bash scripts/seed-dev-data.sh --live       # seed into production (Fly.io)
#   bash scripts/seed-dev-data.sh --clean      # delete all data first, then seed
#   API_URL=https://... bash scripts/seed-dev-data.sh   # custom API URL
#
# Requires: curl, jq

set -euo pipefail

LIVE_API_URL="https://intrada-api.fly.dev"
LIVE=false
CLEAN=false

for arg in "$@"; do
  case "$arg" in
    --live) LIVE=true ;;
    --clean) CLEAN=true ;;
    *) echo "Unknown argument: $arg"; exit 1 ;;
  esac
done

if [ "$LIVE" = true ]; then
  API_URL="${API_URL:-$LIVE_API_URL}"
  echo "⚠️  Targeting LIVE environment: $API_URL"
  echo "   Press Enter to continue or Ctrl+C to abort..."
  read -r
else
  API_URL="${API_URL:-http://localhost:3001}"
fi

# Check dependencies
if ! command -v jq &>/dev/null; then
  echo "Error: jq is required. Install with: brew install jq"
  exit 1
fi

# Helper: POST JSON and return the response body
post() {
  local path="$1"
  local data="$2"
  curl -sf -X POST "${API_URL}${path}" \
    -H "Content-Type: application/json" \
    -d "$data"
}

# Helper: extract "id" from JSON response
extract_id() {
  jq -r '.id'
}

# Helper: generate a ULID-like ID (26 chars, uppercase alphanumeric)
# Not a real ULID but unique enough for seed data
gen_id() {
  local prefix="$1"
  printf "SEED%s%012d" "$prefix" "$RANDOM$RANDOM"
}

# ── Clean existing data ─────────────────────────────────────────────

if [ "$CLEAN" = true ]; then
  echo "Cleaning existing data..."

  # Delete all sessions
  session_ids=$(curl -sf "${API_URL}/api/sessions" | jq -r '.[].id')
  for id in $session_ids; do
    curl -sf -X DELETE "${API_URL}/api/sessions/${id}" > /dev/null 2>&1 || true
  done
  echo "  Deleted sessions"

  # Delete all items (pieces + exercises)
  item_ids=$(curl -sf "${API_URL}/api/items" | jq -r '.[].id')
  for id in $item_ids; do
    curl -sf -X DELETE "${API_URL}/api/items/${id}" > /dev/null 2>&1 || true
  done
  echo "  Deleted items"

  echo "  Done ✓"
  echo ""
fi

# ── Create Items (Pieces) ──────────────────────────────────────────

echo "Creating 8 pieces..."

P1_ID=$(post "/api/items" '{
  "title": "Clair de Lune",
  "kind": "piece",
  "composer": "Claude Debussy",
  "key": "Db Major",
  "tempo": { "marking": "Andante très expressif", "bpm": 66 },
  "notes": "Third movement of Suite bergamasque. Focus on pedalling and voicing.",
  "tags": ["impressionist", "romantic"]
}' | extract_id)
echo "  Clair de Lune ($P1_ID)"

P2_ID=$(post "/api/items" '{
  "title": "Moonlight Sonata, Mvt. 1",
  "kind": "piece",
  "composer": "Ludwig van Beethoven",
  "key": "C# Minor",
  "tempo": { "marking": "Adagio sostenuto", "bpm": 56 },
  "notes": "Piano Sonata No. 14, Op. 27 No. 2. Maintain triplet evenness throughout.",
  "tags": ["classical", "sonata"]
}' | extract_id)
echo "  Moonlight Sonata ($P2_ID)"

P3_ID=$(post "/api/items" '{
  "title": "Nocturne Op. 9 No. 2",
  "kind": "piece",
  "composer": "Frédéric Chopin",
  "key": "Eb Major",
  "tempo": { "marking": "Andante", "bpm": 69 },
  "notes": "Work on rubato and ornamental turns in the melody.",
  "tags": ["romantic", "nocturne"]
}' | extract_id)
echo "  Nocturne Op. 9 No. 2 ($P3_ID)"

P4_ID=$(post "/api/items" '{
  "title": "Gymnopédie No. 1",
  "kind": "piece",
  "composer": "Erik Satie",
  "key": "D Major",
  "tempo": { "marking": "Lent et douloureux", "bpm": 72 },
  "notes": "Keep a steady, unhurried pulse. Minimal pedal.",
  "tags": ["impressionist", "minimalist"]
}' | extract_id)
echo "  Gymnopédie No. 1 ($P4_ID)"

P5_ID=$(post "/api/items" '{
  "title": "Prelude in C Major, BWV 846",
  "kind": "piece",
  "composer": "Johann Sebastian Bach",
  "key": "C Major",
  "tempo": { "marking": "Moderato", "bpm": 80 },
  "notes": "Well-Tempered Clavier Book 1. Finger independence and even arpeggiation.",
  "tags": ["baroque", "prelude"]
}' | extract_id)
echo "  Prelude in C Major ($P5_ID)"

P6_ID=$(post "/api/items" '{
  "title": "Rêverie",
  "kind": "piece",
  "composer": "Claude Debussy",
  "key": "F Major",
  "tempo": { "marking": "Andantino", "bpm": 72 },
  "notes": "Dreamy quality — soft dynamics throughout, gentle phrasing.",
  "tags": ["impressionist", "romantic"]
}' | extract_id)
echo "  Rêverie ($P6_ID)"

P7_ID=$(post "/api/items" '{
  "title": "Arabesque No. 1",
  "kind": "piece",
  "composer": "Claude Debussy",
  "key": "E Major",
  "tempo": { "marking": "Andantino con moto", "bpm": 96 },
  "notes": "Flowing triplets against duplets. Work on hand independence.",
  "tags": ["impressionist"]
}' | extract_id)
echo "  Arabesque No. 1 ($P7_ID)"

P8_ID=$(post "/api/items" '{
  "title": "Ballade No. 1 in G Minor",
  "kind": "piece",
  "composer": "Frédéric Chopin",
  "key": "G Minor",
  "tempo": { "marking": "Largo", "bpm": 66 },
  "notes": "Op. 23. The coda needs separate slow practice. Watch out for the octave passages.",
  "tags": ["romantic", "ballade"]
}' | extract_id)
echo "  Ballade No. 1 ($P8_ID)"

echo "  Done ✓"
echo ""

# ── Create Items (Exercises) ───────────────────────────────────────

echo "Creating 5 exercises..."

E1_ID=$(post "/api/items" '{
  "title": "Hanon No. 1",
  "kind": "exercise",
  "composer": "Charles-Louis Hanon",
  "category": "Technique",
  "key": "C Major",
  "tempo": { "marking": "Moderato", "bpm": 108 },
  "notes": "The Virtuoso Pianist — Exercise 1. Hands together, even touch.",
  "tags": ["technique", "warm-up"]
}' | extract_id)
echo "  Hanon No. 1 ($E1_ID)"

E2_ID=$(post "/api/items" '{
  "title": "C Major Scale — 2 octaves",
  "kind": "exercise",
  "composer": null,
  "category": "Scales",
  "key": "C Major",
  "tempo": { "marking": "Andante", "bpm": 80 },
  "notes": "Hands together, parallel motion. Smooth thumb crossings.",
  "tags": ["scales", "warm-up"]
}' | extract_id)
echo "  C Major Scale ($E2_ID)"

E3_ID=$(post "/api/items" '{
  "title": "Chromatic Scale — hands together",
  "kind": "exercise",
  "composer": null,
  "category": "Scales",
  "key": null,
  "tempo": { "marking": "Allegro", "bpm": 120 },
  "notes": "4 octaves ascending and descending. Focus on fingering consistency.",
  "tags": ["scales", "technique"]
}' | extract_id)
echo "  Chromatic Scale ($E3_ID)"

E4_ID=$(post "/api/items" '{
  "title": "Arpeggios — Major keys",
  "kind": "exercise",
  "composer": null,
  "category": "Arpeggios",
  "key": null,
  "tempo": { "marking": "Moderato", "bpm": 92 },
  "notes": "All 12 major keys, 2 octaves each. Wrist rotation, relaxed arm.",
  "tags": ["arpeggios", "technique"]
}' | extract_id)
echo "  Arpeggios ($E4_ID)"

E5_ID=$(post "/api/items" '{
  "title": "Czerny Op. 299 No. 1",
  "kind": "exercise",
  "composer": "Carl Czerny",
  "category": "Technique",
  "key": "C Major",
  "tempo": { "marking": "Allegro", "bpm": 132 },
  "notes": "School of Velocity. Right hand runs, keep left hand steady.",
  "tags": ["technique", "velocity"]
}' | extract_id)
echo "  Czerny Op. 299 No. 1 ($E5_ID)"

echo "  Done ✓"
echo ""

# ── Create Practice Sessions ────────────────────────────────────────
#
# We create ~25 sessions over the past 35 days to produce interesting
# analytics: streaks, gaps, varied daily totals, score progression.

echo "Creating ~25 practice sessions..."

# File-based counter for unique entry IDs (persists across subshells)
ENTRY_SEQ_FILE=$(mktemp)
echo "0" > "$ENTRY_SEQ_FILE"
trap 'rm -f "$ENTRY_SEQ_FILE"' EXIT

next_entry_id() {
  local seq
  seq=$(cat "$ENTRY_SEQ_FILE")
  seq=$((seq + 1))
  echo "$seq" > "$ENTRY_SEQ_FILE"
  printf "SEEDENTRY%016d" "$seq"
}

# Helper: create a session via the API
# Usage: create_session <date> <start_hour> <duration_mins> <completion_status> <notes> <entries_json>
create_session() {
  local date="$1"
  local start_hour="$2"
  local duration_mins="$3"
  local status="$4"
  local notes="$5"
  local entries="$6"

  local duration_secs=$((duration_mins * 60))
  local started_at="${date}T${start_hour}:00Z"

  # Calculate end time (approximate — just add duration)
  # Use 10# prefix to force base-10 (avoids bash treating "09" as invalid octal)
  local start_h=${start_hour%%:*}
  local start_m=${start_hour##*:}
  local total_mins=$((10#$start_h * 60 + 10#$start_m + duration_mins))
  local end_h=$((total_mins / 60))
  local end_m=$((total_mins % 60))
  local completed_at
  completed_at=$(printf "%sT%02d:%02d:00Z" "$date" "$end_h" "$end_m")

  local notes_json="null"
  if [ "$notes" != "null" ]; then
    notes_json="\"$notes\""
  fi

  local payload
  payload=$(jq -n \
    --argjson entries "$entries" \
    --argjson notes "$notes_json" \
    --arg started "$started_at" \
    --arg completed "$completed_at" \
    --argjson duration "$duration_secs" \
    --arg status "$status" \
    '{entries: $entries, session_notes: $notes, started_at: $started, completed_at: $completed, total_duration_secs: $duration, completion_status: $status}')

  post "/api/sessions" "$payload" > /dev/null

  echo "  ${date} (${duration_mins}m, ${status})"
}

# Helper: build an entry JSON object
# Usage: entry <item_id> <item_title> <item_type> <position> <duration_secs> <status> <score_or_null> [achieved_tempo_or_null]
entry() {
  local eid
  eid=$(next_entry_id)
  local score_json="null"
  if [ "$7" != "null" ]; then
    score_json="$7"
  fi
  local tempo_json="null"
  if [ "${8:-null}" != "null" ]; then
    tempo_json="$8"
  fi
  printf '{"id":"%s","item_id":"%s","item_title":"%s","item_type":"%s","position":%d,"duration_secs":%d,"status":"%s","notes":null,"score":%s,"achieved_tempo":%s}' \
    "$eid" "$1" "$2" "$3" "$4" "$5" "$6" "$score_json" "$tempo_json"
}

# ── Day -35: First session (35 days ago) ─────────────────
create_session "2026-01-13" "18:00" 45 "Completed" "First session back after break" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 72),
  $(entry "$E2_ID" "C Major Scale — 2 octaves" "exercise" 1 300 "Completed" "null"),
  $(entry "$P5_ID" "Prelude in C Major, BWV 846" "piece" 2 1200 "Completed" 2 55),
  $(entry "$P2_ID" "Moonlight Sonata, Mvt. 1" "piece" 3 600 "Completed" 2)
]"

# ── Day -33 ──────────────────────────────────────────────
create_session "2026-01-15" "17:30" 60 "Completed" "null" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 74),
  $(entry "$E4_ID" "Arpeggios — Major keys" "exercise" 1 600 "Completed" "null"),
  $(entry "$P1_ID" "Clair de Lune" "piece" 2 1200 "Completed" 2),
  $(entry "$P3_ID" "Nocturne Op. 9 No. 2" "piece" 3 1200 "Completed" 2)
]"

# ── Day -31 ──────────────────────────────────────────────
create_session "2026-01-17" "18:00" 30 "EndedEarly" "Had to stop early — hand fatigue" "[
  $(entry "$E2_ID" "C Major Scale — 2 octaves" "exercise" 0 300 "Completed" "null"),
  $(entry "$P4_ID" "Gymnopédie No. 1" "piece" 1 900 "Completed" 3),
  $(entry "$P8_ID" "Ballade No. 1 in G Minor" "piece" 2 600 "Completed" 1)
]"

# ── Day -29 ──────────────────────────────────────────────
create_session "2026-01-19" "10:00" 75 "Completed" "Weekend morning practice — good focus" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 76),
  $(entry "$E3_ID" "Chromatic Scale — hands together" "exercise" 1 600 "Completed" "null"),
  $(entry "$E5_ID" "Czerny Op. 299 No. 1" "exercise" 2 600 "Completed" "null" 80),
  $(entry "$P1_ID" "Clair de Lune" "piece" 3 1500 "Completed" 3),
  $(entry "$P2_ID" "Moonlight Sonata, Mvt. 1" "piece" 4 1200 "Completed" 3)
]"

# ── Day -27 ──────────────────────────────────────────────
create_session "2026-01-21" "18:30" 40 "Completed" "null" "[
  $(entry "$E2_ID" "C Major Scale — 2 octaves" "exercise" 0 300 "Completed" "null"),
  $(entry "$P3_ID" "Nocturne Op. 9 No. 2" "piece" 1 1200 "Completed" 3),
  $(entry "$P6_ID" "Rêverie" "piece" 2 900 "Completed" 2)
]"

# ── Day -25 ──────────────────────────────────────────────
create_session "2026-01-23" "19:00" 50 "Completed" "null" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 78),
  $(entry "$E4_ID" "Arpeggios — Major keys" "exercise" 1 600 "Completed" "null"),
  $(entry "$P5_ID" "Prelude in C Major, BWV 846" "piece" 2 900 "Completed" 3 60),
  $(entry "$P7_ID" "Arabesque No. 1" "piece" 3 900 "Completed" 2)
]"

# ── Day -23: Two sessions (streak start) ─────────────────
create_session "2026-01-25" "09:30" 35 "Completed" "Morning technique" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 80),
  $(entry "$E3_ID" "Chromatic Scale — hands together" "exercise" 1 600 "Completed" "null"),
  $(entry "$E5_ID" "Czerny Op. 299 No. 1" "exercise" 2 900 "Completed" "null" 85)
]"

create_session "2026-01-25" "18:00" 55 "Completed" "Evening repertoire" "[
  $(entry "$P1_ID" "Clair de Lune" "piece" 0 1200 "Completed" 3),
  $(entry "$P8_ID" "Ballade No. 1 in G Minor" "piece" 1 1200 "Completed" 2),
  $(entry "$P3_ID" "Nocturne Op. 9 No. 2" "piece" 2 900 "Completed" 3)
]"

# ── Day -22 (streak continues) ───────────────────────────
create_session "2026-01-26" "18:00" 45 "Completed" "null" "[
  $(entry "$E2_ID" "C Major Scale — 2 octaves" "exercise" 0 300 "Completed" "null"),
  $(entry "$P2_ID" "Moonlight Sonata, Mvt. 1" "piece" 1 1200 "Completed" 3),
  $(entry "$P4_ID" "Gymnopédie No. 1" "piece" 2 900 "Completed" 4),
  $(entry "$P6_ID" "Rêverie" "piece" 3 300 "Skipped" "null")
]"

# ── Day -21 (streak continues) ───────────────────────────
create_session "2026-01-27" "17:00" 60 "Completed" "null" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 82),
  $(entry "$P7_ID" "Arabesque No. 1" "piece" 1 1200 "Completed" 3),
  $(entry "$P1_ID" "Clair de Lune" "piece" 2 1200 "Completed" 3),
  $(entry "$P5_ID" "Prelude in C Major, BWV 846" "piece" 3 600 "Completed" 4 64)
]"

# ── Day -20 (streak = 4 days) ────────────────────────────
create_session "2026-01-28" "18:30" 30 "EndedEarly" "Short session, tired" "[
  $(entry "$E4_ID" "Arpeggios — Major keys" "exercise" 0 600 "Completed" "null"),
  $(entry "$P3_ID" "Nocturne Op. 9 No. 2" "piece" 1 1200 "Completed" 4)
]"

# ── Gap: Days -19, -18 (no practice) ─────────────────────

# ── Day -17 ──────────────────────────────────────────────
create_session "2026-01-31" "18:00" 55 "Completed" "Back after 2-day break" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 84),
  $(entry "$E2_ID" "C Major Scale — 2 octaves" "exercise" 1 300 "Completed" "null"),
  $(entry "$P8_ID" "Ballade No. 1 in G Minor" "piece" 2 1200 "Completed" 2),
  $(entry "$P2_ID" "Moonlight Sonata, Mvt. 1" "piece" 3 1200 "Completed" 4)
]"

# ── Day -15 ──────────────────────────────────────────────
create_session "2026-02-02" "10:00" 90 "Completed" "Long weekend session — really productive" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 86),
  $(entry "$E3_ID" "Chromatic Scale — hands together" "exercise" 1 600 "Completed" "null"),
  $(entry "$E5_ID" "Czerny Op. 299 No. 1" "exercise" 2 600 "Completed" "null" 92),
  $(entry "$P1_ID" "Clair de Lune" "piece" 3 1500 "Completed" 4),
  $(entry "$P8_ID" "Ballade No. 1 in G Minor" "piece" 4 2100 "Completed" 3)
]"

# ── Day -13 ──────────────────────────────────────────────
create_session "2026-02-04" "18:30" 40 "Completed" "null" "[
  $(entry "$E2_ID" "C Major Scale — 2 octaves" "exercise" 0 300 "Completed" "null"),
  $(entry "$P4_ID" "Gymnopédie No. 1" "piece" 1 900 "Completed" 4),
  $(entry "$P6_ID" "Rêverie" "piece" 2 900 "Completed" 3),
  $(entry "$P7_ID" "Arabesque No. 1" "piece" 3 300 "Completed" 3)
]"

# ── Day -11 ──────────────────────────────────────────────
create_session "2026-02-06" "17:00" 50 "Completed" "null" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 88),
  $(entry "$E4_ID" "Arpeggios — Major keys" "exercise" 1 600 "Completed" "null"),
  $(entry "$P3_ID" "Nocturne Op. 9 No. 2" "piece" 2 900 "Completed" 4),
  $(entry "$P5_ID" "Prelude in C Major, BWV 846" "piece" 3 900 "Completed" 4 68)
]"

# ── Day -9 ───────────────────────────────────────────────
create_session "2026-02-08" "11:00" 65 "Completed" "null" "[
  $(entry "$E3_ID" "Chromatic Scale — hands together" "exercise" 0 600 "Completed" "null"),
  $(entry "$E5_ID" "Czerny Op. 299 No. 1" "exercise" 1 600 "Completed" "null" 98),
  $(entry "$P1_ID" "Clair de Lune" "piece" 2 1200 "Completed" 4),
  $(entry "$P2_ID" "Moonlight Sonata, Mvt. 1" "piece" 3 1200 "Completed" 4),
  $(entry "$P8_ID" "Ballade No. 1 in G Minor" "piece" 4 300 "Completed" 3)
]"

# ── Day -7 (one week ago, streak start) ──────────────────
create_session "2026-02-10" "18:00" 45 "Completed" "null" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 90),
  $(entry "$P3_ID" "Nocturne Op. 9 No. 2" "piece" 1 1200 "Completed" 4),
  $(entry "$P6_ID" "Rêverie" "piece" 2 900 "Completed" 4)
]"

# ── Day -6 ───────────────────────────────────────────────
create_session "2026-02-11" "17:30" 55 "Completed" "null" "[
  $(entry "$E2_ID" "C Major Scale — 2 octaves" "exercise" 0 300 "Completed" "null"),
  $(entry "$E4_ID" "Arpeggios — Major keys" "exercise" 1 600 "Completed" "null"),
  $(entry "$P7_ID" "Arabesque No. 1" "piece" 2 1200 "Completed" 4),
  $(entry "$P4_ID" "Gymnopédie No. 1" "piece" 3 1200 "Completed" 5)
]"

# ── Day -5 ───────────────────────────────────────────────
create_session "2026-02-12" "18:00" 35 "EndedEarly" "Had to leave early" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 92),
  $(entry "$P1_ID" "Clair de Lune" "piece" 1 1200 "Completed" 4),
  $(entry "$P5_ID" "Prelude in C Major, BWV 846" "piece" 2 300 "Skipped" "null")
]"

# ── Day -4 ───────────────────────────────────────────────
create_session "2026-02-13" "18:30" 50 "Completed" "null" "[
  $(entry "$E3_ID" "Chromatic Scale — hands together" "exercise" 0 600 "Completed" "null"),
  $(entry "$P2_ID" "Moonlight Sonata, Mvt. 1" "piece" 1 1200 "Completed" 5),
  $(entry "$P8_ID" "Ballade No. 1 in G Minor" "piece" 2 1200 "Completed" 3)
]"

# ── Day -3 ───────────────────────────────────────────────
create_session "2026-02-14" "10:00" 70 "Completed" "Valentine's Day practice — playing for someone special" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 94),
  $(entry "$E5_ID" "Czerny Op. 299 No. 1" "exercise" 1 600 "Completed" "null" 105),
  $(entry "$P1_ID" "Clair de Lune" "piece" 2 1500 "Completed" 5),
  $(entry "$P3_ID" "Nocturne Op. 9 No. 2" "piece" 3 1200 "Completed" 5),
  $(entry "$P6_ID" "Rêverie" "piece" 4 300 "Completed" 4)
]"

# ── Day -2 ───────────────────────────────────────────────
create_session "2026-02-15" "18:00" 40 "Completed" "null" "[
  $(entry "$E2_ID" "C Major Scale — 2 octaves" "exercise" 0 300 "Completed" "null"),
  $(entry "$P4_ID" "Gymnopédie No. 1" "piece" 1 1200 "Completed" 5),
  $(entry "$P7_ID" "Arabesque No. 1" "piece" 2 900 "Completed" 4)
]"

# ── Day -1 (yesterday) ──────────────────────────────────
create_session "2026-02-16" "17:00" 60 "Completed" "null" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 96),
  $(entry "$E4_ID" "Arpeggios — Major keys" "exercise" 1 600 "Completed" "null"),
  $(entry "$P2_ID" "Moonlight Sonata, Mvt. 1" "piece" 2 1200 "Completed" 5),
  $(entry "$P5_ID" "Prelude in C Major, BWV 846" "piece" 3 900 "Completed" 5 74),
  $(entry "$P8_ID" "Ballade No. 1 in G Minor" "piece" 4 300 "Completed" 3)
]"

# ── Day 0 (today) ───────────────────────────────────────
create_session "2026-02-17" "09:00" 45 "Completed" "Morning practice before work" "[
  $(entry "$E1_ID" "Hanon No. 1" "exercise" 0 600 "Completed" "null" 98),
  $(entry "$E3_ID" "Chromatic Scale — hands together" "exercise" 1 600 "Completed" "null"),
  $(entry "$P1_ID" "Clair de Lune" "piece" 2 1200 "Completed" 5),
  $(entry "$P3_ID" "Nocturne Op. 9 No. 2" "piece" 3 300 "Completed" 5)
]"

echo "  Done ✓"
echo ""

# ── Summary ─────────────────────────────────────────────────────────

echo "═══════════════════════════════════════════════"
echo "  Seed data complete!"
echo ""
echo "  Library:  8 pieces + 5 exercises"
echo "  Sessions: 25 practice sessions over 35 days"
echo ""
echo "  Analytics highlights:"
echo "    • 8-day current streak (Feb 10–17)"
echo "    • Score progression: most items 2→5"
echo "    • Weekly total: ~5 sessions this week"
echo "    • Busiest day: Feb 14 (70 min)"
echo ""
echo "  Tempo progress (for tempo charts):"
echo "    • Hanon No. 1:    72→98 BPM over 14 sessions (target 108)"
echo "    • Czerny Op. 299: 80→105 BPM over 5 sessions (target 132)"
echo "    • Prelude BWV 846: 55→74 BPM over 6 sessions (target 80)"
echo "═══════════════════════════════════════════════"
