//! Build script to capture git hash for reproducibility tracking.

use std::process::Command;

fn main() {
    // Get git hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string());

    if let Some(hash) = git_hash {
        println!("cargo:rustc-env=GIT_HASH={}", hash);
    }

    // Re-run if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
}
