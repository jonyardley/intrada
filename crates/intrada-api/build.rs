fn main() {
    // Pair with `option_env!("GIT_SHA")` in src/main.rs so changing the env
    // (e.g. across CI deploys) invalidates the cached compile and the right
    // value is baked into the binary. Without this, cargo can serve a stale
    // const from a previous build.
    println!("cargo:rerun-if-env-changed=GIT_SHA");
}
