//! Generates the Swift `SharedTypes` package (Event / Effect / ViewModel +
//! bincode serializers) from the Crux core via facet reflection. Mirrors the
//! crux `counter` example's codegen bin, Swift-only (Android comes later).

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use crux_core::type_generation::facet::{Config, TypeRegistry};

use intrada_ffi::{Intrada, ListQuery};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Directory to write the generated Swift package into.
    #[arg(short, long)]
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();

    // `ListQuery` only appears as `Option<ListQuery>` inside an Event variant,
    // which facet references but doesn't emit a definition for — register it
    // explicitly so the generated Swift compiles (same workaround as #382).
    let typegen = TypeRegistry::new()
        .register_app::<Intrada>()?
        .register_type::<ListQuery>()?
        .build()?;
    let config = Config::builder("SharedTypes", &args.output_dir).build();
    typegen.swift(&config)?;

    Ok(())
}
