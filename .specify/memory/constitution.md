<!--
================================================================================
SYNC IMPACT REPORT
================================================================================
Version change: 2.0.0 → 3.0.0 (MAJOR - Reflects API server, updated persistence model, inclusive design principles)

Modified principles:
- III. User Experience Consistency: Strengthened accessibility from SHOULD to MUST for new components; acknowledged component library and glassmorphism design language
- IV. Performance Requirements: Replaced localStorage-centric rules with API-aware performance principles; localStorage now only for crash recovery
- V. Architecture Integrity: Expanded to cover three-crate architecture (core, web, api); added API server rules

Added principles:
- VI. Inclusive Design: Codifies the neurodiversity-informed design principles from VISION.md as non-negotiable

Modified sections:
- Technical Decision Governance: Added inclusive design to decision framework
- Decision Framework: Reordered to reflect product priorities alongside technical ones

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
- **Boundary Tests**: The core↔shell boundary MUST be tested — core logic via unit tests on events and effects, shell integration via WASM tests on persistence round-trips, API endpoints via integration tests, and user journeys via E2E tests

**Rationale**: Tests provide confidence for refactoring, document expected behavior, and catch regressions before they reach users.

### III. User Experience Consistency

Users experience the product as a unified whole; inconsistency creates friction and erodes trust.

**Non-Negotiable Rules**:

- **Component Library**: UI MUST be built from the shared component library (Button, Card, TextField, TextArea, BackLink, PageHeading, etc.); new reusable patterns MUST be extracted into components rather than duplicated across views
- **Design Language**: All views MUST follow the established glassmorphism design language — translucent cards, consistent border/backdrop treatments, the defined colour palette, and Tailwind v4 utility classes
- **Interaction Patterns**: Similar actions MUST behave similarly across the entire application; form submissions, navigation, and feedback patterns MUST be consistent
- **Error Communication**: Error messages MUST be user-friendly, actionable, and consistent in tone; technical jargon MUST NOT appear in user-facing messages
- **Loading States**: All operations that may take noticeable time MUST provide appropriate feedback; no silent waiting
- **Accessibility**: New UI components MUST follow WCAG 2.1 AA guidelines; existing components SHOULD be improved toward AA compliance when modified. All interactive elements MUST have appropriate ARIA attributes, and all form inputs MUST have associated labels

**Rationale**: Consistent UX reduces user cognitive load, builds trust, and improves task completion rates. Accessibility is central to the product vision, not an optional enhancement.

### IV. Performance Requirements

Performance is a feature, not an optimisation to add later.

**Non-Negotiable Rules**:

- **WASM Bundle Size**: The compiled WASM bundle MUST remain reasonable for the feature set; unnecessary dependencies that bloat the binary MUST be avoided
- **Render Performance**: UI interactions (navigation, form input, list rendering) MUST feel instant to the user; operations on the full library dataset MUST complete without perceptible delay
- **API Responsiveness**: API endpoints MUST respond promptly; database queries MUST use appropriate indices; N+1 query patterns MUST be avoided
- **Network Efficiency**: The web shell MUST minimise redundant API calls; data fetched from the API SHOULD be used for the duration of the session rather than re-fetched unnecessarily
- **Crash Recovery**: Session-in-progress state MUST be persisted to localStorage for crash recovery; this is the only permitted use of localStorage for application data
- **Startup Time**: The app MUST load and become interactive promptly; initial data fetch MUST NOT block the UI from rendering
- **Measurement**: Performance claims require evidence; the core MUST include benchmark tests for operations on realistic dataset sizes (e.g., 10,000 items)

**Rationale**: Poor performance drives users away. In a client-side WASM app backed by a remote API, both client responsiveness and network efficiency directly affect the musician's experience.

### V. Architecture Integrity

The three-crate architecture is the foundation of this project and MUST be preserved.

**Non-Negotiable Rules**:

- **Pure Core**: `intrada-core` MUST contain zero I/O, zero side effects, and zero platform dependencies; it processes events and emits commands (effects) only
- **Shell Isolation**: All platform-specific code (DOM access, localStorage, browser APIs, HTTP calls) MUST live in the shell crate (`intrada-web`), never in core
- **API Server Isolation**: `intrada-api` MUST handle HTTP routing, database access, and persistence independently; it shares domain types with core via the workspace but MUST NOT depend on shell code
- **Effect-Driven Communication**: The core communicates with shells exclusively through the `Effect` enum and the `Command` API; direct function calls from shell to core internals are prohibited
- **Portable by Design**: The core MUST compile and test on any Rust target without WASM or browser dependencies; `cargo test` in `intrada-core` MUST NOT require a browser
- **Validation Sharing**: Validation constants and rules defined in core MUST be the single source of truth; shells and the API MUST import and reuse them rather than duplicating

**Rationale**: The three-crate architecture enables testing business logic in isolation, adding new shells (iOS, desktop) without rewriting logic, evolving the API independently, and maintaining a clear separation between what the app does, how it presents, and how it persists.

### VI. Inclusive Design

Intrada is designed for all musicians, including those who are neurodivergent. Inclusive design is not a separate workstream — it is embedded in every feature.

**Non-Negotiable Rules**:

- **Reduce Decisions to Start**: The path from opening the app to actively practising MUST involve as few decisions as possible; every decision point is an opportunity for a musician to stall
- **Externalise Time**: Practice sessions MUST provide visible, always-present time cues (timers, progress indicators); musicians MUST NOT have to rely on internal time perception
- **Celebrate Comeback, Not Streak**: Consistency messaging MUST use positive, return-focused framing ("4 of the last 7 days") rather than streak-based framing that penalises gaps; the app MUST NOT show broken streaks or zeroes
- **Sensory Consideration**: The app MUST NOT auto-play sounds, use sudden animations, or display bright flashing elements; all haptic and sound cues MUST be optional and configurable
- **Configurable Feedback**: Encouragement messaging MUST be configurable in frequency, tone, and delivery; the app MUST NOT assume one motivational approach works for all musicians
- **Predictable Navigation**: Navigation elements MUST remain in consistent positions across all views; core workflows MUST follow the same pattern every time

**Rationale**: An estimated 15–20% of the population is neurodivergent, and the intersection of ADHD with music is particularly significant. Features that support executive function challenges, time blindness, and sensory sensitivity benefit every user. This is grounded in the research foundation in VISION.md.

## Technical Decision Governance

This section defines how the principles above guide technical decisions and implementation choices.

### Decision Framework

When making technical decisions, evaluate against all six principles. Architecture and inclusive design are foundational; the others are weighed by context:

1. **Architecture Integrity**: Does this preserve the three-crate pure-core pattern?
2. **Inclusive Design**: Does this maintain low friction, predictable interaction, and accessibility?
3. **Code Quality**: Will this approach result in maintainable, readable code?
4. **UX Consistency**: Does this approach maintain consistent user experience?
5. **Testing Standards**: Can this approach be effectively tested?
6. **Performance**: Does this approach meet performance requirements?

Tradeoffs between principles MUST be documented in the relevant design document (plan.md or PR description).

### Implementation Guidance

- **New Features**: MUST pass Constitution Check in plan.md before implementation begins
- **Code Reviews**: Reviews MUST verify compliance with all six principles
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

**Version**: 3.0.0 | **Ratified**: 2026-02-18 | **Last Amended**: 2026-02-18
