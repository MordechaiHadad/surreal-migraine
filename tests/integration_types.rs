use eyre::Result;
use tempfile::tempdir;
use surreal_migraine::types::{DiskSource, MigrationKind, MigrationSource};

#[test]
fn disk_source_list_and_gets() -> Result<()> {
    let tmpdir = tempdir()?;
    let tmp = tmpdir.path().to_path_buf();

    // File migration
    let file_path = tmp.join("001_init.surql");
    std::fs::write(&file_path, "CREATE TABLE test;")?;

    // Paired migration
    let paired_dir = tmp.join("002_add_user");
    std::fs::create_dir_all(&paired_dir)?;
    std::fs::write(paired_dir.join("up.surql"), "CREATE TABLE user;")?;
    std::fs::write(paired_dir.join("down.surql"), "DROP TABLE user;")?;

    let ds = DiskSource::new(&tmp);
    let list = ds.list()?;

    assert_eq!(list.len(), 2);
    assert_eq!(list[0].name, "001_init.surql");
    match list[0].kind {
        MigrationKind::File => {}
        _ => panic!("expected file migration"),
    }

    assert_eq!(list[1].name, "002_add_user");
    match list[1].kind {
        MigrationKind::Paired => {}
        _ => panic!("expected paired migration"),
    }

    let up0 = ds.get_up(&list[0])?;
    assert_eq!(up0, "CREATE TABLE test;");

    let up1 = ds.get_up(&list[1])?;
    assert_eq!(up1, "CREATE TABLE user;");

    let down1 = ds.get_down(&list[1])?;
    assert_eq!(down1, Some("DROP TABLE user;".to_string()));

    let down0 = ds.get_down(&list[0])?;
    assert_eq!(down0, None);

    Ok(())
}
