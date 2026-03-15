# Tasks: iOS Design System Foundation

**Input**: Design documents from `specs/194-ios-design-system/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Not requested — visual verification via Xcode Previews.

**Organization**: Tasks grouped by user story. US1 and US2 are both P1 (co-dependent: tokens + components). US3-US5 follow.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Directory structure and dark mode enforcement

- [x] T001 Create `ios/Intrada/DesignSystem/Tokens/` and `ios/Intrada/DesignSystem/Modifiers/` directories (already done — verify they exist)
- [x] T002 Force dark mode by adding `.preferredColorScheme(.dark)` to `WindowGroup` in `ios/Intrada/IntradaApp.swift`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: All design tokens MUST be complete before any component can be built

**⚠️ CRITICAL**: No component work can begin until this phase is complete

- [x] T003 [P] Create colour token extensions in `ios/Intrada/DesignSystem/Tokens/Colors.swift` — define all Color extensions from research.md conversion table: text tokens (textPrimary, textSecondary, textLabel, textMuted, textFaint), accent tokens (accent, accentHover, accentText, accentFocus), warm accent tokens (warmAccent, warmAccentHover, warmAccentText, warmAccentSurface), status tokens (success/successHover/successText/successSurface, warning/warningText/warningSurface, danger/dangerHover/dangerText/dangerSurface, info/infoText/infoSurface), surface tokens (surfacePrimary, surfaceSecondary, surfaceChrome, surfaceFallback, surfaceHover, surfaceInput), border tokens (borderDefault, borderCard, borderInput), badge tokens (badgePieceBg, badgePieceText, badgeExerciseBg, badgeExerciseText), progress tokens (progressTrack, progressFill, progressComplete). Add `#Preview` with colour swatches.
- [x] T004 [P] Create typography extensions in `ios/Intrada/DesignSystem/Tokens/Typography.swift` — define Font extensions: `.heading` (Georgia serif, 28pt, relativeTo .largeTitle), `.sectionTitle` (system 18pt semibold), `.cardTitle` (system 14pt semibold), `.fieldLabel` (system 12pt medium). Create ViewModifiers: `SectionTitleStyle`, `CardTitleStyle`, `FieldLabelStyle` (uppercase + tracking), `FormLabelStyle`, `HintTextStyle`. Add `#Preview` showcasing all styles.
- [x] T005 [P] Create spacing constants in `ios/Intrada/DesignSystem/Tokens/Spacing.swift` — define `enum Spacing` with static CGFloat values: cardCompact (12), card (16), cardComfortable (24), section (48), sectionLarge (64). Add `#Preview`.
- [x] T006 [P] Create radius constants in `ios/Intrada/DesignSystem/Tokens/Radius.swift` — define `enum DesignRadius` with static CGFloat values: card (16), button (12), input (12), badge (8). Add `#Preview`.
- [x] T007 Create glassmorphism ViewModifier in `ios/Intrada/DesignSystem/Modifiers/GlassCard.swift` — implement `.glassCard(padding:)` modifier using `.ultraThinMaterial` background, `Color.borderCard` border (1px), `DesignRadius.card` corners, subtle shadow. Support padding parameter (compact/standard/comfortable, default standard). Add `.glassCardActive()` variant with accent border + glow. Add `#Preview`.
- [x] T008 Create form input ViewModifier in `ios/Intrada/DesignSystem/Modifiers/InputStyle.swift` — implement `.inputStyle(hasError:)` modifier with `Color.surfaceInput` background, `Color.borderInput` border, `DesignRadius.input` corners, focus state with `Color.accentFocus` border. Error state uses `Color.dangerText` border. Add `#Preview`.

**Checkpoint**: All tokens and modifiers defined. Components can now be built.

---

## Phase 3: User Story 1 — Visual Consistency (Priority: P1) 🎯 MVP

**Goal**: All design tokens exist and are visually verified against web app

**Independent Test**: Compare iOS colour swatch preview with web design catalogue — all tokens should match

### Implementation for User Story 1

- [x] T009 [US1] Verify colour accuracy by running the app and comparing token swatches side-by-side with web app at `/design`. Adjust any Color values in `ios/Intrada/DesignSystem/Tokens/Colors.swift` that don't match visually.
- [x] T010 [US1] Verify typography by comparing heading font (serif) and body text hierarchy with web app. Adjust font sizes/weights in `ios/Intrada/DesignSystem/Tokens/Typography.swift` if needed.

**Checkpoint**: Tokens visually match the web app.

---

## Phase 4: User Story 2 — Reusable Component Library (Priority: P1)

**Goal**: 13 reusable components built, each with `#Preview`, all using only design tokens

**Independent Test**: Every component renders correctly in Xcode Previews without errors

### Implementation for User Story 2

- [x] T011 [P] [US2] Create `CardView` in `ios/Intrada/Components/CardView.swift` — SwiftUI view wrapping content in `.glassCard()` modifier. Accept `children: @ViewBuilder` content and optional `padding: Spacing` parameter. Add `#Preview` showing card with sample text content.
- [x] T012 [P] [US2] Create `FormFieldError` in `ios/Intrada/Components/FormFieldError.swift` — conditional text view showing validation error in `Color.dangerText`, font `.caption`. Accept `message: String?`. Only renders when message is non-nil. Add `#Preview` with and without error.
- [x] T013 [P] [US2] Create `PageHeading` in `ios/Intrada/Components/PageHeading.swift` — serif heading text using `Font.heading`. Accept `text: String` and optional `subtitle: String?`. Subtitle uses `.textSecondary`, `.subheadline`. Add `#Preview`.
- [x] T014 [P] [US2] Create `TypeBadge` in `ios/Intrada/Components/TypeBadge.swift` — pill-shaped badge for "Piece" or "Exercise". Accept `itemType: String`. Use `Color.badgePieceBg`/`Color.badgePieceText` for pieces, `Color.badgeExerciseBg`/`Color.badgeExerciseText` for exercises. `DesignRadius.badge` corners. Add `#Preview` showing both types.
- [x] T015 [P] [US2] Create `StatCardView` in `ios/Intrada/Components/StatCardView.swift` — metric display with title (fieldLabel style, uppercase), large value (`.title2.bold()`, textPrimary), optional subtitle (`.caption`, textMuted). Wrapped in `.glassCard(padding: .cardCompact)`. Add `#Preview`.
- [x] T016 [US2] Create `ButtonView` in `ios/Intrada/Components/ButtonView.swift` — tappable button with variants: `.primary` (accent bg, white text), `.secondary` (surfaceSecondary bg, borderDefault border), `.danger` (danger bg, white text), `.dangerOutline` (dangerSurface bg, dangerText text). Accept `variant: ButtonVariant`, `action: () -> Void`, `disabled: Bool`, `loading: Bool` (shows ProgressView). Min height 44pt. `DesignRadius.button` corners. Add `#Preview` showing all variants.
- [x] T017 [US2] Create `TextFieldView` in `ios/Intrada/Components/TextFieldView.swift` — form input with: label (formLabel style), optional hint (hintText style), SwiftUI `TextField` with `.inputStyle()` modifier, `FormFieldError` below. Accept `label: String`, `text: Binding<String>`, `hint: String?`, `error: String?`, `placeholder: String`. Add `#Preview` with states: empty, filled, error, with hint.
- [x] T018 [US2] Create `TextAreaView` in `ios/Intrada/Components/TextAreaView.swift` — multi-line version of TextFieldView using SwiftUI `TextEditor`. Same props as TextFieldView plus optional `minHeight: CGFloat` (default 100). Apply `.inputStyle()` modifier. Add `#Preview`.
- [x] T019 [P] [US2] Create `BackLink` in `ios/Intrada/Components/BackLink.swift` — navigation link with left arrow SF Symbol and accent-coloured label. Accept `label: String` and `action: () -> Void`. Uses `Color.accentText`. Add `#Preview`.

**Checkpoint**: All core components built. Developer can compose views from the component library.

---

## Phase 5: User Story 3 — Polished Navigation Shell (Priority: P2)

**Goal**: Tab bar matches design language, sign-in screen uses tokens

**Independent Test**: Navigate between tabs — tab bar uses chrome styling, selected tab highlighted with accent

### Implementation for User Story 3

- [x] T020 [US3] Update tab bar styling in `ios/Intrada/Navigation/MainTabView.swift` — replace `.tint(.indigo)` with `.tint(Color.accent)`. Style tab bar appearance using `UITabBarAppearance` with `Color.surfaceChrome` background. Replace `.foregroundStyle(.indigo)` on account icon with `.foregroundStyle(Color.accentText)`.
- [x] T021 [US3] Update sign-in view in `ios/Intrada/IntradaApp.swift` — replace `Color.black` with dark background, `.indigo` with `Color.accent`, `.white` with `Color.textPrimary`, `.secondary` with `Color.textSecondary`, `.red` with `Color.dangerText`. Apply glassmorphism to sign-in card if appropriate.
- [x] T022 [US3] Update placeholder views in `ios/Intrada/Navigation/MainTabView.swift` — replace `Color(.systemBackground)` with dark background, `.tertiary` with `Color.textFaint`, `.secondary` with `Color.textSecondary`.

**Checkpoint**: Navigation shell looks consistent with web app.

---

## Phase 6: User Story 4 — Loading & Empty States (Priority: P2)

**Goal**: Skeleton loading placeholders and updated empty state component

**Independent Test**: View skeleton previews — pulsing animation visible with correct surface colour

### Implementation for User Story 4

- [x] T023 [P] [US4] Create `SkeletonLine` in `ios/Intrada/Components/SkeletonLine.swift` — pulsing text-width rectangle using `Color.surfaceSecondary` with SwiftUI `.opacity()` animation (pulse between 0.3 and 1.0). Accept optional `width: CGFloat?` and `height: CGFloat` (default 16). Add `#Preview`.
- [x] T024 [P] [US4] Create `SkeletonBlock` in `ios/Intrada/Components/SkeletonBlock.swift` — pulsing rectangular block for card placeholders. Accept `height: CGFloat` (default 96). Same animation as SkeletonLine. `DesignRadius.card` corners. Add `#Preview`.
- [x] T025 [US4] Update `EmptyStateView` in `ios/Intrada/Components/EmptyStateView.swift` — replace `.tertiary` with `Color.textFaint`, `.secondary` with `Color.textSecondary`, `.indigo` with `Color.accent`, `.borderedProminent` with `ButtonView(.primary)` or token-based styling. Add `#Preview`.

**Checkpoint**: Loading and empty states match design language.

---

## Phase 7: User Story 5 — Error & Feedback Display (Priority: P3)

**Goal**: Toast and error banner components with status colours

**Independent Test**: Trigger toast in preview — auto-dismisses after 3 seconds with correct colour variant

### Implementation for User Story 5

- [x] T026 [US5] Create `ToastManager` in `ios/Intrada/Components/ToastManager.swift` — `@Observable @MainActor` class with `message: String`, `variant: ToastVariant`, `isShowing: Bool`. `show(_:variant:)` method sets values and starts 3-second auto-dismiss timer via `Task.sleep`. `ToastVariant` enum: `.info`, `.success`, `.warning`, `.danger`.
- [x] T027 [US5] Create `Toast` in `ios/Intrada/Components/Toast.swift` — overlay view reading from `ToastManager` environment. Show SF Symbol icon + message text in rounded container with left border. Use status colour tokens per variant (e.g. `.success` → Color.success border, Color.successSurface bg, Color.successText text). Slide-in animation from top. Add `.toastOverlay()` ViewModifier for easy attachment. Add `#Preview` showing each variant.
- [x] T028 [US5] Create `ErrorBanner` in `ios/Intrada/Components/ErrorBanner.swift` — persistent banner with `Color.dangerSurface` background, `Color.dangerText` border accent, error message in `Color.dangerText`. Accept `message: String` and `onDismiss: (() -> Void)?`. Add `#Preview`.
- [x] T029 [US5] Inject `ToastManager` into environment in `ios/Intrada/IntradaApp.swift` — create `@State private var toastManager = ToastManager()` and add `.environment(toastManager)` to ContentRouter. Add `.toastOverlay()` modifier to the root view.

**Checkpoint**: Feedback system complete. All 13+ components built.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and verification

- [x] T030 [P] Remove empty `ios/Intrada/Auth/` directory if still present
- [x] T031 [P] Update `CLAUDE.md` iOS component table with final list of all implemented components
- [x] T032 Run full quickstart.md verification — build app, check all previews, compare with web, test Dynamic Type at 3 sizes
- [x] T033 Regenerate Xcode project with `cd ios && xcodegen generate` to pick up all new files

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all components
- **US1 (Phase 3)**: Depends on Phase 2 — visual verification of tokens
- **US2 (Phase 4)**: Depends on Phase 2 — can run in parallel with US1
- **US3 (Phase 5)**: Depends on Phase 2 — can run in parallel with US1/US2
- **US4 (Phase 6)**: Depends on Phase 2 — can run in parallel with US1/US2/US3
- **US5 (Phase 7)**: Depends on Phase 2 — can run in parallel with others
- **Polish (Phase 8)**: Depends on all user stories being complete

### Within Phase 2 (Foundational)

- T003, T004, T005, T006 can all run in parallel (different files)
- T007 depends on T003 (uses Color tokens) and T005 (uses Spacing) and T006 (uses Radius)
- T008 depends on T003 (uses Color tokens) and T006 (uses Radius)

### Within Phase 4 (US2 — Components)

- T011, T012, T013, T014, T015, T019 can all run in parallel (independent components)
- T016 (ButtonView) is independent
- T017 (TextFieldView) depends on T012 (FormFieldError) and T008 (InputStyle)
- T018 (TextAreaView) depends on T017 pattern (similar structure)

### Parallel Opportunities

```bash
# Phase 2 — all token files in parallel:
T003: Colors.swift
T004: Typography.swift
T005: Spacing.swift
T006: Radius.swift

# Phase 4 — independent components in parallel:
T011: CardView.swift
T012: FormFieldError.swift
T013: PageHeading.swift
T014: TypeBadge.swift
T015: StatCardView.swift
T019: BackLink.swift

# Phase 6 — skeleton components in parallel:
T023: SkeletonLine.swift
T024: SkeletonBlock.swift
```

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational tokens (T003-T008)
3. Complete Phase 3: Verify tokens match web (T009-T010)
4. Complete Phase 4: Build all core components (T011-T019)
5. **STOP and VALIDATE**: Every component renders in Xcode Previews
6. Commit and review

### Incremental Delivery

1. Tokens + Core Components → **MVP** (US1 + US2)
2. Navigation polish → Tab bar matches web (US3)
3. Loading states → Skeletons + EmptyState (US4)
4. Feedback system → Toast + ErrorBanner (US5)
5. Each phase adds visual polish without breaking previous work

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Every component MUST have a `#Preview` block
- Every colour MUST use a named token — no raw `.white`, `.indigo`, `.gray`
- Commit after each phase checkpoint
- Reference research.md for accurate sRGB colour values
- Reference data-model.md for component variant definitions
