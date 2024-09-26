use cheminee::indexing::index_manager::IndexManager;

#[test]
fn test_missing_index() -> eyre::Result<()> {
    let temp_index_dir = tempdir::TempDir::new("cheminee-tests")?;
    let manager = IndexManager::new(temp_index_dir.path(), true)?;

    assert!(manager.exists("foo")?.is_none());

    Ok(())
}

#[test]
fn test_creating_index() -> eyre::Result<()> {
    let temp_index_dir = tempdir::TempDir::new("cheminee-tests")?;
    let manager = IndexManager::new(temp_index_dir.path(), true)?;

    let schema = cheminee::schema::LIBRARY.get("descriptor_v1").unwrap();
    let index = manager.create("test-index", schema, false, None)?;

    assert_eq!(&index.schema(), schema);

    Ok(())
}
