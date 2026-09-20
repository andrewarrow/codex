use std::path::Path;
use std::process::Command;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-ObjC");
    }

    stamp_build_commit();
}

fn stamp_build_commit() {
    println!("cargo:rerun-if-env-changed=STABLE_GIT_COMMIT");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");

    // Explicit stamps take precedence over a local checkout, matching
    // scripts/workspace-status.sh.
    if let Some(commit) = std::env::var("STABLE_GIT_COMMIT")
        .ok()
        .and_then(|value| valid_commit(Some(&value)).map(str::to_owned))
    {
        println!("cargo:rustc-env=STABLE_GIT_COMMIT={commit}");
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let checkout = Path::new(&manifest_dir);
    track_git_head(checkout);

    let commit = git_output(checkout, &["rev-parse", "--verify", "HEAD"])
        .and_then(|value| valid_commit(Some(&value)).map(str::to_owned))
        .or_else(|| {
            std::env::var("GITHUB_SHA")
                .ok()
                .and_then(|value| valid_commit(Some(&value)).map(str::to_owned))
        });

    if let Some(commit) = commit {
        println!("cargo:rustc-env=STABLE_GIT_COMMIT={commit}");
    }
}

fn track_git_head(checkout: &Path) {
    if let Some(head_path) = git_output(checkout, &["rev-parse", "--git-path", "HEAD"]) {
        println!("cargo:rerun-if-changed={head_path}");
    }

    if let Some(reference) = git_output(checkout, &["symbolic-ref", "-q", "HEAD"])
        && let Some(reference_path) = git_output(checkout, &["rev-parse", "--git-path", &reference])
    {
        println!("cargo:rerun-if-changed={reference_path}");
    }
}

fn git_output(checkout: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn valid_commit(value: Option<&str>) -> Option<&str> {
    value.filter(|commit| commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
