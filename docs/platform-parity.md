# Platform Parity Analysis

*Last reviewed: 2026-04-09*

Feature-by-feature comparison of web (Leptos) and iOS (SwiftUI) shells.

---

## Web features missing from iOS

| Feature | Notes |
|---------|-------|
| Week strip navigator | iOS shows sessions grouped by date but lacks the week strip with day dots and prev/next week navigation |

## iOS features missing from web

| Feature | Notes |
|---------|-------|
| Library search/filter | iOS has search bar + type tabs (All/Pieces/Exercises). Web shows full list with no filtering |
| Between-item scoring (transition sheet) | iOS prompts for score/tempo/notes between items during practice. Web only does this in the summary |
| Abandon session | iOS has this in the pause overlay. Web only has "end early" |
| Session intention display | iOS shows the intention during active practice and in the summary. Web doesn't surface it |
| Score trend dots | iOS analytics has a visual dot visualization for score history per item |
| Sign out | iOS has it in the tab bar account menu. Web doesn't expose it |

## Core events neither shell exposes

| Event | Description |
|-------|-------------|
| `AddItemMidSession` / `AddNewItemMidSession` | Add items during an active session |
| `AddNewItemToSetlist` | Create a new item inline during building |

## Different approaches (not gaps)

| Area | Web | iOS |
|------|-----|-----|
| Tempo history | Line chart | History list |
| 28-day chart | Line chart | Bar chart |
| Session history nav | Week strip | Date-grouped list |
| Delete confirmation | Inline banner | `.confirmationDialog` |
| Session detail | Expanded card inline | Dedicated view |
