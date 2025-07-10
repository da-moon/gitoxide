use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

mod util;

fn init_repo() -> TempDir {
    let dir = util::init_repo();
    let repo = dir.path();
    let commits = [
        ("Alice", "a@example.com", "2020-01-01T00:00:00 +0000"),
        ("Alice", "a@example.com", "2020-01-02T00:00:00 +0000"),
        ("Alice", "a@example.com", "2020-01-03T00:00:00 +0000"),
        ("Bob", "b@example.com", "2020-01-01T00:00:00 +0000"),
        ("Bob", "b@example.com", "2020-01-04T00:00:00 +0000"),
        ("Alice", "a@example.com", "2020-01-05T00:00:00 +0000"),
    ];
    for (i, (name, email, date)) in commits.iter().enumerate() {
        let mut f = File::create(repo.join(format!("f{i}"))).unwrap();
        writeln!(f, "{i}").unwrap();
        Command::new("git")
            .args(["add", &format!("f{i}")])
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
                &format!("c{i}"),
                "--date",
                date,
            ])
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date)
            .current_dir(repo)
            .output()
            .unwrap();
    }
    dir
}

#[test]
fn streaks_default() {
    let dir = init_repo();
    let output = Command::new(util::bin_path())
        .args(["streaks", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Alice"));
    assert!(out.contains("Bob"));
}

#[test]
fn streaks_filtered() {
    let dir = init_repo();
    let output = Command::new(util::bin_path())
        .args([
            "streaks",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--author",
            "Alice",
        ])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Alice"));
    assert!(!out.contains("Bob"));
}

#[test]
fn json_output() {
    let dir = init_repo();
    let output = Command::new(util::bin_path())
        .args(["--json", "streaks", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let obj = json.as_object().expect("expected JSON object");
    if let Some(val) = obj.values().next() {
        assert!(val.is_number(), "streak value should be numeric");
    }
}
