use std::process::Command;

fn main() {
    // get git commit sha-1
    let git_commit_sha = match Command::new("git").args(&["rev-parse", "HEAD"]).output() {
        Ok(out) => String::from_utf8(out.stdout).unwrap(),
        Err(_) => String::from("unknown"),
    };
    println!("cargo:rustc-env=GIT_COMMIT_SHA={}", git_commit_sha);
    println!("cargo:rerun-if-changed=build.rs");
}
