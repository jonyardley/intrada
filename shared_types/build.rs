use crux_core::typegen::TypeGen;
use shared::Chopin;
// use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=../shared");

    let mut gen = TypeGen::new();

    gen.register_app::<Chopin>()?;

    //let output_root = PathBuf::from("./generated");

    // gen.swift("SharedTypes", output_root.join("swift"))?;

    // gen.java("com.example.counter.shared_types", output_root.join("java"))?;

    // gen.typescript("shared_types", output_root.join("typescript"))?;

    Ok(())
}
