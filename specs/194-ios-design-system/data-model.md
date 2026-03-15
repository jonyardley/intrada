# Data Model: iOS Design System Foundation

**Date**: 2026-03-15
**Feature**: #194

## Overview

This feature has no database entities or API contracts. The "data model" is the token and component type system that structures the design system.

## Token Types

### Colour Tokens

Organised into semantic groups, each as a static `Color` extension:

| Group | Tokens |
|-------|--------|
| **Text** | textPrimary, textSecondary, textLabel, textMuted, textFaint |
| **Accent** | accent, accentHover, accentText, accentFocus |
| **Warm Accent** | warmAccent, warmAccentHover, warmAccentText, warmAccentSurface |
| **Success** | success, successHover, successText, successSurface |
| **Warning** | warning, warningText, warningSurface |
| **Danger** | danger, dangerHover, dangerText, dangerSurface |
| **Info** | info, infoText, infoSurface |
| **Surface** | surfacePrimary, surfaceSecondary, surfaceChrome, surfaceFallback, surfaceHover, surfaceInput |
| **Border** | borderDefault, borderCard, borderInput |
| **Badge** | badgePieceBg, badgePieceText, badgeExerciseBg, badgeExerciseText |
| **Progress** | progressTrack, progressFill, progressComplete |

### Spacing Constants

| Token | Value (pt) | Use |
|-------|-----------|-----|
| cardCompact | 12 | Stat cards, compact elements |
| card | 16 | Standard card padding |
| cardComfortable | 24 | Comfortable card padding |
| section | 48 | Between page sections |
| sectionLarge | 64 | Major section breaks |

### Radius Constants

| Token | Value (pt) | Use |
|-------|-----------|-----|
| card | 16 | Card corners |
| button | 12 | Button corners |
| input | 12 | Input field corners |
| badge | 8 | Badge/pill corners |
| pill | .infinity | Fully rounded |

### Typography Styles

| Style | Font | Size | Weight | Colour |
|-------|------|------|--------|--------|
| pageHeading | Serif (Georgia) | 28pt | Bold | textPrimary |
| sectionTitle | System | 18pt | Semibold | textPrimary |
| cardTitle | System | 14pt | Semibold | textSecondary |
| fieldLabel | System | 12pt | Medium, uppercase | textMuted |
| formLabel | System | 14pt | Medium | textLabel |
| hintText | System | 12pt | Regular | textMuted |
| body | System | 16pt | Regular | textSecondary |

## Component Variants

### ButtonVariant
- `primary` — accent background, white text
- `secondary` — surfaceSecondary background, borderDefault border
- `danger` — danger background, white text
- `dangerOutline` — dangerSurface background, dangerText text

### ToastVariant
- `info` — info colour, circle-i icon
- `success` — success colour, checkmark icon
- `warning` — warning colour, triangle icon
- `danger` — danger colour, X icon

### CardPadding
- `compact` — 12pt
- `standard` — 16pt (default)
- `comfortable` — 24pt
