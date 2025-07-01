use assert_cmd::cargo::cargo_bin;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;
const NOW: &str = "1578096000"; // 2020-01-04T00:00:00Z

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

/// Create a repository with a merge commit to ensure merges are skipped.
fn init_repo_with_merge() -> TempDir {
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

    // base commit
    let date1 = "2020-01-01T00:00:00 +0000";
    let mut f = File::create(repo.join("file0.txt")).unwrap();
    writeln!(f, "0").unwrap();
    Command::new("git")
        .args(["add", "file0.txt"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "c0", "--date", date1])
        .env("GIT_AUTHOR_DATE", date1)
        .env("GIT_COMMITTER_DATE", date1)
        .current_dir(repo)
        .output()
        .unwrap();

    // feature branch commit
    Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(repo)
        .output()
        .unwrap();
    let date_feature = "2020-01-03T00:00:00 +0000";
    let mut f = File::create(repo.join("file2.txt")).unwrap();
    writeln!(f, "2").unwrap();
    Command::new("git")
        .args(["add", "file2.txt"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "feature", "--date", date_feature])
        .env("GIT_AUTHOR_DATE", date_feature)
        .env("GIT_COMMITTER_DATE", date_feature)
        .current_dir(repo)
        .output()
        .unwrap();

    // main branch commit
    Command::new("git")
        .args(["checkout", "-"])
        .current_dir(repo)
        .output()
        .unwrap();
    let date_main = "2020-01-02T00:00:00 +0000";
    let mut f = File::create(repo.join("file1.txt")).unwrap();
    writeln!(f, "1").unwrap();
    Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "main", "--date", date_main])
        .env("GIT_AUTHOR_DATE", date_main)
        .env("GIT_COMMITTER_DATE", date_main)
        .current_dir(repo)
        .output()
        .unwrap();

    // merge feature branch
    Command::new("git")
        .args(["merge", "feature", "--no-ff", "-m", "merge"])
        .env("GIT_AUTHOR_DATE", "2020-01-04T00:00:00 +0000")
        .env("GIT_COMMITTER_DATE", "2020-01-04T00:00:00 +0000")
        .current_dir(repo)
        .output()
        .unwrap();
    dir
}

/// Path to the compiled binary under test.
fn bin() -> std::path::PathBuf {
    cargo_bin("git-productivity-analyzer")
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

    let output = Command::new(bin())
        .args(["frecency", "--working-dir", temp.path().to_str().unwrap(), "--now", NOW])
        .output()
        .expect("failed to run frecency");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
/// Verify `--ascending` flips the default descending order.
fn order_descending_and_ascending() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["frecency", "--working-dir", dir.path().to_str().unwrap(), "--now", NOW])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    let first = out.lines().next().unwrap();
    assert!(first.contains("file2.txt"));

    let output = Command::new(bin())
        .args([
            "frecency",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--ascending",
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
    let output = Command::new(bin())
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

    let output = Command::new(bin())
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
/// Validate that JSON output is well-formed when `--json` is used.
fn json_output() {
    let dir = init_repo();
    let output = Command::new(bin())
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
    let output = Command::new(bin())
        .args(["frecency", "--working-dir", dir.path().to_str().unwrap(), "--now", NOW])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert_eq!(text.lines().count(), 3);
    let first = text.lines().next().unwrap();
    assert!(first.contains("file2.txt"));
}
