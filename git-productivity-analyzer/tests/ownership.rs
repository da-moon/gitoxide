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
    let output = Command::new(bin())
        .args(["--json", "ownership", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    
    use util::json_test_helpers::*;
    
    // Validate the JSON structure has expected directories
    let obj = assert_json_object(&v, "Ownership output");
    let expected_dirs = &[".", "src", "docs"];
    
    for dir in expected_dirs {
        assert_contains_key(obj, dir, "Ownership output");
    }
    
    // Assert that the root directory has the expected structure
    let root = assert_json_object(assert_contains_key(obj, ".", "Ownership output"), "Root directory");
    assert_contains_key(root, "Alice <a@example.com>", "Root directory");
    
    // Assert the src directory has Alice's ownership
    let src = assert_json_object(assert_contains_key(obj, "src", "Ownership output"), "Src directory");
    let alice_src = assert_number(assert_contains_key(src, "Alice <a@example.com>", "Src directory"), "Alice's src ownership");
    assert_positive(alice_src, "Alice's ownership percentage in src");
    
    // Assert the docs directory has Bob's ownership
    let docs = assert_json_object(assert_contains_key(obj, "docs", "Ownership output"), "Docs directory");
    let bob_docs = assert_number(assert_contains_key(docs, "Bob <b@example.com>", "Docs directory"), "Bob's docs ownership");
    assert_positive(bob_docs, "Bob's ownership percentage in docs");
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
