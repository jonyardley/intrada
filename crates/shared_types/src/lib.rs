//! Type generation crate for Intrada.
//!
//! This crate has no runtime code. Its `build.rs` uses `crux_core::typegen::TypeGen`
//! to generate Swift (and optionally Kotlin/TypeScript) type definitions for all
//! types crossing the FFI boundary.
//!
//! Generated files are written to `generated/swift/SharedTypes/`.
