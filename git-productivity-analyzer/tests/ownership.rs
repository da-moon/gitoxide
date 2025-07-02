use assert_cmd::cargo::cargo_bin;
use std::fs::{self, File};
use std::process::Command;
use tempfile::TempDir;

mod util;

fn init_repo() -> TempDir {
    let dir = util::init_repo();
    let repo = dir.path();
    fs::create_dir(repo.join("src")).unwrap();
    fs::create_dir(repo.join("docs")).unwrap();
    File::create(repo.join("README")).unwrap();
    Command::new("git")
        .args(["add", "README"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args([
            "-c",
            "user.name=Alice",
            "-c",
            "user.email=a@example.com",
            "commit",
            "-m",
            "root",
        ])
        .current_dir(repo)
        .output()
        .unwrap();
    File::create(repo.join("src/a")).unwrap();
    Command::new("git")
        .args(["add", "src/a"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args([
            "-c",
            "user.name=Alice",
            "-c",
            "user.email=a@example.com",
            "commit",
            "-m",
            "init",
        ])
        .current_dir(repo)
        .output()
        .unwrap();
    File::create(repo.join("docs/b")).unwrap();
    Command::new("git")
        .args(["add", "docs/b"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args([
            "-c",
            "user.name=Bob",
            "-c",
            "user.email=b@example.com",
            "commit",
            "-m",
            "docs",
        ])
        .current_dir(repo)
        .output()
        .unwrap();
    dir
}

fn bin() -> std::path::PathBuf {
    cargo_bin("git-productivity-analyzer")
}

#[test]
fn ownership_default() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["ownership", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn ownership_json() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["--json", "ownership", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(v.is_object());
    for (_dir, authors) in v.as_object().unwrap() {
        assert!(authors.is_object());
    }
}
