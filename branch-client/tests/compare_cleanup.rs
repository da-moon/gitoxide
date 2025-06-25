mod common;
use common::{bin, init_repo};
use std::process::Command;

#[test]
fn compare_branches() {
    let dir = init_repo();
    let repo = dir.path();
    Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(repo)
        .output()
        .unwrap();
    std::fs::write(repo.join("f2"), "a").unwrap();
    Command::new("git")
        .args(["add", "f2"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feature commit"])
        .current_dir(repo)
        .output()
        .unwrap();
    let output = Command::new(bin())
        .args(["--repo", repo.to_str().unwrap(), "compare", "feature", "master"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("ahead 1"));
}

#[test]
fn cleanup_branches() {
    let dir = init_repo();
    let repo = dir.path();
    Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(repo)
        .output()
        .unwrap();
    std::fs::write(repo.join("f2"), "a").unwrap();
    Command::new("git")
        .args(["add", "f2"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feature commit"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["checkout", "master"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["merge", "feature", "--no-edit", "--no-ff"])
        .current_dir(repo)
        .output()
        .unwrap();
    let output = Command::new(bin())
        .args(["--repo", repo.to_str().unwrap(), "cleanup", "--dry-run"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("would delete feature"));
    Command::new(bin())
        .args(["--repo", repo.to_str().unwrap(), "cleanup"])
        .status()
        .unwrap();
    let branches = Command::new("git").args(["branch"]).current_dir(repo).output().unwrap();
    assert!(!String::from_utf8_lossy(&branches.stdout).contains("feature"));
}
