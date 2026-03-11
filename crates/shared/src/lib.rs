// The crux `#[effect]` macro generates an enum with large variant size differences;
// we can't Box through the macro so we suppress the lint.
#![allow(clippy::large_enum_variant)]

//! FFI bridge crate for Intrada.
//!
//! Wraps `intrada-core` with a `crux_core::Bridge` and exposes it
//! via UniFFI proc-macro bindings for iOS/Android consumption.
//!
//! Two bridge implementations are provided:
//!
//! - `CoreFFI` — BCS binary serialisation via `crux_core::Bridge` (primary, used by iOS shell)
//! - `CoreJson` — JSON string serialisation via `crux_core::Core` (legacy, kept for testing)
//!
//! The iOS shell uses `CoreFFI` with auto-generated BCS types from `shared_types`.
//! The JSON bridge is retained for integration testing and as a reference.

pub use intrada_core::*;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

mod core;
pub use self::core::CoreFFI;

mod json_bridge;
pub use self::json_bridge::CoreJson;
