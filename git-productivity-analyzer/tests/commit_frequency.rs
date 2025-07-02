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

fn bin() -> std::path::PathBuf {
    cargo_bin("git-productivity-analyzer")
}

#[test]
fn default_run() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args(["commit-frequency", "--working-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Sebastian"));
    assert!(out.contains("Eliah"));
}

#[test]
fn author_filter() {
    let dir = init_repo();
    let output = Command::new(bin())
        .args([
            "commit-frequency",
            "--working-dir",
            dir.path().to_str().unwrap(),
            "--author",
            "Sebastian",
        ])
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(out.contains("Sebastian"));
    assert!(!out.contains("Eliah"));
}
