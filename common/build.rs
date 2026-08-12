use std::process::Command;

fn main() {
    // ---- Build timestamp ----
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    println!("cargo:rustc-env=JUICITY_BUILD_TIMESTAMP={}", timestamp);

    // ---- Git commit hash ----
    // CI may override via JUICITY_BUILD_COMMIT (e.g. inject the real `github.sha`)
    let git_hash = match env_override("JUICITY_BUILD_COMMIT") {
        Some(h) => h,
        None => git_command(&["rev-parse", "--short=7", "HEAD"])
            .unwrap_or_else(|| "unknown".to_string()),
    };
    println!("cargo:rustc-env=JUICITY_GIT_HASH={}", git_hash);

    // ---- Git tag ----
    // CI may override via JUICITY_BUILD_TAG:
    //   - release: use the exact release tag passed by the caller
    //   - otherwise: keep the existing style (exact tag / "unstable (<describe>)" / "unstable")
    let git_tag = match env_override("JUICITY_BUILD_TAG") {
        Some(t) => t,
        None => match git_command(&["describe", "--tags", "--exact-match", "HEAD"]) {
            Some(t) => t,
            None => match git_command(&["describe", "--tags"]) {
                Some(d) => format!("unstable ({})", d),
                None => "unstable".to_string(),
            },
        },
    };
    println!("cargo:rustc-env=JUICITY_GIT_TAG={}", git_tag);

    // Rerun build script only when Git HEAD changes (or if build.rs itself changes)
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/tags");
    println!("cargo:rerun-if-changed=../../.git/packed-refs");

    // Rerun when CI overrides the tag / commit baked into the build
    println!("cargo:rerun-if-env-changed=JUICITY_BUILD_TAG");
    println!("cargo:rerun-if-env-changed=JUICITY_BUILD_COMMIT");
}

/// Read a CI override environment variable (ignores empty values).
fn env_override(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

/// Run `git` with the given args; return trimmed stdout on success.
fn git_command(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                None
            }
        })
}
