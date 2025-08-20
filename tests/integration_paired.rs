use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn default_add_creates_paired_folder() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("smg").unwrap();
    cmd.args([
        "add",
        "create_users",
        "--dir",
        dir.path().to_str().unwrap(),
    ]);
    cmd.assert().success();

    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();

    // expect a folder starting with 000_create_users
    let found = entries.iter().any(|n| {
        let s = n.to_string_lossy();
        s.starts_with("000_create_users")
    });
    assert!(found, "paired folder was not created");

    // ensure up.surql and down.surql exist inside folder
    let folder = dir.path().join("000_create_users");
    assert!(folder.join("up.surql").exists());
    assert!(folder.join("down.surql").exists());
}

#[test]
fn single_flag_creates_single_file() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("smg").unwrap();
    cmd.args([
        "add",
        "create_users",
        "--single",
        "--dir",
        dir.path().to_str().unwrap(),
    ]);
    cmd.assert().success();

    let entries: Vec<String> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert!(entries.iter().any(|n| n.starts_with("000_create_users.surql")));
}
