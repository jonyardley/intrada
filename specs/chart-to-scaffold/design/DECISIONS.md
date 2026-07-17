# Chart-to-scaffold — design decisions

Mocks: `chart-entry.dc.html`, `scaffold-preview.dc.html`. Built on the Paper &
Score system (tokens from `Theme.swift`). Also pushed to Claude Design →
`mockups/chart-to-scaffold/`.

## Chart entry (`chart-entry.dc.html`)

- **Lives as a "Chord chart" card on the piece `LibraryDetailScreen`**, above the
  Related exercises section — chart is a property of the piece, not a new screen.
- **Two states mocked:** parsed (read-only bar grid + "See the curriculum" CTA)
  and editing-with-parse-error.
- **Parsed view = a bar grid**, not the raw text. Chord symbols in Source Serif
  (matches score/title voice); 4 bars per row; section label as an eyebrow.
- **Parse errors surface against the offending token** (invariant: never swallow):
  the bad token is highlighted inline in the mono editor *and* an error card names
  the bar, why it failed, and suggests the nearest valid symbols. Never a partial
  chart — Save is blocked until it parses.
- **Grammar is the bar-and-pipe we control** (Resolved #3): `[Section]` labels,
  one bar between `|` pipes. A "Format" hint card teaches it in place.
- **"See the curriculum"** is the one primary action (one-primary-per-screen).
  Copy leads with the promise, not the mechanism.

## Scaffold preview (`scaffold-preview.dc.html`)

- **A bottom sheet of selectable rows**, mirroring the linked-exercise Picker A
  (checkbox + gold exercise bar + serif title + one-line rationale).
- **Subtitle states key-awareness + agency:** "Derived in G minor · 5 exercises ·
  nothing added yet" — reinforces *nothing is created until you commit*.
- **Rationale is one plain-language line per exercise** (musician's words, not
  theory jargon — e.g. "3rd + 7th of every chord", not "guide-tone dyads").
- **Three row states** (see the anatomy panel): selected (purple check, will be
  added), deselected (empty box), already-linked (locked, greyed, "Already
  linked" tag — the dedup/reconciliation surface, Key decision #6).
- **Fallback badge** (gold "↳ N fallback") marks a spec whose changes fell back to
  the arpeggio because a chord was out of the v1 vocabulary — surfaced, not hidden.
- **Commit is Phase B.** The mock shows the sticky "Add N exercises" button for
  design completeness, annotated "ships in Phase B". Phase A renders this sheet
  **read-only** (no commit, no selection persistence yet).

## Deferred to build

- iPad `SplitView`, VoiceOver labels, Dynamic Type — built *with* the SwiftUI
  screens, not mocked here.
- Real fallback data: the mock's fallback badge is illustrative; Autumn Leaves'
  changes are all in-vocabulary.
