use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

mod util;

fn init_repo() -> TempDir {
    let dir = util::init_repo();
    let repo = dir.path();
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

#[test]
fn churn_by_author() {
    let dir = init_repo();
    let output = util::run(&["churn", "--working-dir", dir.path().to_str().unwrap()]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Alice"));
    assert!(out.contains("Bob"));
}

#[test]
fn churn_per_file() {
    let dir = init_repo();
    let output = util::run(&["churn", "--working-dir", dir.path().to_str().unwrap(), "--per-file"]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("a.txt"));
    assert!(out.contains("b.txt"));
}

#[test]
fn author_filter() {
    let dir = init_repo();
    let output = util::run(&[
        "churn",
        "--working-dir",
        dir.path().to_str().unwrap(),
        "--author",
        "alice",
    ]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Alice"));
    assert!(!out.contains("Bob"));
}

#[test]
fn author_filter_case_insensitive() {
    let dir = init_repo();
    let output = util::run(&[
        "churn",
        "--working-dir",
        dir.path().to_str().unwrap(),
        "--author",
        "ALICE",
    ]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Alice"));
    assert!(!out.contains("Bob"));
}

#[test]
fn author_filter_no_matches() {
    let dir = init_repo();
    let output = util::run(&[
        "churn",
        "--working-dir",
        dir.path().to_str().unwrap(),
        "--author",
        "nonexistent",
    ]);
    assert!(
        output.status.success(),
        "Process did not exit successfully for no-match author filter"
    );
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.trim().is_empty() || out.contains("no results") || out.contains("No commits"));
}

#[test]
fn json_output() {
    let dir = init_repo();
    let json = util::run_json(&["--json", "churn", "--working-dir", dir.path().to_str().unwrap()]);
    let totals = json.get("totals").expect("missing totals");
    let map = totals.as_object().expect("totals not object");
    assert!(!map.is_empty(), "expected at least one entry");
    for entry in map.values() {
        assert!(entry.get("added").is_some());
        assert!(entry.get("removed").is_some());
    }
}
