use anyhow::Result;

// NOTE: Automated typegen via crux_core::typegen::TypeGen is currently blocked by
// a serde-reflection limitation: GoalKind uses `#[serde(tag = "type")]` (internally-
// tagged enum) whose variants have different field structures. serde-reflection
// cannot trace this pattern.
//
// The BCS binary encoding used by the Crux FFI bridge does NOT depend on serde tag
// attributes, so the FFI format is correct regardless. Swift types are written
// manually in ios/Intrada/Generated/SharedTypes.swift until either:
//   - GoalKind is restructured (breaking API change), or
//   - crux_core gains facet-based typegen support for our types.
//
// See: https://github.com/novifinancial/serde-reflection/issues/15

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../shared/src");
    println!("cargo:rerun-if-changed=../intrada-core/src");

    // Placeholder — typegen disabled until GoalKind compatibility is resolved.
    // When re-enabling, use:
    //   let mut gen = crux_core::typegen::TypeGen::new();
    //   gen.register_type_with_samples::<AppEffect>(samples)?;
    //   gen.register_type_with_samples::<Event>(samples)?;
    //   gen.register_app::<Intrada>()?;
    //   gen.swift("SharedTypes", "generated/swift")?;

    Ok(())
}
