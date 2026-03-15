# Implementation Plan: iOS Design System Foundation

**Branch**: `194-ios-design-system` | **Date**: 2026-03-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/194-ios-design-system/spec.md`

## Summary

Build the iOS design system foundation: colour/typography/spacing tokens that match the web's `input.css`, reusable SwiftUI components mirroring the web component library, polished navigation chrome, and Xcode Preview catalogue. This enables all future iOS feature views to be composed from a consistent, token-based component library.

## Technical Context

**Language/Version**: Swift 6.0, iOS 17.0+
**Primary Dependencies**: SwiftUI, ClerkKit (existing)
**Storage**: N/A — no new persistence
**Testing**: Xcode Previews (visual verification), manual device testing
**Target Platform**: iOS 17.0+ (iPhone)
**Project Type**: Mobile (iOS shell within Crux architecture)
**Performance Goals**: 60fps, instant component rendering
**Constraints**: Dark mode only, must match web design language exactly
**Scale/Scope**: ~40 colour tokens, ~13 components, ~4 typography styles

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | One file per token group, one file per component. No dead code. |
| II. Testing Standards | ✅ Pass | Every component has `#Preview`. Visual verification via Xcode Previews. |
| III. UX Consistency | ✅ Pass | This IS the consistency feature — tokens match web 1:1. |
| IV. Performance | ✅ Pass | Static Color extensions, native Material blur. No runtime overhead. |
| V. Architecture Integrity | ✅ Pass | Pure UI layer — no core changes. Shell-only. |
| VI. Inclusive Design | ✅ Pass | Dynamic Type support, no auto-play, consistent navigation. |

## Project Structure

### Documentation (this feature)

```text
specs/194-ios-design-system/
├── spec.md
├── plan.md              # This file
├── research.md          # Colour conversions, SwiftUI patterns
├── data-model.md        # Token types, component variants
└── quickstart.md        # Verification steps
```

### Source Code (repository root)

```text
ios/Intrada/
├── IntradaApp.swift                    # Modified: add .preferredColorScheme(.dark)
├── DesignSystem/
│   ├── Tokens/
│   │   ├── Colors.swift                # NEW: Color extensions (all tokens)
│   │   ├── Typography.swift            # NEW: Font extensions + ViewModifiers
│   │   ├── Spacing.swift               # NEW: CGFloat spacing constants
│   │   └── Radius.swift                # NEW: Corner radius constants
│   └── Modifiers/
│       ├── GlassCard.swift             # NEW: Glassmorphism ViewModifier
│       └── InputStyle.swift            # NEW: Form input ViewModifier
├── Components/
│   ├── EmptyStateView.swift            # Modified: use design tokens
│   ├── CardView.swift                  # NEW
│   ├── ButtonView.swift                # NEW
│   ├── TextFieldView.swift             # NEW
│   ├── TextAreaView.swift              # NEW
│   ├── StatCardView.swift              # NEW
│   ├── TypeBadge.swift                 # NEW
│   ├── PageHeading.swift               # NEW
│   ├── Toast.swift                     # NEW
│   ├── ToastManager.swift              # NEW
│   ├── ErrorBanner.swift               # NEW
│   ├── FormFieldError.swift            # NEW
│   ├── BackLink.swift                  # NEW
│   ├── SkeletonLine.swift              # NEW
│   └── SkeletonBlock.swift             # NEW
├── Navigation/
│   └── MainTabView.swift               # Modified: chrome styling, token colours
└── Views/
    └── (empty — ready for feature views)
```

**Structure Decision**: Flat `Components/` directory (matching web's `components/`) with dedicated `DesignSystem/` for tokens and modifiers. One file per component, one file per token group.

## Implementation Phases

### Phase 1: Tokens & Modifiers (Foundation)

**Goal**: All design tokens defined and usable. No visual changes yet.

**Files**:
1. `DesignSystem/Tokens/Colors.swift` — All Color extensions from research.md conversion table
2. `DesignSystem/Tokens/Typography.swift` — Font extensions (heading serif, body, caption) + ViewModifiers (cardTitle, sectionTitle, fieldLabel, formLabel, hintText)
3. `DesignSystem/Tokens/Spacing.swift` — CGFloat constants for card padding, section spacing
4. `DesignSystem/Tokens/Radius.swift` — CGFloat constants for corners
5. `DesignSystem/Modifiers/GlassCard.swift` — `.glassCard()` ViewModifier (Material + border + radius + shadow)
6. `DesignSystem/Modifiers/InputStyle.swift` — `.inputStyle()` ViewModifier (background, border, focus state)
7. `IntradaApp.swift` — Add `.preferredColorScheme(.dark)` to WindowGroup

**Validation**: All token files compile. Previews render colour swatches.

### Phase 2: Core Components

**Goal**: The 8 most-used components built and previewable.

**Files**:
1. `Components/CardView.swift` — Uses `.glassCard()` modifier, configurable padding
2. `Components/ButtonView.swift` — Primary/Secondary/Danger variants, loading + disabled states, 44pt min height
3. `Components/TextFieldView.swift` — Label + hint + input + FormFieldError, `.inputStyle()` modifier
4. `Components/TextAreaView.swift` — Multi-line version of TextFieldView
5. `Components/StatCardView.swift` — Title (fieldLabel) + value (large bold) + subtitle
6. `Components/TypeBadge.swift` — Piece/Exercise pill with badge tokens
7. `Components/PageHeading.swift` — Serif heading font
8. `Components/FormFieldError.swift` — Inline danger text

**Validation**: Every component has a `#Preview` block. Components use only token colours.

### Phase 3: Feedback & Loading Components

**Goal**: Toast, error banner, skeleton components.

**Files**:
1. `Components/ToastManager.swift` — `@Observable` class with show/dismiss, auto-hide timer
2. `Components/Toast.swift` — Info/Success/Warning/Danger variants with icon + message
3. `Components/ErrorBanner.swift` — Persistent error display with dismiss
4. `Components/SkeletonLine.swift` — Pulsing text placeholder
5. `Components/SkeletonBlock.swift` — Pulsing rectangular placeholder
6. `Components/BackLink.swift` — Arrow + label navigation link

**Validation**: Toast auto-dismisses after 3 seconds. Skeletons animate. All previews render.

### Phase 4: Navigation Polish & Existing Component Updates

**Goal**: Tab bar matches design language. EmptyStateView uses tokens.

**Files**:
1. `Navigation/MainTabView.swift` — Replace `.indigo` with `.accent`, style tab bar chrome
2. `Components/EmptyStateView.swift` — Replace `.tertiary`/`.secondary`/`.indigo` with token colours
3. `IntradaApp.swift` — Inject `ToastManager` into environment
4. Sign-in view in `IntradaApp.swift` — Replace raw colours with tokens

**Validation**: Tab bar visually matches web's `BottomTabBar`. EmptyStateView uses only tokens. Sign-in screen uses design tokens.

## Post-Phase 1 Constitution Re-Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ | One file per concern. No duplication. |
| II. Testing Standards | ✅ | Every component has `#Preview`. |
| III. UX Consistency | ✅ | All tokens match web. Components mirror web equivalents. |
| IV. Performance | ✅ | Static extensions, native blur. Zero runtime overhead. |
| V. Architecture Integrity | ✅ | Shell-only changes. Core untouched. |
| VI. Inclusive Design | ✅ | Dynamic Type via `.custom(_:relativeTo:)`. No auto-play. Dark mode forced. |

## Complexity Tracking

No constitution violations. All work is within the existing iOS shell, adding no new dependencies or architectural patterns.
