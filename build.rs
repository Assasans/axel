use std::process::Command;

fn main() {
  println!("cargo:rerun-if-changed=.git/HEAD");
  println!("cargo:rerun-if-changed=.git/index");

  let git_hash = Command::new("git")
    .args(["rev-parse", "HEAD"])
    .output()
    .map(|output| String::from_utf8(output.stdout).unwrap_or_default().trim().to_string())
    .unwrap_or_else(|_| "unknown".to_string());
  println!("cargo:rustc-env=GIT_HASH={}", git_hash);

  let is_dirty = Command::new("git")
    .args(["diff-index", "--quiet", "--ignore-submodules", "HEAD", "--"])
    .status()
    .map(|s| !s.success())
    .unwrap_or(true);
  println!("cargo:rustc-env=GIT_DIRTY={}", if is_dirty { "1" } else { "0" });

  let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".into());
  println!("cargo:rustc-env=BUILD_PROFILE={}", profile);
}
