# Quickstart: iOS Design System Foundation

**Date**: 2026-03-15
**Feature**: #194

## Prerequisites

- Xcode 16.2+
- `just ios` completed successfully (Rust library built, types generated)
- Physical iPhone or Simulator running iOS 17+

## Verification Steps

### 1. Build and Run

```bash
just ios
```

Open `ios/Intrada.xcodeproj` in Xcode, build and run on device or simulator.

### 2. Verify Dark Mode

- [ ] App always displays in dark mode regardless of system setting
- [ ] No light-mode flash on launch

### 3. Verify Design Tokens

Open Xcode Previews for `Colors.swift`:
- [ ] All colour swatches render without errors
- [ ] Colours are visually consistent with the web app

### 4. Verify Components

Open Xcode Previews for each component:
- [ ] `CardView` — glassmorphism effect visible, correct padding
- [ ] `ButtonView` — all variants render (Primary, Secondary, Danger)
- [ ] `TextFieldView` — label, hint, input, and error state all display
- [ ] `TextAreaView` — multi-line input with label and error
- [ ] `StatCardView` — title, value, subtitle layout correct
- [ ] `TypeBadge` — "Piece" and "Exercise" pills with distinct colours
- [ ] `PageHeading` — serif font, correct size
- [ ] `Toast` — all variants render with correct colours and icons
- [ ] `ErrorBanner` — danger-styled banner with dismiss
- [ ] `FormFieldError` — red error text below field
- [ ] `BackLink` — arrow icon with accent text
- [ ] `SkeletonLine` — pulsing animation
- [ ] `SkeletonBlock` — pulsing animation
- [ ] `EmptyStateView` — updated with design tokens

### 5. Verify Navigation

- [ ] Tab bar uses chrome/glassmorphism styling
- [ ] Selected tab highlighted with accent colour
- [ ] All four tabs accessible

### 6. Verify Dynamic Type

In Settings → Display & Brightness → Text Size:
- [ ] Set to largest size — components remain usable
- [ ] Set to smallest size — components render correctly
- [ ] Headings, body text, and labels scale proportionally

### 7. Cross-Platform Comparison

Compare side-by-side with web app at `localhost:8080/design`:
- [ ] Card backgrounds match
- [ ] Button colours match
- [ ] Text colours match hierarchy (primary > secondary > muted > faint)
- [ ] Overall "feel" is recognisably the same product
