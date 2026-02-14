# Research: Music Library

**Feature Branch**: `001-music-library`
**Date**: 2026-02-08

## Decision 1: Storage Backend

**Decision**: SQLite via `rusqlite` with the `bundled` feature

**Rationale**: SQLite provides efficient querying, built-in FTS5 for text search, and a single-file local-first model that fits the single-user architecture perfectly. At 10,000 items, SQLite handles indexed queries with ease. The `bundled` feature simplifies cross-platform deployment by compiling SQLite from source — important for future iOS/web targets.

**Alternatives considered**:
- JSON files (serde_json): Rejected — requires loading entire dataset into memory for every query. Search/filter at 10,000 items would need custom indexing. Simple but doesn't scale to the performance targets.
- sled/redb: Rejected — less mature than SQLite, weaker querying capabilities, smaller ecosystem.

## Decision 2: ID Generation

**Decision**: ULID via the `ulid` crate

**Rationale**: ULIDs are lexicographically sortable (embedded timestamp in first 48 bits), making items naturally ordered by creation time. They're more compact than UUIDs (26 chars vs 36) and support future sync scenarios through temporal ordering and causality tracking. Stored as TEXT in SQLite for human readability.

**Alternatives considered**:
- UUID v4: Rejected — purely random, no temporal ordering. Harder to reason about creation order.
- UUID v7: Viable (adds timestamp-based sortability) but ULID has stronger Rust ecosystem support for this use case.
- nanoid: Rejected — no timestamp component, loses ordering and causality tracking needed for future sync.

## Decision 3: CLI Framework

**Decision**: `clap` v4.5 with derive feature

**Rationale**: clap remains the standard for Rust CLI applications. The derive API allows defining CLI structure as native Rust structs with minimal boilerplate. Provides automatic help/version generation, subcommands, type validation, and shell completion out of the box.

**Alternatives considered**:
- pico-args: Rejected — lacks help generation, derive support, and validation. Too minimal for a feature-rich tool.
- argh: Rejected — follows non-standard conventions, missing short flags and flexibility.

## Decision 4: Error Handling

**Decision**: `thiserror` for the core library crate, `anyhow` for the CLI binary crate

**Rationale**: This is the idiomatic Rust convention for workspace projects. The core library uses `thiserror` to define specific, well-typed errors (e.g. `ValidationError`, `StorageError`) that consumers can handle programmatically. The CLI binary uses `anyhow` for application-level error propagation with context chains. The two interop seamlessly via the `?` operator.

**Alternatives considered**:
- Custom error types without thiserror: Rejected — manual `Error`/`Display` implementations are boilerplate-heavy with no benefit.
- anyhow in the library crate: Rejected — library consumers lose the ability to match on specific error types.

## Decision 5: Workspace Structure

**Decision**: Flat `crates/` directory with virtual workspace manifest

**Rationale**: A virtual manifest (workspace-only root `Cargo.toml` without `[package]`) keeps all crates as equal participants and avoids polluting the root with `src/`. The `crates/` directory makes the crate list scannable. Shared dependency versions are centralised via `[workspace.dependencies]`.

**Alternatives considered**:
- Root-level main crate: Rejected — pollutes root with `src/`, requires `--workspace` flags, creates structural inconsistency.
- Nested hierarchy (e.g. `crates/frontends/cli/`): Rejected — Cargo's flat dependency model makes tree structures confusing.

## Decision 6: Search Implementation

**Decision**: In-memory filtering initially, with SQLite FTS5 as the production approach

**Rationale**: At 10,000 items, in-memory filtering (load all items, filter with case-insensitive string matching) completes in milliseconds and keeps the initial implementation simple. SQLite FTS5 provides indexed, sub-10ms searches and supports tokenisation and ranking — a natural progression when search needs become more sophisticated. Both approaches are behind the storage effect, so the transition is seamless.

**Alternatives considered**:
- tantivy: Rejected — full Lucene-like search engine designed for millions of documents. Overkill for 10,000 items.
- Pure regex: Viable for simple cases but slower than indexed search and doesn't support ranking.

## Decision 7: Application Framework

**Decision**: Crux (`crux_core` 0.16.2) with the Command-based API

**Rationale**: Crux is a cross-platform app development framework for Rust that enforces a pure core / effectful shell architecture. The core contains all business logic (event handling, validation, model updates) as pure functions with no side effects. Side effects (storage, rendering) are expressed as `Command<Effect, Event>` return values that the shell (CLI, iOS, web) fulfils. This matches the project's multi-frontend goal (CLI first, later iOS/web) perfectly — the same core crate works across all platforms with zero changes. The previous version of Intrada already used Crux successfully.

The Command API (crux_core 0.16+) replaces the older capability-injection pattern. `Capabilities = ()` — all effects are returned as Commands from `update()`, making the architecture simpler and more testable. The core is side-effect free, which means all tests are pure unit tests with no mocking needed.

**Key architecture**:
- **App trait**: `update(event, &mut model) -> Command<Effect, Event>` + `view(&model) -> ViewModel`
- **Model**: In-memory state (loaded from storage at startup)
- **ViewModel**: Computed from Model for display
- **Effects**: Render (trigger UI refresh) + Storage (persist to SQLite/UserDefaults/etc.)
- **Events**: User actions + storage response events
- **Shell**: Platform-specific code that processes effects and resolves them back to events

**Alternatives considered**:
- Plain Library trait (original plan): Rejected — tightly couples business logic to a specific storage implementation. Each frontend would need its own trait implementation. Doesn't naturally support the pure core / effectful shell separation that enables cross-platform sharing.
- Elm/Redux-style without Crux: Viable but Crux provides the FFI bridge (UniFFI for iOS, wasm-bindgen for web), type generation (facet), and a tested Command API. Building this from scratch would be reinventing Crux.

## Decision 8: Storage Architecture

**Decision**: Storage as a shell-side concern with Effect-based communication

**Rationale**: Following Crux's architecture, storage is handled entirely by the shell. The core defines Storage effects (e.g. SavePiece, LoadAll, DeleteItem) that the shell fulfils using platform-appropriate mechanisms. For the CLI shell, this means SQLite via rusqlite. For a future iOS shell, this could be UserDefaults, Core Data, or CloudKit. The core receives data back via events (e.g. DataLoaded, SaveCompleted).

This keeps the core pure and testable — tests simply verify that the right effects are returned for given events, without needing any database. The Model holds the in-memory state, and the shell is responsible for persisting it.

**Alternatives considered**:
- crux_kv capability: Viable for simple key-value storage but too limiting for structured queries (search, filter by type/key/category/tags). SQLite's query capabilities are needed.
- Storage inside the core: Rejected — would make the core impure, breaking Crux's architecture and making cross-platform sharing impossible.
