//! Generates the iOS shell's shared Swift types from the core via facet typegen.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use crux_core::type_generation::facet::{Config, TypeRegistry};

use intrada_ffi::{Intrada, ListQuery};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let args = Args::parse();

    // `ListQuery` only appears as `Option<ListQuery>` inside an Event variant,
    // which facet references but doesn't emit a definition for, so register it
    // explicitly or the generated types do not compile (#382).
    let typegen = TypeRegistry::new()
        .register_app::<Intrada>()?
        .register_type::<ListQuery>()?
        .build()?;
    typegen.swift(&Config::builder("SharedTypes", &args.output_dir).build())?;

    Ok(())
}
