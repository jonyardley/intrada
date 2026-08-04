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
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn engine_sources() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/engine");
    let mut sources: Vec<(String, String)> = fs::read_dir(&dir)
        .expect("engine/ should exist")
        .map(|entry| entry.expect("readable dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            (
                name,
                code_only(&fs::read_to_string(&path).expect("readable source")),
            )
        })
        .collect();
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

#[test]
fn the_engine_owns_the_coach_state_and_view() {
    let sources = engine_sources();
    let names: Vec<&str> = sources.iter().map(|(name, _)| name.as_str()).collect();
    assert!(
        names.contains(&"coach.rs"),
        "engine/coach.rs is the bridge surface"
    );
    assert!(
        names.contains(&"session.rs"),
        "engine/session.rs is the state machine"
    );
}
