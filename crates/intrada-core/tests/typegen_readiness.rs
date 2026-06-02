//! Typegen-readiness guard: the native iOS bridge generates Swift types via
//! facet reflection, which requires every type reachable from
//! `Event` / `Effect` / `ViewModel` to derive `Facet`. A non-representable
//! type fails `register_app` here rather than in the iOS build.
#![cfg(feature = "facet_typegen")]

use crux_core::type_generation::facet::TypeRegistry;
use intrada_core::Intrada;

#[test]
fn registers_full_app_type_graph() {
    let mut registry = TypeRegistry::new();
    registry
        .register_app::<Intrada>()
        .expect("register_app should cover the full Intrada Event/Effect/ViewModel graph");
    registry
        .build()
        .expect("type registry should build into a code generator");
}
