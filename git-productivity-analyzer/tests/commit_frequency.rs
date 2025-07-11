use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

mod util;

fn init_repo() -> TempDir {
    let dir = util::init_repo();
    let repo = dir.path();
    let authors = [
        ("Sebastian Thiel", "a@example.com"),
        ("Eliah Kagan", "b@example.com"),
        ("Edward Shen", "c@example.com"),
    ];
    for (name, email) in authors.iter() {
        let file = format!("{}.txt", name.replace(' ', "_"));
        File::create(repo.join(&file)).unwrap();
        Command::new("git")
            .args(["add", &file])
            .current_dir(repo)
            .output()
            .unwrap();
        Command::new("git")
            .args([
                "-c",
                &format!("user.name={}", name),
                "-c",
                &format!("user.email={}", email),
                "commit",
                "-m",
                name,
            ])
            .current_dir(repo)
            .output()
            .unwrap();
    }
    dir
}

#[test]
fn default_run() {
    let dir = init_repo();
    let output = util::run(&["commit-frequency", "--working-dir", dir.path().to_str().unwrap()]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Sebastian"));
    assert!(out.contains("Eliah"));
}

#[test]
fn author_filter() {
    let dir = init_repo();
    let output = util::run(&[
        "commit-frequency",
        "--working-dir",
        dir.path().to_str().unwrap(),
        "--author",
        "Sebastian",
    ]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Sebastian"));
    assert!(!out.contains("Eliah"));
}

#[test]
fn author_filter_no_matches() {
    let dir = init_repo();
    let output = util::run(&[
        "commit-frequency",
        "--working-dir",
        dir.path().to_str().unwrap(),
        "--author",
        "Nonexistent",
    ]);
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.trim().is_empty() || out.contains("no results") || out.contains("No commits"));
}

#[test]
fn json_output() {
    let dir = init_repo();
    let json = util::run_json(&[
        "--json",
        "commit-frequency",
        "--working-dir",
        dir.path().to_str().unwrap(),
    ]);
    assert!(json.get("commits_per_day").is_some());
    assert!(json.get("commits_per_week").is_some());
    assert!(json.get("active_days_per_author").is_some());
}
