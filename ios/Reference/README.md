# Reference Swift (not built)

Preserved implementations mined from the deleted Tauri shell
(`crates/intrada-mobile`, removed in the clear-the-decks PR; see
`docs/rebuild-review.md`). Nothing in this folder is part of any Xcode
target; these files exist to be ported, not compiled.

- `BackgroundAudioPlugin.swift`: the audio-session groundwork a metronome
  or click track needs. `AVAudioSession(.playback, .mixWithOthers)`,
  silent-loop keep-alive so iOS does not suspend timers, interruption
  re-arm, `MPNowPlayingInfoCenter` seeding. Spec:
  `specs/background-audio-plugin.md`.
- `LiveActivityPlugin.swift`, `IntradaActivityWidget.swift`,
  `IntradaWidgetBundle.swift`, `IntradaActivityAttributes.swift`: a working
  ActivityKit Lock Screen / Dynamic Island practice timer. Spec:
  `specs/live-activity-plugin.md`.

The Tauri command/IPC layers these hung off are gone; only the native Swift
logic carries forward. Delete each file here once its logic has been ported
into the app proper.
