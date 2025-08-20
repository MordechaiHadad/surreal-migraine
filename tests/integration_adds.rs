use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use tempfile::tempdir;

#[test]
fn numeric_add_creates_000() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("smg").unwrap();
    cmd.args([
        "add",
        "init_migration",
        "--dir",
        dir.path().to_str().unwrap(),
    ]);
    cmd.assert().success();

    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert!(
        entries
            .iter()
            .any(|n| n.to_string_lossy().starts_with("000_init_migration"))
    );
}

#[test]
fn temporal_add_creates_timestamped() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("smg").unwrap();
    cmd.args([
        "add",
        "create_users",
        "--temporal",
        "--dir",
        dir.path().to_str().unwrap(),
    ]);
    cmd.assert().success();

    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert!(
        entries
            .iter()
            .any(|n| n.to_string_lossy().contains("create_users"))
    );
}

#[test]
fn numeric_collision_increments() {
    let dir = tempdir().unwrap();
    // precreate 000 and 001
    File::create(dir.path().join("000_foo.surql")).unwrap();
    File::create(dir.path().join("001_bar.surql")).unwrap();

    let mut cmd = Command::cargo_bin("smg").unwrap();
    cmd.args(["add", "new_mig", "--dir", dir.path().to_str().unwrap()]);
    cmd.assert().success();

    let entries: Vec<String> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert!(entries.iter().any(|n| n.starts_with("002_")));
}

#[test]
fn invalid_name_errors() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("smg").unwrap();
    // name with only invalid chars -> sanitized empty
    cmd.args(["add", "\"<>|", "--dir", dir.path().to_str().unwrap()]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("sanitized"));
}
