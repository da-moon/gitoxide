use assert_cmd::cargo::cargo_bin;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

/// Create a tiny repository with three sequential commits for testing.
fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = dir.path();
    Command::new("git").arg("init").current_dir(repo).output().unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "tester@example.com"])
        .current_dir(repo)
        .output()
        .unwrap();
    let dates = [
        "2020-01-01T00:00:00 +0000",
        "2020-01-02T00:00:00 +0000",
        "2020-01-03T00:00:00 +0000",
    ];
    for (i, date) in dates.iter().enumerate() {
        let mut f = File::create(repo.join(format!("file{i}.txt"))).unwrap();
        writeln!(f, "{i}").unwrap();
        Command::new("git")
            .args(["add", &format!("file{i}.txt")])
            .current_dir(repo)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", &format!("c{i}"), "--date", date])
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date)
            .current_dir(repo)
            .output()
            .unwrap();
    }
    dir
}

/// Path to the compiled binary under test.
fn bin() -> std::path::PathBuf {
    cargo_bin("git-productivity-analyzer")
}

#[test]
/// Verify `--ascending` flips the default descending order.
fn order_descending_and_ascending() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["frecency", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    let first = out.lines().next().unwrap();
    assert!(first.contains("file2.txt"));

    let output = Command::new(bin())
        .args(["frecency", "--working-dir", dir.path().to_str().unwrap(), "--ascending"])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    let first = out.lines().next().unwrap();
    assert!(first.contains("file0.txt"));
}

#[test]
/// Ensure path filtering and commit limiting behave correctly.
fn path_filter_and_max_commits() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args([
            "frecency",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--paths",
            "file1.txt",
            "file2.txt",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("file1.txt"));
    assert!(text.contains("file2.txt"));
    assert!(!text.contains("file0.txt"));

    let output = Command::new(bin())
        .args([
            "frecency",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--max-commits",
            "1",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.lines().count() == 1);
}

#[test]
/// Validate that JSON output is well-formed when `--json` is used.
fn json_output() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["--json", "frecency", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
}
