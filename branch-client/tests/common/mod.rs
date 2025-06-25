use assert_cmd::cargo::cargo_bin;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = dir.path();
    Command::new("git").arg("init").current_dir(repo).output().unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "user"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "user@example.com"])
        .current_dir(repo)
        .output()
        .unwrap();
    std::fs::write(repo.join("file"), "a").unwrap();
    Command::new("git")
        .args(["add", "file"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(repo)
        .output()
        .unwrap();
    dir
}

pub fn bin() -> PathBuf {
    cargo_bin("branch-client")
}
