use assert_cmd::cargo::cargo_bin;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

/// create a small git repository with three sequential commits
fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = dir.path();

    // basic repository configuration
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
    // use deterministic commit dates for predictable ordering
    let dates = [
        "2020-01-01T00:00:00 +0000",
        "2020-01-02T00:00:00 +0000",
        "2020-01-03T00:00:00 +0000",
    ];
    for (i, date) in dates.iter().enumerate() {
        // create a new file for each commit
        let mut f = File::create(repo.join(format!("file{i}.txt"))).unwrap();
        writeln!(f, "{i}").unwrap();
        Command::new("git")
            .args(["add", &format!("file{i}.txt")])
            .current_dir(repo)
            .output()
            .unwrap();
        Command::new("git")
            // commit with the given date so age weights are stable
            .args(["commit", "-m", &format!("c{i}"), "--date", date])
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date)
            .current_dir(repo)
            .output()
            .unwrap();
    }
    dir
}

/// path to the built binary under test
fn bin() -> std::path::PathBuf {
    cargo_bin("git-productivity-analyzer")
}

#[test]
fn order_descending_and_ascending() {
    let dir = init_repo();
    // default sorting should put the newest file first
    let output = Command::new(bin())
        .args(["frecency", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    let first = out.lines().next().unwrap();
    assert!(first.contains("file2.txt"));

    // ascending order flips the result
    let output = Command::new(bin())
        .args(["frecency", "--working-dir", dir.path().to_str().unwrap(), "--ascending"])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    let first = out.lines().next().unwrap();
    assert!(first.contains("file0.txt"));
}

#[test]
fn path_filter_and_max_commits() {
    let dir = init_repo();
    // restrict analysis to two paths
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

    // limit to a single commit
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
fn json_output() {
    let dir = init_repo();
    // ensure the JSON flag produces valid JSON
    let output = Command::new(bin())
        .args(["--json", "frecency", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
}
