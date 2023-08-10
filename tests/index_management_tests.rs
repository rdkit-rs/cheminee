use cheminee::indexing::index_manager::IndexManager;

#[test]
fn test_index_management() -> eyre::Result<()> {
    let temp_index_dir = tempdir::TempDir::new("/tmp")?;
    let manager = IndexManager::new(temp_index_dir.path(), true)?;

    assert!(manager.exists("foo")?.is_none());

    Ok(())
}
