fn main() {
    // Pair with `option_env!` reads in src/lib.rs so changing the env after
    // a build doesn't yield stale binaries.
    println!("cargo:rerun-if-env-changed=SENTRY_DSN_MOBILE");
    println!("cargo:rerun-if-env-changed=GIT_SHA");
    tauri_build::build()
}
