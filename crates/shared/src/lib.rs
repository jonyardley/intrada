//! FFI bridge crate for Intrada.
//!
//! Wraps `intrada-core` with a `crux_core::Bridge` and exposes it
//! via UniFFI proc-macro bindings for iOS/Android consumption.
//!
//! Two bridge implementations are provided:
//!
//! - `CoreFFI` — BCS binary serialisation via `crux_core::Bridge` (for future use)
//! - `CoreJson` — JSON string serialisation via `crux_core::Core` (used by iOS shell)
//!
//! The JSON bridge exists because `crux_core::typegen` codegen for Swift BCS
//! is blocked by a `serde-reflection` limitation with GoalKind's enum variants.

pub use intrada_core::*;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

mod core;
pub use self::core::CoreFFI;

mod json_bridge;
pub use self::json_bridge::CoreJson;
