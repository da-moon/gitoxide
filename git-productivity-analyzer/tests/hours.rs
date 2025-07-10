use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

mod util;

fn init_repo() -> TempDir {
    let dir = util::init_repo();
    let repo = dir.path();
    Command::new("git")
        .args(["config", "user.name", "Sebastian Thiel"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "git@example.com"])
        .current_dir(repo)
        .output()
        .unwrap();
    let dates = [
        "2020-01-01T00:00:00 +0000",
        "2020-01-02T00:00:00 +0000",
        "2020-01-03T00:00:00 +0000",
    ];
    for (i, date) in dates.iter().enumerate() {
        let mut f = File::create(repo.join(format!("file{i}"))).unwrap();
        writeln!(f, "{i}").unwrap();
        Command::new("git")
            .args(["add", &format!("file{i}")])
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
        Command::new("git")
            .args(["tag", &format!("t{i}")])
            .current_dir(repo)
            .output()
            .unwrap();
    }
    dir
}

#[test]
fn default_run() {
    let dir = init_repo();
    let output = util::run(&["hours", "--working-dir", dir.path().to_str().unwrap()]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("total commits = 3"));
}

#[test]
fn show_pii() {
    let dir = init_repo();
    let output = util::run(&["hours", "--working-dir", dir.path().to_str().unwrap(), "--show-pii"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Sebastian Thiel"));
}

#[test]
fn file_stats() {
    let dir = init_repo();
    let output = util::run(&["hours", "--working-dir", dir.path().to_str().unwrap(), "--file-stats"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("total files added"));
}

#[test]
fn line_stats() {
    let dir = init_repo();
    let output = util::run(&["hours", "--working-dir", dir.path().to_str().unwrap(), "--line-stats"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("total lines added"));
}

#[test]
fn json_output() {
    let dir = init_repo();
    let json = util::run_json(&["--json", "hours", "--working-dir", dir.path().to_str().unwrap()]);
    assert!(json.get("total_hours").is_some());
    assert!(json.get("total_commits").is_some());
}

#[test]
fn since_until() {
    let dir = init_repo();
    let output = Command::new("git")
        .args(["rev-parse", "t1"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let tag1 = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let output_since = util::run(&[
        "--since",
        &tag1,
        "--json",
        "hours",
        "--working-dir",
        dir.path().to_str().unwrap(),
    ]);
    assert!(output_since.status.success());
    let json_since: serde_json::Value = serde_json::from_slice(&output_since.stdout).unwrap();
    assert!(json_since.get("total_commits").is_some());

    let output_until = util::run(&[
        "--until",
        &tag1,
        "--json",
        "hours",
        "--working-dir",
        dir.path().to_str().unwrap(),
    ]);
    assert!(output_until.status.success());
    let json_until: serde_json::Value = serde_json::from_slice(&output_until.stdout).unwrap();
    assert!(json_until.get("total_commits").is_some());
}
