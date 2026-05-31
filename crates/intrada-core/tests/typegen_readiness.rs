//! Typegen-readiness: the native iOS bridge generates Swift types from the
//! core via `crux_core`'s facet reflection. That only works if every type
//! reachable from `Event` / `Effect` / `ViewModel` derives `Facet`. This test
//! is the discovery + regression guard: if a new field introduces a type that
//! isn't facet-representable, `register_app` fails here rather than in the iOS
//! build. Gated on `facet_typegen` so it's a no-op for default builds.
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
