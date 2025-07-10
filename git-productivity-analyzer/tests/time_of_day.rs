use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

mod util;

fn init_repo() -> TempDir {
    let dir = util::init_repo();
    let repo = dir.path();
    let commits = [
        ("A", "2020-01-01T00:00:00 +0000"),
        ("B", "2020-01-01T12:00:00 +0000"),
        ("C", "2020-01-01T23:00:00 +0000"),
    ];
    for (i, (name, date)) in commits.iter().enumerate() {
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
                &format!("user.email={}@example.com", name.to_lowercase()),
                "commit",
                "-m",
                name,
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
fn default_run() {
    let dir = init_repo();
    let output = Command::new(util::bin_path())
        .args(["time-of-day", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn author_filter() {
    let dir = init_repo();
    let output = Command::new(util::bin_path())
        .args([
            "time-of-day",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--author",
            "a@example.com",
        ])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = out.lines().collect();
    assert_eq!(lines.len(), 24);
    assert!(lines.iter().any(|l| l.starts_with("00-00") && l.ends_with("1")));
    let total: u32 = lines
        .iter()
        .filter_map(|l| l.split_whitespace().nth(1))
        .filter_map(|v| v.parse::<u32>().ok())
        .sum();
    assert_eq!(total, 1);
}

#[test]
fn json_output() {
    let dir = init_repo();
    let output = Command::new(util::bin_path())
        .args(["--json", "time-of-day", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let bins = json.get("bins").expect("missing bins");
    assert!(bins.is_array(), "bins should be an array");
}
