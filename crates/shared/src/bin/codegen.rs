//! CLI tool for generating Swift/Kotlin type bindings.
//!
//! Build and run with:
//! ```sh
//! cargo build -p shared --features codegen
//! cargo run -p shared --features codegen -- --language swift --output-dir ./generated/swift
//! ```

fn main() -> anyhow::Result<()> {
    crux_core::cli::run(Some("shared"))
}
