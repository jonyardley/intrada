# Quickstart: iOS Library

**Feature**: 001-ios-library | **Date**: 2026-03-16

## Prerequisites

1. Xcode 16+ installed
2. iOS Simulator available (iPhone 16 Pro + iPad Pro 13")
3. Rust toolchain with iOS targets: `just ios` must complete successfully
4. Clerk configured with valid test credentials
5. API server running (or using production `intrada-api.fly.dev`)

## Build & Run

```bash
# 1. Build Rust core for iOS simulator
just ios

# 2. Compile check (fast, ~30s)
just ios-swift-check

# 3. Smoke test on simulator (~15s after build)
just ios-smoke-test

# 4. Preview check
just ios-preview-check
```

## Verification Checklist

### V1: Library List (iPhone)
- [ ] Open app → Library tab is selected by default
- [ ] Items display in a scrollable list with title, composer, type badge, key, tempo, tags
- [ ] Tap "Pieces" tab → only pieces shown
- [ ] Tap "Exercises" tab → only exercises shown
- [ ] Tap "All" tab → all items shown
- [ ] Type in search → list filters by title/composer match
- [ ] Clear search → all items return
- [ ] With no items → empty state shown with "Add Item" prompt
- [ ] While loading → skeleton placeholders visible

### V2: Library List (iPad)
- [ ] Open app on iPad simulator → sidebar list + detail pane visible
- [ ] Select item in sidebar → detail appears in right pane
- [ ] Rotate to landscape → layout adapts (wider detail pane)
- [ ] Search and filter work in sidebar
- [ ] Empty selection → "Select an item" placeholder in detail pane

### V3: Item Detail
- [ ] Tap item → detail view shows all populated fields
- [ ] Title, composer, type badge, key, tempo displayed correctly
- [ ] Tags shown as pills
- [ ] Notes displayed with proper formatting
- [ ] Created/updated timestamps shown
- [ ] Item with practice history → summary card with session count, minutes, score
- [ ] Item without practice history → no practice section shown
- [ ] "..." toolbar menu → shows Edit and Delete options

### V4: Add Item
- [ ] Tap "+" or "Add Item" → add form appears
- [ ] Piece selected by default → composer field marked required
- [ ] Switch to Exercise → composer field becomes optional
- [ ] Submit with empty title → validation error on title field
- [ ] Submit piece with empty composer → validation error on composer field
- [ ] Fill all fields + tags → submit succeeds
- [ ] Success toast shown → returned to library list
- [ ] New item appears in list

### V5: Edit Item
- [ ] From detail "..." menu → tap Edit → edit form appears
- [ ] All fields pre-populated with current values
- [ ] Type tabs show current type (not editable)
- [ ] Change title → save → return to detail → title updated
- [ ] Clear notes → save → notes section hidden in detail
- [ ] Validation errors show inline

### V6: Delete Item
- [ ] From detail "..." menu → tap Delete
- [ ] Confirmation dialog appears with item title
- [ ] Cancel → nothing happens
- [ ] Confirm → item removed, returned to list, success toast

### V7: Cross-Platform Consistency
- [ ] Visual style matches web app (dark glassmorphism, indigo accents)
- [ ] Information hierarchy matches (same fields, same order)
- [ ] All operations available on web are available on iOS
- [ ] Colour tokens match between platforms

## Common Issues

| Issue | Fix |
|-------|-----|
| Build fails with missing types | Run `just typegen` to regenerate Swift types |
| Simulator shows blank screen | Run `just ios` to rebuild Rust core for simulator target |
| Auth fails on simulator | Check Clerk publishable key in `Config.swift` |
| List doesn't load | Verify API server is running and `Config.apiBaseURL` is correct |
| Toast not showing | Ensure `.toastOverlay()` is applied before `.environment()` in modifier chain |
