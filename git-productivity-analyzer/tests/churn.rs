use assert_cmd::cargo::cargo_bin;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = dir.path();
    Command::new("git").arg("init").current_dir(repo).output().unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(repo)
        .output()
        .unwrap();
    // first commit by Alice
    File::create(repo.join("a.txt")).unwrap();
    Command::new("git")
        .args(["add", "a.txt"])
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
            "a1",
        ])
        .current_dir(repo)
        .output()
        .unwrap();
    // second commit by Bob modifies a
    let mut f = File::create(repo.join("a.txt")).unwrap();
    writeln!(f, "more").unwrap();
    Command::new("git")
        .args(["add", "a.txt"])
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
            "b1",
        ])
        .current_dir(repo)
        .output()
        .unwrap();
    // third commit by Alice adds new file
    File::create(repo.join("b.txt")).unwrap();
    Command::new("git")
        .args(["add", "b.txt"])
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
            "a2",
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
fn churn_by_author() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["churn", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Alice"));
    assert!(out.contains("Bob"));
}

#[test]
fn churn_per_file() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["churn", "--working-dir", dir.path().to_str().unwrap(), "--per-file"])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("a.txt"));
    assert!(out.contains("b.txt"));
}
