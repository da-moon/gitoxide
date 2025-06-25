mod common;
use common::{bin, init_repo};
use std::process::Command;

#[test]
fn create_list_delete() {
    let dir = init_repo();
    let repo = dir.path();
    Command::new(bin())
        .args(["--repo", repo.to_str().unwrap(), "create", "test"])
        .status()
        .unwrap();
    let output = Command::new(bin())
        .args(["--repo", repo.to_str().unwrap(), "list"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("test"));
    Command::new(bin())
        .args(["--repo", repo.to_str().unwrap(), "delete", "test"])
        .status()
        .unwrap();
    let output = Command::new(bin())
        .args(["--repo", repo.to_str().unwrap(), "list"])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&output.stdout).contains("test"));
}
