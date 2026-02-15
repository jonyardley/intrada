<!--
================================================================================
SYNC IMPACT REPORT
================================================================================
Version change: 1.0.0 → 2.0.0 (MAJOR - Tailored to actual project architecture)

Modified principles:
- I. Code Quality: Minor wording tweaks (no substantive change)
- II. Testing Standards: Replaced generic "Contract Tests" with Crux-specific "Boundary Tests" reflecting actual core↔shell architecture
- III. User Experience Consistency: Removed "Progressive Enhancement" (impossible for WASM app), replaced "Design System Adherence" with "Visual Consistency" (no formal design system exists), reframed "Accessibility" as aspirational target
- IV. Performance Requirements: Complete rewrite — removed API/server assumptions, replaced with client-side WASM performance principles (bundle size, render performance, localStorage efficiency)

Modified sections:
- Technical Decision Governance: Simplified compliance verification for solo developer
- Amendment Process: Simplified for solo developer (removed team consensus, 48-hour comment period)

Added sections:
- V. Architecture Integrity (new — codifies the Crux pure-core pattern)

Removed sections: None (all four original principles retained, one added)

Templates requiring updates:
- .specify/templates/plan-template.md: ✅ No updates needed (Constitution Check section is generic)
- .specify/templates/spec-template.md: ✅ No updates needed
- .specify/templates/tasks-template.md: ✅ No updates needed
- .specify/templates/checklist-template.md: ✅ No updates needed
- .specify/templates/agent-file-template.md: ✅ No updates needed

Follow-up TODOs: None
================================================================================
-->

# Intrada Constitution

## Core Principles

### I. Code Quality

All code MUST adhere to established quality standards that ensure maintainability, readability, and reliability.

**Non-Negotiable Rules**:

- **Clarity over cleverness**: Code MUST be readable and self-documenting; complex logic requires explanatory comments
- **Single Responsibility**: Each module and function MUST have one clear purpose
- **Consistent Style**: All code MUST follow the project's established linting and formatting rules (`cargo clippy -- -D warnings`, `cargo fmt`); no exceptions without documented justification
- **No Dead Code**: Unused code, imports, and commented-out blocks MUST be removed, not left for "future use"
- **Explicit over Implicit**: Dependencies, side effects, and error conditions MUST be visible at the call site
- **Type Safety**: Types MUST be explicit; Rust's type system should be leveraged fully — avoid `unwrap()` in production code without justification

**Rationale**: Maintainable code reduces cognitive load, speeds onboarding, and prevents bugs from creeping in during future changes.

### II. Testing Standards

Testing is a core part of development, not an afterthought.

**Non-Negotiable Rules**:

- **Test Coverage**: All public interfaces and critical paths MUST have tests; coverage gaps require documented justification
- **Test Independence**: Each test MUST be independently runnable and MUST NOT depend on execution order or shared mutable state
- **Meaningful Assertions**: Tests MUST verify behavior, not implementation; avoid testing private methods directly
- **Fast Feedback**: Unit tests MUST complete within seconds; slow tests MUST be isolated and clearly marked
- **Failure Clarity**: When a test fails, the failure message MUST clearly indicate what went wrong and where
- **Boundary Tests**: The core↔shell boundary MUST be tested — core logic via unit tests on events and effects, shell integration via WASM tests on persistence round-trips, and user journeys via E2E tests

**Rationale**: Tests provide confidence for refactoring, document expected behavior, and catch regressions before they reach users.

### III. User Experience Consistency

Users experience the product as a unified whole; inconsistency creates friction and erodes trust.

**Non-Negotiable Rules**:

- **Visual Consistency**: UI components MUST use consistent styling patterns; spacing, colours, typography, and component structure should be uniform across all views
- **Interaction Patterns**: Similar actions MUST behave similarly across the entire application; form submissions, navigation, and feedback patterns MUST be consistent
- **Error Communication**: Error messages MUST be user-friendly, actionable, and consistent in tone; technical jargon MUST NOT appear in user-facing messages
- **Loading States**: All operations that may take noticeable time MUST provide appropriate feedback; no silent waiting
- **Accessibility**: Features SHOULD follow WCAG 2.1 AA guidelines where practical; accessibility improvements should be prioritised when adding new UI components

**Rationale**: Consistent UX reduces user cognitive load, builds trust, and improves task completion rates.

### IV. Performance Requirements

Performance is a feature, not an optimisation to add later.

**Non-Negotiable Rules**:

- **WASM Bundle Size**: The compiled WASM bundle MUST remain reasonable for the feature set; unnecessary dependencies that bloat the binary MUST be avoided
- **Render Performance**: UI interactions (navigation, form input, list rendering) MUST feel instant to the user; operations on the full library dataset MUST complete without perceptible delay
- **Data Efficiency**: localStorage reads and writes MUST be efficient; the full dataset MUST NOT be serialised/deserialised unnecessarily on every interaction
- **Startup Time**: The app MUST load and become interactive promptly; first-run seeding MUST NOT cause noticeable delay
- **Measurement**: Performance claims require evidence; the core MUST include benchmark tests for operations on realistic dataset sizes (e.g., 10,000 items)

**Rationale**: Poor performance drives users away; in a client-side app, every millisecond of delay is felt directly by the user with no server-side optimisation to fall back on.

### V. Architecture Integrity

The Crux pure-core pattern is the foundation of this project and MUST be preserved.

**Non-Negotiable Rules**:

- **Pure Core**: `intrada-core` MUST contain zero I/O, zero side effects, and zero platform dependencies; it processes events and emits commands (effects) only
- **Shell Isolation**: All platform-specific code (DOM access, localStorage, browser APIs) MUST live in the shell crate (`intrada-web`), never in core
- **Effect-Driven Communication**: The core communicates with shells exclusively through the `Effect` enum and the `Command` API; direct function calls from shell to core internals are prohibited
- **Portable by Design**: The core MUST compile and test on any Rust target without WASM or browser dependencies; `cargo test` in `intrada-core` MUST NOT require a browser
- **Validation Sharing**: Validation constants and rules defined in core MUST be the single source of truth; shells MUST import and reuse them rather than duplicating

**Rationale**: The pure-core pattern enables testing business logic in isolation, adding new shells (iOS, desktop) without rewriting logic, and maintaining a clear separation between what the app does and how it interacts with the platform.

## Technical Decision Governance

This section defines how the principles above guide technical decisions and implementation choices.

### Decision Framework

When making technical decisions, evaluate against all five principles in priority order:

1. **Architecture Integrity**: Does this preserve the pure-core pattern?
2. **Code Quality**: Will this approach result in maintainable, readable code?
3. **Testing Standards**: Can this approach be effectively tested?
4. **UX Consistency**: Does this approach maintain consistent user experience?
5. **Performance**: Does this approach meet performance requirements?

Tradeoffs between principles MUST be documented in the relevant design document (plan.md or PR description).

### Implementation Guidance

- **New Features**: MUST pass Constitution Check in plan.md before implementation begins
- **Code Reviews**: Reviews MUST verify compliance with all five principles
- **Technical Debt**: Principle violations MUST be tracked as technical debt with remediation timelines
- **Exceptions**: Any exception to a principle MUST be documented with justification

### Compliance Verification

- **Pre-merge**: All PRs MUST demonstrate compliance via passing CI (tests, clippy, fmt)
- **Feature Completion**: Each feature MUST pass its quickstart.md verification steps before the PR is merged

## Amendment Process

This constitution governs all development practices. Changes follow this process:

### Proposing Amendments

1. Create a proposal describing the change and rationale
2. Identify impact on existing code and practices
3. Document in a spec or PR description

### Approval Requirements

- **Principle Changes**: Require a documented rationale and migration plan for existing code
- **Governance Changes**: Require documented justification and impact assessment
- **Clarifications**: Can be made directly with a clear commit message

### Version Policy

- **MAJOR**: Backward-incompatible principle changes or removals
- **MINOR**: New principles or significant guidance additions
- **PATCH**: Clarifications, typo fixes, non-semantic refinements

**Version**: 2.0.0 | **Ratified**: 2026-02-15 | **Last Amended**: 2026-02-15
