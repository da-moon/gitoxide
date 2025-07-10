use assert_cmd::cargo::cargo_bin;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn git(repo: &Path, args: &[&str]) {
    Command::new("git").args(args).current_dir(repo).output().unwrap();
}

fn git_env(repo: &Path, args: &[&str], env: &[(&str, &str)]) {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(repo);
    for (key, value) in env {
        cmd.env(key, value);
    }
    cmd.output().unwrap();
}

fn config_user(repo: &Path, name: &str, email: &str) {
    git(repo, &["config", "user.name", name]);
    git(repo, &["config", "user.email", email]);
}

fn commit_file(repo: &Path, path: &str, content: &str, msg: &str, date: &str) {
    std::fs::write(repo.join(path), content).unwrap();
    git(repo, &["add", path]);
    git_env(
        repo,
        &["commit", "-m", msg, "--date", date],
        &[("GIT_AUTHOR_DATE", date), ("GIT_COMMITTER_DATE", date)],
    );
}

pub fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = dir.path();
    Command::new("git").arg("init").current_dir(repo).output().unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(repo)
        .output()
        .unwrap();
    dir
}

pub fn init_repo_with_merge() -> TempDir {
    let dir = init_repo();
    let repo = dir.path();
    config_user(repo, "Tester", "tester@example.com");
    commit_file(repo, "file0.txt", "0", "c0", "2020-01-01T00:00:00 +0000");
    git(repo, &["checkout", "-b", "feature"]);
    commit_file(repo, "file2.txt", "2", "feature", "2020-01-03T00:00:00 +0000");
    git(repo, &["checkout", "-"]);
    commit_file(repo, "file1.txt", "1", "main", "2020-01-02T00:00:00 +0000");
    git_env(
        repo,
        &["merge", "feature", "--no-ff", "-m", "merge"],
        &[
            ("GIT_AUTHOR_DATE", "2020-01-04T00:00:00 +0000"),
            ("GIT_COMMITTER_DATE", "2020-01-04T00:00:00 +0000"),
        ],
    );
    dir
}

pub fn bin_path() -> std::path::PathBuf {
    cargo_bin("git-productivity-analyzer")
}
