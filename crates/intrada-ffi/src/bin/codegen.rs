//! Generates the shells' shared types from the core via facet typegen.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use crux_core::type_generation::facet::{Config, TypeRegistry};

use intrada_ffi::{Intrada, ListQuery};

#[derive(Clone, Copy, ValueEnum)]
enum Lang {
    Swift,
    Kotlin,
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    output_dir: PathBuf,
    #[arg(short, long, value_enum, default_value_t = Lang::Swift)]
    lang: Lang,
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
    match args.lang {
        Lang::Swift => typegen.swift(&Config::builder("SharedTypes", &args.output_dir).build())?,
        Lang::Kotlin => {
            typegen.kotlin(&Config::builder("com.intrada.shared", &args.output_dir).build())?
        }
    }

    Ok(())
}
