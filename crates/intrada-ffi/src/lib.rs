//! Native-shell FFI bridge for the Crux core — see CLAUDE.md "Native iOS Shell".

pub use intrada_core::*;

pub mod ffi;
pub use ffi::{CoreError, CoreFFI};

// Pin + assert: a uniffi version skew vs cargo-swift's bindgen becomes a compile
// error here, not a runtime mismatch in the iOS build.
#[cfg(feature = "uniffi")]
const _: () = assert!(
    uniffi::check_compatible_version("0.29.4"),
    "use uniffi v0.29.4 to match cargo-swift's bindgen"
);
#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
