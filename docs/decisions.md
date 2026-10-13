# Decisions

Decisions whose full records were removed from the repo, one line each; git history holds every record.

## Product direction

- Native shells on the one Rust core replaced the Tauri webview in June 2026: an instrument side tool must feel native and work offline (#775).
- Coach pivot reversed on 2026-08-13 (#1344): session builder restored, coach machinery removed; it went too far, too fast, in an unwanted direction.
- API, account, sync service and MCP removed on 2026-09-12 (#1746): nothing on the roadmap needed a server; any return starts from scratch.
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
- A key is one typed value everywhere and keeps its spelling (#2246, #2106).
- Clearing a key crosses the bridge as a three-way KeyEdit, because Kotlin cannot tell `Option<Option>` apart (#2461).
- Session intention, time target and the three reflection boxes removed for good (#1766, #1374); their five columns stay, unread, in the session table.
- The core owns a saved session's stored form, including pre-play rows, so both phones read them the same way (#2234).
- Saving a session clears the summary and the crash copy only once the store confirms the write (#974).
- The profile is device data, never account data: name, instrument, icon and highlighter colour, saved as its own singleton (#1691).
- First run ticks are worked out from library and history, never stored; only the welcome's dismissal is saved (#2116).
- The rep target and click start are device defaults the core owns and applies; the shell only saves the blob (#1915).
- A preferred length of 10 to 120 minutes lives on the practice defaults; the builder can change it for today only (#1736).
- Library sort is a saved device setting the core applies; the default is newest first (#848).
- The core can send a notice as well as an error; notices show as a calm grey banner (#1325).
- Derived views are cached per change and the library crosses the bridge once per render (#1998).
- A list load that lands after a newer edit is dropped, so a reload never overwrites it (#2067).
- Exercises from a chord chart are a pure function of chart and key, linked the normal way; read the changes only, never melodies (#1107).
- Photo recognition is an effect; the model may choose but never invent, and fills a form the musician reviews (#1439, #1436, #1437).
- A rescan fills only empty fields or ones still holding an earlier read, never typed text (#2229).
- The click sounds only the beats the metre and chosen pattern say, so it never misstates the tempo (#1499).

## Shells and platform

- The Focus Player is native, with the core owning the clock and rep counting (#932).
- The click keeps going in the background through a silent audio session, and the timer reads the wall clock (#1399).
- Lock screen progress uses a Live Activity; the native design replaces the Tauri plugin design (#1600).
- TestFlight ships via fastlane match and an App Store Connect key, on dispatch or a v tag, never per PR, saving macOS minutes (#784, #984).

## Process

- Agent teams retired: one agent per vertical slice (#1223).
- Mutation-test by deleting a line, not inverting it (#1423).
- Rented macOS runners replaced by the self-hosted Mac (#1577, #1962).
- Harness changes are judged on speed, rework and spend; a lever unmoved after two reads is reverted (#1897, #1898).
- Components: one spec per component in the repo, pictures from snapshot tests, a check in just check; the design page is a view (#2603).
- August 2026 audit of the restored builder: fix and trim first, then the tone of voice doc and one copy sweep (#1344, #1359).
- September 2026 audit: silent data loss first, then wrong on screen, then drag (#1967).
