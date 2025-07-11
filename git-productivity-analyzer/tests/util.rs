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

pub fn run(args: &[&str]) -> std::process::Output {
    use assert_cmd::cargo::cargo_bin;
    Command::new(cargo_bin("git-productivity-analyzer"))
        .args(args)
        .output()
        .unwrap()
}

/// Helper functions for JSON test validation to reduce boilerplate
pub mod json_test_helpers {
    use serde_json::Value;
    
    /// Assert that a JSON value is an object and return it
    pub fn assert_json_object(value: &Value, context: &str) -> &serde_json::Map<String, Value> {
        value.as_object().unwrap_or_else(|| panic!("{} should be a JSON object", context))
    }
    
    /// Assert that a JSON value is an array and return it
    pub fn assert_json_array(value: &Value, context: &str) -> &Vec<Value> {
        value.as_array().unwrap_or_else(|| panic!("{} should be a JSON array", context))
    }
    
    /// Assert that a JSON object contains a key and return the value
    pub fn assert_contains_key<'a>(obj: &'a serde_json::Map<String, Value>, key: &str, context: &str) -> &'a Value {
        obj.get(key).unwrap_or_else(|| panic!("{} should contain key '{}'", context, key))
    }
    
    /// Assert that a JSON value is a number and return it as f64
    pub fn assert_number(value: &Value, context: &str) -> f64 {
        value.as_f64().unwrap_or_else(|| panic!("{} should be a number", context))
    }
    
    /// Assert that a JSON value is a u64 and return it
    pub fn assert_u64(value: &Value, context: &str) -> u64 {
        value.as_u64().unwrap_or_else(|| panic!("{} should be a u64", context))
    }
    
    /// Assert that a JSON value is a string and return it
    pub fn assert_string(value: &Value, context: &str) -> &str {
        value.as_str().unwrap_or_else(|| panic!("{} should be a string", context))
    }
    
    /// Assert that a number is positive
    pub fn assert_positive(value: f64, context: &str) {
        assert!(value > 0.0, "{} should be positive, got {}", context, value);
    }
    
    /// Assert that an array has the expected length
    pub fn assert_array_length(arr: &[Value], expected_len: usize, context: &str) {
        assert_eq!(arr.len(), expected_len, "{} should have {} elements, got {}", context, expected_len, arr.len());
    }
    
    /// Assert that values in an array are in descending order
    pub fn assert_descending_order(values: &[f64], context: &str) {
        for i in 1..values.len() {
            assert!(values[i-1] >= values[i], 
                   "{} should be in descending order: {} >= {} at positions {} and {}", 
                   context, values[i-1], values[i], i-1, i);
        }
    }
    
    /// Validate common JSON structure for analytics output
    pub fn validate_analytics_json(json: &Value, expected_fields: &[&str]) -> &serde_json::Map<String, Value> {
        let obj = assert_json_object(json, "Analytics output");
        
        // Check that all expected fields are present
        for field in expected_fields {
            assert_contains_key(obj, field, "Analytics output");
        }
        
        obj
    }
    
    /// Validate an array of objects where each object has required fields
    pub fn validate_object_array(json: &Value, required_fields: &[&str], context: &str) -> &Vec<Value> {
        let arr = assert_json_array(json, context);
        
        for (i, entry) in arr.iter().enumerate() {
            let obj = assert_json_object(entry, &format!("{} entry {}", context, i));
            
            for field in required_fields {
                assert_contains_key(obj, field, &format!("{} entry {}", context, i));
            }
        }
        
        arr
    }
}
