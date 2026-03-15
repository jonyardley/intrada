# Research: iOS Design System Foundation

**Date**: 2026-03-15
**Feature**: #194

## Decision 1: Colour Token Implementation

**Decision**: Use `Color` static extensions with SwiftUI's `Color(red:green:blue:opacity:)` initialiser, converting oklch values from `input.css` to sRGB.

**Rationale**: SwiftUI's `Color` initialiser takes sRGB values directly. The oklch → sRGB conversion is done once at design time (not at runtime). Static `Color` extensions match the web's CSS custom property pattern and provide autocomplete.

**Alternatives considered**:
- Asset Catalog color sets: More Xcode-native but harder to version-control and review in PRs. Tokens are code, not resources.
- Style Dictionary (token generation tool): Overkill for a single platform with ~40 tokens. Manual mapping is clearer and more maintainable.

## Decision 2: Glassmorphism Approach

**Decision**: Use SwiftUI's `.ultraThinMaterial` as the primary glassmorphism effect, with a solid fallback colour matching the web's `--color-surface-fallback`.

**Rationale**: SwiftUI's Material system handles the blur natively and adapts to the system appearance. `.ultraThinMaterial` in dark mode produces the closest match to the web's `backdrop-filter: blur(12px)` on `white/12%` background. The web already has a fallback path for browsers without `backdrop-filter` — the iOS approach mirrors this.

**Alternatives considered**:
- Custom `UIVisualEffectView` wrapper: More control but unnecessary complexity. SwiftUI Materials are sufficient and well-optimised.
- Manual blur + overlay: Fragile, performance-heavy on older devices.

## Decision 3: Typography System

**Decision**: Use SwiftUI `Font` extensions with `.custom(_:size:relativeTo:)` for the heading serif font, and standard `.system()` fonts for body text. Create `ViewModifier` extensions for composite typography styles (e.g. `cardTitle`, `fieldLabel`).

**Rationale**: `.custom(_:size:relativeTo:)` automatically supports Dynamic Type scaling. The web uses Source Serif 4 for headings — iOS will use Georgia (the closest built-in serif) or bundle Source Serif 4 if file size permits. All body text uses the system font (San Francisco), matching the web's default sans-serif.

**Alternatives considered**:
- Bundle Source Serif 4: Would add ~200KB to the app bundle. Acceptable if exact matching is required. Decision: start with Georgia, evaluate visual match.
- UIFontMetrics: Lower-level API, unnecessary when `.custom(_:relativeTo:)` is available.

## Decision 4: Toast Implementation

**Decision**: Use an `@Observable` ToastManager injected via `.environment()`, with a `ViewModifier` that overlays the toast at the top of the screen with slide animation and auto-dismiss after 3 seconds.

**Rationale**: This matches the web's approach where toasts are managed by a central signal and rendered at the app level. Using SwiftUI's environment keeps it testable and mockable.

**Alternatives considered**:
- Third-party toast library: Adds a dependency for something simple. The component is ~50 lines.
- SwiftUI `.alert()`: Doesn't match the design language — uses native iOS chrome, not our glassmorphism style.

## Decision 5: Force Dark Mode

**Decision**: Apply `.preferredColorScheme(.dark)` at the app root (`WindowGroup`).

**Rationale**: The entire design language is built around a dark palette. Light mode would require a complete second set of tokens. The web app is dark-only. One line of code, zero ambiguity.

## Decision 6: Directory Structure

**Decision**: Layer the design system as:
```
DesignSystem/
  Tokens/
    Colors.swift      — Color extensions
    Typography.swift   — Font extensions + ViewModifiers
    Spacing.swift      — CGFloat constants
    Radius.swift       — Corner radius constants
  Modifiers/
    GlassCard.swift    — Glassmorphism modifier
    InputStyle.swift   — Form input modifier
Components/
  CardView.swift
  ButtonView.swift
  ...
```

**Rationale**: Mirrors the web's `input.css` (tokens) → `@utility` (modifiers) → `components/` (views) hierarchy. Keeps token definitions separate from component implementations. One file per concern.

## Colour Token Reference (oklch → sRGB)

Accurate conversions from the web's oklch values to SwiftUI Color(red:green:blue:) values:

| Token | oklch | sRGB (0-1) | Hex |
|-------|-------|------------|-----|
| textPrimary | oklch(100% 0 0) | 1.000, 1.000, 1.000 | #FFFFFF |
| textSecondary | oklch(86.9% 0.022 252.894) | 0.792, 0.836, 0.888 | #CAD5E2 |
| textLabel | oklch(92.9% 0.013 255.508) | 0.886, 0.910, 0.943 | #E2E8F0 |
| textMuted | oklch(70.4% 0.04 256.788) | 0.565, 0.632, 0.725 | #90A1B9 |
| textFaint | oklch(55.4% 0.046 257.417) | 0.384, 0.455, 0.557 | #62748E |
| accent | oklch(50.5% 0.24 274) | 0.271, 0.265, 0.915 | #4544E9 |
| accentHover | oklch(58% 0.22 274) | 0.345, 0.389, 0.976 | #5863F9 |
| accentText | oklch(78% 0.11 274) | 0.633, 0.699, 0.996 | #A1B2FE |
| accentFocus | oklch(66% 0.17 274) | 0.454, 0.525, 0.975 | #7486F9 |
| warmAccent | oklch(70% 0.12 78) | 0.781, 0.581, 0.236 | #C7943C |
| warmAccentText | oklch(84% 0.08 80) | 0.903, 0.774, 0.560 | #E6C58F |
| success | oklch(62% 0.16 158) | 0.000, 0.634, 0.374 | #00A25F |
| successText | oklch(79% 0.12 160) | 0.431, 0.825, 0.627 | #6ED2A0 |
| warning | oklch(74% 0.14 62) | 0.913, 0.583, 0.256 | #E99541 |
| warningText | oklch(83% 0.11 65) | 0.976, 0.723, 0.473 | #F9B879 |
| danger | oklch(63% 0.17 18) | 0.865, 0.331, 0.381 | #DD5561 |
| dangerHover | oklch(69% 0.15 18) | 0.914, 0.440, 0.469 | #E97078 |
| dangerText | oklch(77% 0.13 16) | 0.987, 0.565, 0.595 | #FC9098 |
| info | oklch(62% 0.14 238) | 0.000, 0.565, 0.815 | #0090D0 |
| infoText | oklch(79% 0.11 240) | 0.458, 0.768, 0.981 | #75C4FA |
| surfaceChrome | oklch(15.6% 0.011 261.692) | 0.038, 0.049, 0.067 @ 60% | #0A0C11 |
| surfaceFallback | oklch(25.7% 0.09 281.288) | 0.117, 0.103, 0.301 @ 80% | #1E1A4D |

Surface colours with alpha:
- surfacePrimary: white @ 12% opacity
- surfaceSecondary: white @ 5% opacity
- surfaceHover: white @ 22% opacity
- surfaceInput: white @ 10% opacity
- borderDefault: white @ 10% opacity
- borderCard: white @ 15% opacity
- borderInput: white @ 12% opacity
