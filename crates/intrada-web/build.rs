// Tells cargo to rebuild this crate whenever these env vars change. Without
// this, `option_env!` reads the value at first compile and bakes it into a
// const — subsequent .env or shell env changes don't trigger a rebuild,
// leading to stale WASM artifacts that point at the wrong API URL or use a
// stale Clerk key.
fn main() {
    println!("cargo:rerun-if-env-changed=INTRADA_API_URL");
    println!("cargo:rerun-if-env-changed=CLERK_PUBLISHABLE_KEY");
}
