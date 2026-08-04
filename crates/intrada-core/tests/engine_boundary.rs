//! The engine quarantine (`specs/intrada-coach-engine.md` §1), enforced rather
//! than trusted: `engine/` never reads the self-report scoring path, and it
//! carries no `local_first` branch. A diff that adds one fails here.
//!
//! A source-text check, not a type check, because the thing being prevented is
//! an *import* — gravity, not a compile error. Enforced in CI via `just test`.

use std::fs;
use std::path::Path;

/// Every symbol whose presence would mean the engine had reached into the
/// self-report path or grown a dual-mode branch.
const FORBIDDEN: &[(&str, &str)] = &[
    ("analytics", "the self-report scoring path (§1)"),
    ("ItemPracticeSummary", "the self-report scoring path (§1)"),
    ("practice_summaries", "the self-report scoring path (§1)"),
    ("local_first", "dual-mode is retired in engine/ (§1)"),
];

/// The rule is about imports, not prose — a doc comment naming what the engine
/// must not touch is the documentation working, so strip comments first.
fn code_only(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

fn rust_files(dir: &Path, out: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(dir).expect("engine/ should exist") {
        let path = entry.expect("readable dir entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            out.push((
                name,
                code_only(&fs::read_to_string(&path).expect("readable source")),
            ));
        }
    }
}

fn engine_sources() -> Vec<(String, String)> {
    let mut sources = Vec::new();
    rust_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/engine"),
        &mut sources,
    );
    sources.sort();
    sources
}

#[test]
fn the_engine_never_reaches_into_the_self_report_path() {
    let sources = engine_sources();
    assert!(!sources.is_empty(), "no engine sources found to check");

    for (name, source) in &sources {
        for (symbol, why) in FORBIDDEN {
            assert!(
                !source.contains(symbol),
                "engine/{name} mentions `{symbol}` — {why}"
            );
        }
    }
}

/// The quarantine only holds while the coach types are declared *inside* it —
/// move one out and the boundary test above stops covering it.
#[test]
fn the_engine_declares_the_coach_state_machine_and_bridge_types() {
    let sources = engine_sources();
    for declaration in [
        "pub struct CoachState",
        "pub struct CoachView",
        "pub enum CoachEvent",
        "pub enum SessionState",
        "pub struct BlockRecord",
    ] {
        assert!(
            sources
                .iter()
                .any(|(_, source)| source.contains(declaration)),
            "`{declaration}` must be declared in engine/"
        );
    }
}
