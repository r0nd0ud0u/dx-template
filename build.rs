fn main() {
    // `SERVER_URL`/`INSECURE_ACCEPT_INVALID_CERTS` can be baked in at compile time (via
    // `option_env!` in src/main.rs) as a fallback for native builds — desktop clients get
    // a real environment at launch, but an installed Android APK has no shell to read
    // env vars from at runtime, so mobile releases need the value baked in at build time.
    // Without these, cargo wouldn't know to recompile when only the env var (not any
    // source file) changes between builds, silently keeping a stale baked-in value.
    println!("cargo:rerun-if-env-changed=SERVER_URL");
    println!("cargo:rerun-if-env-changed=INSECURE_ACCEPT_INVALID_CERTS");
}
