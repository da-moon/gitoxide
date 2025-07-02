use std::process::Command;
use tempfile::TempDir;

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

    let date1 = "2020-01-01T00:00:00 +0000";
    std::fs::write(repo.join("file0.txt"), "0").unwrap();
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

    Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(repo)
        .output()
        .unwrap();
    let date_feature = "2020-01-03T00:00:00 +0000";
    std::fs::write(repo.join("file2.txt"), "2").unwrap();
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

    Command::new("git")
        .args(["checkout", "-"])
        .current_dir(repo)
        .output()
        .unwrap();
    let date_main = "2020-01-02T00:00:00 +0000";
    std::fs::write(repo.join("file1.txt"), "1").unwrap();
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

    Command::new("git")
        .args(["merge", "feature", "--no-ff", "-m", "merge"])
        .env("GIT_AUTHOR_DATE", "2020-01-04T00:00:00 +0000")
        .env("GIT_COMMITTER_DATE", "2020-01-04T00:00:00 +0000")
        .current_dir(repo)
        .output()
        .unwrap();

    dir
}
