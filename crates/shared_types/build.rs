use anyhow::Result;
use std::path::PathBuf;

use crux_core::type_generation::facet::{Config, TypeRegistry};
use shared::{Intrada, ListQuery};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../shared/src");
    println!("cargo:rerun-if-changed=../intrada-core/src");

    let mut gen = TypeRegistry::new();
    gen.register_app::<Intrada>()?;
    // ListQuery is used inside Option<ListQuery> in Event::SetQuery.
    // The facet typegen doesn't auto-discover types nested inside Option,
    // so we register it explicitly.
    gen.register_type::<ListQuery>()?;
    let codegen = gen.build()?;

    let output_root = PathBuf::from("./generated");

    codegen.swift(
        &Config::builder("SharedTypes", output_root.join("swift"))
            .add_extensions()
            .add_runtimes()
            .build(),
    )?;

    Ok(())
}
