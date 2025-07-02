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

fn bin() -> std::path::PathBuf {
    cargo_bin("git-productivity-analyzer")
}

#[test]
fn default_run() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["time-of-day", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn author_filter() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args([
            "time-of-day",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--author",
            "A",
        ])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(!out.is_empty());
}
