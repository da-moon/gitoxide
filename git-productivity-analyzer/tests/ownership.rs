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

#[test]
fn ownership_default() {
    let dir = init_repo();
    let output = util::run(&["ownership", "--working-dir", dir.path().to_str().unwrap()]);
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn ownership_json() {
    let dir = init_repo();
    let v = util::run_json(&["--json", "ownership", "--working-dir", dir.path().to_str().unwrap()]);
    let obj = v.as_object().expect("top-level JSON object");
    assert!(obj.contains_key("."));
    assert!(obj.contains_key("src"));
    assert!(obj.contains_key("docs"));
    let src = obj.get("src").unwrap().as_object().unwrap();
    assert!(src.contains_key("Alice <a@example.com>"));
}

#[test]
fn ownership_depth() {
    let dir = init_repo();
    let output = util::run(&[
        "ownership",
        "--working-dir",
        dir.path().to_str().unwrap(),
        "--depth",
        "0",
    ]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.lines().any(|l| l.starts_with(".")));
}

#[test]
fn ownership_path_filter() {
    let dir = init_repo();
    let output = util::run(&[
        "ownership",
        "--working-dir",
        dir.path().to_str().unwrap(),
        "--path",
        "src/*",
    ]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("src"));
    assert!(!out.contains("docs"));
}

#[test]
fn ownership_path_filter_no_match() {
    let dir = init_repo();
    let output = util::run(&[
        "ownership",
        "--working-dir",
        dir.path().to_str().unwrap(),
        "--path",
        "no_such_path/*",
    ]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.trim().is_empty() || out.contains("No files matched") || out.contains("no files found"));
}
