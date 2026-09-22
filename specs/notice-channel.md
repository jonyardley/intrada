# Notice channel: a calm banner for an outcome that is not a failure

> Tier 3 (a new field on the `ViewModel`, so the bridge shape changes). Spec
> rides with the core branch per CLAUDE.md. Issue #1325, finding 18 of
> [`docs/audit-2026-09.md`](../docs/audit-2026-09.md), epic #1967. Native iOS
> only.

## Problem

`ViewModel.error` is the only string the core can hand the shell, and the shell
renders it one way: `GlobalBanner` in `IntradaColor.danger` with a warning
triangle. So the app can say "that failed" and nothing else. An outcome that is
true, expected and not a failure has nowhere to go, and today the core's answer
is to say nothing.

The concrete case the audit found: when a play closes with the metronome
sounding, the core converts the displayed tempo to crotchets and keeps it on
the play (`stamp_tempo` in `domain/session.rs`). A reading it cannot keep, such
as a minim-beat metre past 250 BPM, which is over 500 crotchets and outside
`validate_achieved_tempo`, stamps nothing and raises nothing. The play the
musician just made has no tempo, the tempo trend has a hole, and the screen
never said why. Raising it as an error would be wrong too: the tap landed, the
item completed, the session went on. It is not a refusal.

The two channels the shell has today:

| Channel | Means | Shell renders |
|---|---|---|
| `ViewModel.error` | the core refused the event, or a background write failed | red banner, no success haptic |
| the bridge throwing | the core could not run at all | the halted banner |

This spec adds the third.

## Decisions

1. **A separate field, not a tone on `error`.** `ViewModel.notice: Option<String>`
   and `ViewModel.notice_seq: u64`, beside `error` and `error_seq`. The reason
   is the haptic: `Store.sendAccepted` reads "did `errorSeq` move" as "was the
   event refused", and a notice must not read as a refusal. A tone on the
   existing field would make every shell reader branch on it; a second field
   leaves the refusal signal exactly as it is.
2. **A notice is never muted.** The dismiss mute (`Model::error_muted`, #346,
   #1936) exists to stop a background failure re-popping a banner the musician
   already closed. A notice is a consequence of their own action, like a
   refusal, so it always shows. `raise_notice` replaces the standing message and
   bumps `notice_seq` on every call, identical text included, the contract
   `error_seq` keeps for `raise_error`.
3. **`Event::ClearNotice` dismisses it.** Nothing else clears a notice: not a
   later accepted event, not a new session. The musician closes it or a newer
   notice replaces it. `ClearError` and `ClearNotice` are independent, and an
   error and a notice can stand at the same time.
4. **The first notice is the unusable tempo reading.** Every close path that
   carries a reading (`PrepareReflection`, `NextItem`, `EndSessionEarly`,
   `SwitchVariation`) reports a sounding reading the core could not keep. A
   silent reading, a valid one and a skip (which carries no reading) raise
   nothing. The reflection sheet and the tap that follows it both close the
   same play with the same reading, so the notice is raised twice with the
   same text; the shell shows one banner.
5. **The screen is a second PR.** The shell half (a neutral `GlobalBanner`
   tone on `inkSecondary` over `cardFill` with a hairline underneath, chosen
   by Jon on 2026-09-22 from the two tones in
   [`notice-channel/design/notice-banner.html`](notice-channel/design/notice-banner.html),
   rendered where the red one is in `RootView` and `PlayerHost`, the haptic
   left as it is since `errorSeq` does not move) ships after the core PR merges,
   in the same working session (#1348, #1374). Design principle T30 records
   the two tones.

## Copy

The notice on screen, written against `docs/tone-of-voice.md` (a musician's
words, metronome never click, crotchet never quarter note):

> That metronome setting doesn't give a crotchet tempo, so this play has none.

The summary's tempo field stays editable, so the musician can type one there.
The string lives in the core (`UNUSABLE_TEMPO_NOTICE` in `domain/session.rs`),
which is where the copy for `error` already lives.

## What the core PR touches

- `model.rs`: `Model::last_notice`, `Model::notice_seq`, `raise_notice`,
  `clear_notice`; `ViewModel::notice`, `ViewModel::notice_seq`.
- `app.rs`: `Event::ClearNotice`; the view projection copies both fields.
- `domain/session.rs`: `close_open_play` reports what it did with the reading
  (`TempoStamp`), and the four handlers that pass a reading raise the notice
  on `Unusable` once their borrow of the session ends. `stamp_tempo`'s own
  behaviour is unchanged: it still stamps nothing for a reading it cannot use.
- `specs/README.md`: this spec in the planned table.

No field enters the `ActiveSession` graph, so the crash-recovery blob is not
touched and `Store.sessionInProgressKey` stays where it is (#1345).

## Tests

Test-first, in this order:

- `domain/session.rs`: the existing unusable-reading table gains the notice
  assertion and keeps its "raises no error" one; a table over the four close
  paths, each raising the notice with the same text and leaving `error_seq`
  alone; a silent and a valid reading raising nothing; a skip raising nothing.
- `app.rs`: `view` carries `notice` and `notice_seq`; `ClearNotice` clears the
  notice and leaves a standing error; `ClearError` leaves a standing notice; an
  accepted event after a notice leaves `error_seq` where it was.
- `model.rs`: `raise_notice` bumps the sequence on identical text.
- `ios/IntradaTests/VariationPlayBridgeTests.swift`: a `LiveBridge` round
  trip that closes a play with a minim reading past the ceiling and reads the
  notice and the unmoved `errorSeq` back off the `ViewModel` (#846: a stub
  bridge cannot catch a dropped field).

## Out of scope

- The manual tempo sheet's range error and `record_rep`'s guards (finding 18
  names them too). Those are refusals of a value the musician typed or a tap
  that cannot count, and stay on `error`.
- Sorting the standing `raise_error` calls into refusals and notices. That is
  the refusal sweep the issue defers to its companion.
- Any change to what `stamp_tempo` accepts.
