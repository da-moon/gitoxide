use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

mod util;
const NOW: &str = "1578096000"; // 2020-01-04T00:00:00Z

/// Create a tiny repository with three sequential commits for testing.
fn init_repo() -> TempDir {
    let dir = util::init_repo();
    let repo = dir.path();
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

/// Create a repository with a merge commit to ensure merges are skipped.
fn init_repo_with_merge() -> TempDir {
    util::init_repo_with_merge()
}

#[test]
/// Verify `frecency` on an empty repository produces no output, does not panic,
/// and exits with code zero.
fn frecency_empty_repository() {
    let temp = tempfile::tempdir().unwrap();
    Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .output()
        .expect("failed to init repo");

    let output = Command::new(util::bin_path())
        .args(["frecency", "--working-dir", temp.path().to_str().unwrap(), "--now", NOW])
        .output()
        .expect("failed to run frecency");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
/// Verify `--order ascending` flips the default descending order.
fn order_descending_and_ascending() {
    let dir = init_repo();
    let output = Command::new(util::bin_path())
        .args(["frecency", "--working-dir", dir.path().to_str().unwrap(), "--now", NOW])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    let first = out.lines().next().unwrap();
    assert!(first.contains("file2.txt"));

    let output = Command::new(util::bin_path())
        .args([
            "frecency",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--order",
            "ascending",
            "--now",
            NOW,
        ])
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
    let output = Command::new(util::bin_path())
        .args([
            "frecency",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--paths",
            "file1.txt",
            "file2.txt",
            "--now",
            NOW,
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("file1.txt"));
    assert!(text.contains("file2.txt"));
    assert!(!text.contains("file0.txt"));

    let output = Command::new(util::bin_path())
        .args([
            "frecency",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--max-commits",
            "1",
            "--now",
            NOW,
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.lines().count() == 1);
}

#[test]
fn path_only() {
    let dir = init_repo();
    let output = Command::new(util::bin_path())
        .args([
            "frecency",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--path-only",
            "--now",
            NOW,
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    // each line should contain exactly one whitespace-separated field
    for line in text.lines() {
        assert_eq!(line.split_whitespace().count(), 1);
    }
}

#[test]
/// Validate that JSON output is well-formed when `--json` is used.
fn json_output() {
    let dir = init_repo();
    let output = Command::new(util::bin_path())
        .args([
            "--json",
            "frecency",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--now",
            NOW,
        ])
        .output()
        .unwrap();
    serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
}

#[test]
/// Ensure merge commits are ignored when scoring files.
fn merge_commits_skipped() {
    let dir = init_repo_with_merge();
    let output = Command::new(util::bin_path())
        .args(["frecency", "--working-dir", dir.path().to_str().unwrap(), "--now", NOW])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(text.lines().count(), 3);
    let first = text.lines().next().unwrap();
    assert!(first.contains("file2.txt"));
}
