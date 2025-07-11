use assert_cmd::cargo::cargo_bin;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

mod util;

fn init_repo() -> TempDir {
    let dir = util::init_repo();
    let repo = dir.path();
    for i in 1..=3 {
        let mut f = File::create(repo.join(format!("f{i}"))).unwrap();
        for _ in 0..i {
            writeln!(f, "{i}").unwrap();
        }
        Command::new("git")
            .args(["add", &format!("f{i}")])
            .current_dir(repo)
            .output()
            .unwrap();
        Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                &format!("c{i}"),
            ])
            .current_dir(repo)
            .output()
            .unwrap();
    }
    dir
}

fn bin() -> std::path::PathBuf {
    cargo_bin("git-productivity-analyzer")
}

#[test]
fn default_run() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["commit-size", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("files per commit"));
}

#[test]
fn percentiles() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args([
            "--json",
            "commit-size",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--percentiles",
            "50,100",
        ])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let expected = serde_json::json!([[50.0, 2], [100.0, 3]]);
    assert_eq!(v.get("line_percentiles").unwrap(), &expected);
}

#[test]
fn empty_repository() {
    let dir = util::init_repo();
    let output = Command::new(bin())
        .args(["commit-size", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    
    // Assert the process exits with a specific code (0 for success, non-zero for expected failure)
    assert!(output.status.success() || output.status.code() == Some(1));
    
    // Assert stderr does not contain unexpected errors or panics
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panic"));
    assert!(!stderr.contains("fatal"));
    assert!(!stderr.contains("internal error"));
    
    // The command may fail on an empty repository but should not panic
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty() || stdout.contains("No commits") || stdout.contains("0"));
}
