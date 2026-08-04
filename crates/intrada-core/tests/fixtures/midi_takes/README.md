# MIDI take fixtures

Real captures from the MIDI capture spike (PR 2 — see
`docs/rebuild-review.md` §6), recorded on a Roland LX-706 over Bluetooth MIDI.
Test fixtures for PR 3's attempt-segmentation spike, which they still drive.

**The iOS capture harness that produced them is deleted** (#1176): machine
listening is deferred by decision 18, and the shipping app kept only the click.
These takes are therefore not currently reproducible — recover the harness from
git history (`ios/Intrada/MidiSpike/`, up to and including 94537fc) when the
scoring path returns.

- **`take-01-freeplay-mixed-bluetooth.jsonl`** — the first exploratory
  capture: a long, unstructured session mixing a scale run, ad hoc chords,
  and a steady on-the-beat single-note passage. No fixed phrase.
- **`take-02-paused-mid-phrase-bluetooth.jsonl`** — plays the Gate Drill
  phrase (F C F C B F B F) correctly in order, but with a ~1.8s pause
  between notes 6 and 7 before finishing.
- **`take-03-noodle-after-start-bluetooth.jsonl`** — plays the first 4
  phrase notes correctly (F C F C), then diverges into a continuous
  ascending/descending scale run instead of continuing the phrase.
- **`take-04-restart-bluetooth.jsonl`** — plays notes 1-6 of the phrase,
  pauses (~1.8s), then goes back to note 1 and plays the full 8-note
  phrase through cleanly.
- **`take-05-collapse-bluetooth.jsonl`** — plays notes 1-5 correctly, then
  hits a wrong note (E instead of F) at note 6, and stops — no recovery.

Each file is JSONL: one `take_header` line (transport, bpm, beat grid,
click start), then one line per captured MIDI note event, annotated with
bar/beat/offsetMs against that take's own beat grid.
