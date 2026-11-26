# Decisions

Decisions whose full records were removed from the repo, one line each; git history holds every record.

## Product direction

- Native shells on the one Rust core replaced the Tauri webview in June 2026: an instrument side tool must feel native and work offline (#775).
- The Leptos web shell and Tauri iOS host were removed in July 2026, leaving the two native shells (PR #1133).
- Coach pivot reversed on 2026-08-13 (#1344): session builder restored, coach machinery removed; it went too far, too fast, in an unwanted direction.
- API, account, sync service and MCP removed on 2026-09-12 (#1746): nothing on the roadmap needed a server; any return starts from scratch.
- The paused Android app (#2860) was removed on 2026-11-20 (#2924): every feature ships on iPhone only; its last state is on the tag `android-paused-2026-11-20`.
- July 2026: the ten-step notebook journey stopped being the yardstick; the roadmap now ranks the work (#1146, #1344).
- August 2026: quick wins first (metronome, tempo, Up next, getting cold), then capture through one pass create (#1390).
- No lesson entity: adding a piece with its chart and exercises in one save is the lesson capture answer (#1390, #1080).
- No goals entity: direction is a star on items, and Goals were removed rather than migrated (#739).
- Machine listening deferred: an attempt is defined by its target phrase; timing alone cannot tell a resume from a restart (#1161, #1176).
- Getting cold is a graded estimate, kept quiet: a signal that cries wolf gets ignored (#1416, #1419).
- Staleness weights time since practice by how well a piece is learned, not a flat 14 days (#1416).
- The suggestion is the Practice hero; reasons are strings the core writes; dismissing it lasts the app session (#1082).
- Today's plan fills the preferred length and starts in one tap, skipping the builder (#57, #999).
- An exercise shows one "Used in" list of pieces, not a single related breadcrumb (#1363).
- On a piece, writing your own exercise comes before chart-generated ones (#1431).
- Related exercises: "related" over "linked"; the 0 to 10 ring replaced bars and dots (#1020).
- May 2026: a light paper direction explored in Pencil; the 8f look (T24, #1723) replaced it (#771).

## Core and data

- A session records each play with its variation and tempo, not one score per item (#1739); old history is read, not migrated.
- Store events and the musician's own answers, never a verdict the app works out; verdicts are computed when read (#50).
- How a section is going is worked out from plays when read, never stored on it (#2245, #2321).
- Links live on the piece, one row per exercise and section, reconciled so relinking revives the old row (#2248).
- Sections and trouble spots removed on 2026-11-08, with their saved data, to simplify the app while the basics are worked on; a piece is one whole thing again, and a way back is parked in #2768 (#2767).
- A key is one typed value everywhere and keeps its spelling (#2246, #2106).
- Clearing a key crosses the bridge as a three-way KeyEdit, because Kotlin cannot tell `Option<Option>` apart (#2461).
- Session intention, time target and the three reflection boxes removed for good (#1766, #1374); their five columns stay, unread, in the session table.
- The core owns a saved session's stored form, including pre-play rows, so both phones read them the same way (#2234).
- Saving a session clears the summary and the crash copy only once the store confirms the write (#974).
- The profile is device data, never account data: name, instrument, icon and highlighter colour, saved as its own singleton (#1691).
- First run ticks are worked out from library and history, never stored; only the welcome's dismissal is saved (#2116).
- The click start is a device default the core owns and applies; the shell only saves the blob (#1915). The repetition target it once held went with #2791.
- A preferred length of 10 to 120 minutes lives on the practice defaults; the builder can change it for today only (#1736).
- Library sort is a saved device setting the core applies; the default is newest first (#848).
- The core can send a notice as well as an error; notices show as a calm grey banner (#1325).
- Derived views are cached per change and the library crosses the bridge once per render (#1998).
- A list load that lands after a newer edit is dropped, so a reload never overwrites it (#2067).
- Exercises from a chord chart are a pure function of chart and key, linked the normal way; read the changes only, never melodies (#1107).
- Reading a piece's details from a photo was removed: it failed too often to be trusted. Score photos stay (#2792).
- The click sounds only the beats the metre and chosen pattern say, so it never misstates the tempo (#1499).

## Shells and platform

- The Focus Player is native, with the core owning the clock (#932).
- The click keeps going in the background through a silent audio session, and the timer reads the wall clock (#1399).
- Lock screen progress uses a Live Activity; the native design replaces the Tauri plugin design (#1600).
- Accessibility text sizes dropped on 2026-11-26: text caps at the largest standard size, because layouts did not hold beyond it and the owner chose not to support them (#1411, closes epic #2134).
- TestFlight ships via fastlane match and an App Store Connect key, on dispatch or a v tag, never per PR, saving macOS minutes (#784, #984).

## Process

- Agent teams retired: one agent per vertical slice (#1239).
- Mutation-test by deleting a line, not inverting it (#1423), and count the
  changed line with `grep -c` before the run: a pattern that matched nothing
  leaves the tests green, which reads as a mutant they missed (#2962). Run
  the unbroken copy first and expect green: a copy that cannot run fails
  every case, which reads as a mutant caught (#2967).
- Rented macOS runners replaced by the self-hosted Mac (#1577, #1962).
- Harness changes were judged on speed, rework and spend, with a revert after two unmoved reads (#1897, #1898); the reads and the revert rule were dropped on 2026-11-24 (#2959), since no read was ever recorded.
- Components: one spec per component in the repo, pictures from snapshot tests, a check in just check; the design page is a view (#2603).
- August 2026 audit of the restored builder: fix and trim first, then the tone of voice doc and one copy sweep (#1344, #1359).
- September 2026 audit: silent data loss first, then wrong on screen, then drag (#1967).
- Agents take a ready issue to a draft PR; sign-off on plan, mockup, screenshots and merge stays with a person (#2799).
- Pencil (pen.dev) is trialled beside Claude Design on the next new screen, then one is chosen; the trial moved to the design tooling epic on 2026-11-12 (#2804, #2698).
- The agent crew builds on this repo's own harness: claim, leased worktrees, task, reviewer and test agents, guards (#2799).
- Claude models only. The crew loop is a script that starts one Claude Code background session per issue, once it passes seven checks; the one-evening bake-off was dropped on 2026-11-12, since one run per candidate could not separate them (#2799, #2802).
- The crew loop is started by hand with `just crew` and runs one issue at a time, capped at 3 hours a session, 45 minutes of silence, 3 issues a run and $150 of spend over 24 hours (#2803, #2821).
- The crew acts as its own agent account (ostinato#11), which GitHub refuses a merge into main, and the crew only works on an issue whose `ready` label the owner added (#2811). Agents merge nothing, whatever the risk (#2799).
- From 2026-11-14, model and effort for every spawn, and the lead's raise for sensitive surfaces, come from the role table in the owner's global `~/.claude/CLAUDE.md`; agent definitions lost their `model:`, `effort:` and `effort-ceiling:` pins, and `guard-spawn.sh` now prompts rather than refuses a spawn with no model named, on Fable or above medium effort; the table's Sonnet rows supersede #1893's Opus floor for judgement (#1893, #2044, #2866).
- From 2026-11-20, a session stays on the level it opens on, sensitive surfaces included, and never asks the owner to type `/effort`; this supersedes the lead's raise for sensitive surfaces in the 2026-11-14 line (#2929).
- From 2026-11-25, the `ready` label is retired: the owner's approval of the plan comment is the go, and `just status` ranks every open issue; this supersedes the label in the crew account line (#2811) (#2800, #3016).
- Agent knowledge stays in the repo and on GitHub: hand-offs as issue comments, one pruned lessons file, no wiki or memory database (#2799).
- On 2026-11-19 the crew loop and Ostinato were removed from intrada, so the owner can try them on a new project; sessions are owner-driven again (#2909).
