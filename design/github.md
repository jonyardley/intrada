repo: jonyardley/intrada
branch: main
path: design/

## Last sync
date: 2026-08-04T07:00:03Z

### Updated in this project
- Rebuilt `Drill Loop.dc.html` A2/A3 around spec v7 / decision 18 (machine listening deferred): A3 now uses the tap-verdict pattern (wide primary "Yes — clean" / ghost secondary "No — missed it") instead of the old pass/miss/unsure/gate-open machine-scored states.
- Dropped the "unsure — app isn't sure" A3 variant entirely (no machine confidence exists to be uncertain about).
- Added the "no microphone yet" dashed note to A2 (phone default, AX5, iPad).
- Resolved the RepCounter/GateDots collision: GateDots is now user-tapped (fills on tap-verdict), same mechanism as RepCounter — no more collision. Gate counting is cumulative, not consecutive, per decision 17's self-correction argument.

## Screen map
| Project screen | Repo source |
|---|---|
| Intrada Design System.dc.html | design/intrada-design-system.dc.html, ios/Intrada/DesignSystem/Theme.swift |
