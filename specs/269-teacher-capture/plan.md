# Implementation Plan: Teacher Assignment Capture

**Branch**: `269-teacher-capture` | **Date**: 2026-04-11 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/269-teacher-capture/spec.md`

## Summary

Introduce a **Lesson** entity — a lightweight record of a teaching session (date, notes, photos). This is the core Layer 1 (Capture) feature: musicians capture raw lesson information in under 30 seconds, with no forced organisation. The lesson entity is standalone in this iteration; item linking is deferred to a follow-up feature.

Technical approach: new Crux domain module (events, model, effects), new API endpoints with Turso persistence, photo storage in Cloudflare R2 with shell-managed uploads (outside Crux's JSON-only HTTP), and new UI screens on both web (Leptos) and iOS (SwiftUI).

## Technical Context

**Language/Version**: Rust stable 1.89.0 (core, API, web), Swift 6.0 (iOS)  
**Primary Dependencies**: crux_core 0.17.0-rc3, axum 0.8, leptos 0.8, SwiftUI, UniFFI  
**Storage**: Turso (libsql) for lesson metadata, Cloudflare R2 for photos  
**Testing**: cargo test (core + API), Playwright (E2E), just ios-swift-check (iOS)  
**Target Platform**: Web (WASM/CSR) + iOS 17+  
**Project Type**: Multi-crate workspace + iOS app  
**Performance Goals**: Lesson capture in <30s user time, lesson list renders instantly for <1000 lessons  
**Constraints**: Crux HTTP is JSON-only (no multipart), 1MB body limit, photos handled by shell  
**Scale/Scope**: Single user, ~100 lessons/year typical, 1-5 photos per lesson

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Code Quality | ✅ Pass | Follows existing entity patterns (items, sessions, routines). Single responsibility per module. |
| II. Testing Standards | ✅ Pass | Core unit tests for lesson events/effects, API integration tests, E2E tests for user flows. Boundary tested at core↔shell and API layers. |
| III. UX Consistency | ✅ Pass | Uses existing component library (Card, Button, TextField, TextArea). Pencil designs follow glassmorphism language. |
| IV. Performance | ✅ Pass | Lesson list is small dataset (<1000). Photos stored externally, not in DB. No N+1 queries (photos joined on lesson fetch). |
| V. Architecture Integrity | ✅ Pass | Core is pure (events → effects, no I/O). Photo upload handled by shell, not core. API handles persistence independently. |
| VI. Inclusive Design | ✅ Pass | Minimal decisions to start (date auto-filled, one text field). No forced structure. Predictable flow. |

No violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/269-teacher-capture/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: technical decisions
├── data-model.md        # Phase 1: entity schemas
├── quickstart.md        # Phase 1: verification steps
├── contracts/
│   └── api.md           # Phase 1: API endpoint contracts
└── checklists/
    └── requirements.md  # Spec quality checklist
```

### Source Code (repository root)

```text
crates/
  intrada-core/
    src/
      domain/
        lesson.rs          # NEW: Lesson event handling, model updates
        types.rs           # MODIFIED: Add Lesson, LessonPhoto, CreateLesson, UpdateLesson
        mod.rs             # MODIFIED: Add lesson module
      validation.rs        # MODIFIED: Add lesson validation rules
      http.rs              # MODIFIED: Add lesson API request builders
      view_model.rs        # MODIFIED: Add lessons to ViewModel

  intrada-api/
    src/
      migrations.rs        # MODIFIED: Add lessons + lesson_photos tables
      db/
        lessons.rs         # NEW: Lesson DB queries (CRUDL + photo queries)
        mod.rs             # MODIFIED: Add lessons module
      routes/
        lessons.rs         # NEW: Lesson API routes + photo upload
        mod.rs             # MODIFIED: Mount lesson routes
      storage.rs           # NEW: R2 client for photo upload/delete

  intrada-web/
    src/
      views/
        lessons.rs         # NEW: Lessons list + detail + capture form views
        mod.rs             # MODIFIED: Add lessons module
      components/
        photo_upload.rs    # NEW: Photo upload component (file input + preview)
        mod.rs             # MODIFIED: Add photo_upload

  shared/                  # MODIFIED: Lesson types flow through UniFFI/BCS bridge
  shared_types/            # AUTO: Facet typegen generates Swift types

ios/Intrada/
  Features/Lessons/
    LessonListView.swift       # NEW: Lessons list screen
    LessonDetailView.swift     # NEW: Lesson detail screen
    LessonCaptureView.swift    # NEW: Capture form screen
    PhotoCaptureView.swift     # NEW: Camera/photo picker + upload
```

**Structure Decision**: Follows existing multi-crate pattern. Lesson is a new domain module in core, new route group in API, new view group in web and iOS. Photo upload is a new shell-level capability (not a Crux effect) in both web and iOS shells.
