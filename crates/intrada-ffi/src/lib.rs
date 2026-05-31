//! `intrada-ffi` — the native-shell bridge for the Crux core.
//!
//! A thin dumb pipe: UniFFI exposes the three byte-buffer bridge methods
//! (`update` / `resolve` / `view`) and the `codegen` bin emits the Swift type
//! package via facet reflection. No domain logic lives here — see the
//! "Native iOS Shell" section in CLAUDE.md.

pub use intrada_core::*;

pub mod ffi;
pub use ffi::{CoreError, CoreFFI};

// UniFFI's generated Swift must match the bindgen cargo-swift bundles; pinning
// + this assert turns a version skew into a compile error rather than a
// runtime mismatch in the iOS build.
#[cfg(feature = "uniffi")]
const _: () = assert!(
    uniffi::check_compatible_version("0.29.4"),
    "use uniffi v0.29.4 to match cargo-swift's bindgen"
);
#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
