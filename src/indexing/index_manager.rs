use std::path::PathBuf;
use tantivy::directory::MmapDirectory;
use tantivy::{schema::Schema, IndexBuilder, TantivyError};

pub struct IndexManager {
    storage_dir: PathBuf,
}

impl IndexManager {
    pub fn new<T>(storage_dir: T, create_storage_dir_if_missing: bool) -> eyre::Result<Self>
    where
        T: Into<PathBuf>,
    {
        let storage_dir = storage_dir.into();
        if storage_dir.exists() && !storage_dir.is_dir() {
            return Err(eyre::eyre!(
                "{:?} exists but it is not a directory",
                storage_dir
            ));
        } else if !storage_dir.exists() && create_storage_dir_if_missing {
            std::fs::create_dir_all(&storage_dir)?;
        }

        Ok(Self {
            storage_dir: storage_dir.into(),
        })
    }

    pub fn create(&self, name: &str, schema: &Schema, force: bool) -> eyre::Result<tantivy::Index> {
        let builder = IndexBuilder::new().schema(schema.clone());
        let index_path = self.storage_dir.join(name);

        if !index_path.exists() {
            std::fs::create_dir_all(&index_path)?;
        }

        let index = match builder.create_in_dir(&index_path) {
            Ok(index) => index,
            Err(TantivyError::IndexAlreadyExists) => {
                if force {
                    std::fs::remove_dir_all(&index_path)?;
                    std::fs::create_dir(&index_path)?;
                    let builder = IndexBuilder::new().schema(schema.clone());
                    builder.create_in_dir(&index_path)?
                } else {
                    return Err(eyre::eyre!(
                        "index already exists and force reset option not set"
                    ));
                }
            }
            Err(e) => return Err(eyre::eyre!("unhandled error: {:?}", e)),
        };

        Ok(index)
    }

    pub fn exists(&self, name: &str) -> eyre::Result<Option<tantivy::schema::Schema>> {
        let index_path = self.storage_dir.join(name);

        if index_path.exists() && index_path.is_dir() {
            let mmap_directory = MmapDirectory::open(index_path)?;
            let index = tantivy::Index::open(mmap_directory)?;

            Ok(Some(index.schema()))
        } else {
            Ok(None)
        }
    }

    pub fn open(&self, name: &str) -> eyre::Result<tantivy::Index> {
        let index_path = self.storage_dir.join(name);

        if !index_path.exists() {
            return Err(eyre::eyre!("{:?} path does not exist", index_path));
        }

        let mmap_directory = MmapDirectory::open(index_path)?;
        let index = tantivy::Index::open(mmap_directory)?;

        Ok(index)
    }

    pub fn delete(&self, name: &str) -> eyre::Result<()> {
        let index_path = self.storage_dir.join(name);

        if !index_path.exists() {
            return Err(eyre::eyre!("{:?} path does not exist", index_path));
        }

        std::fs::remove_dir_all(&index_path)?;

        Ok(())
    }

    pub fn list(&self) -> eyre::Result<Vec<PathBuf>> {
        let paths = std::fs::read_dir(&self.storage_dir)?;

        let paths: Vec<PathBuf> = paths.into_iter().map(|p| p.unwrap().path()).collect();

        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use crate::indexing::index_manager::IndexManager;

    #[test]
    fn index_manager() -> eyre::Result<()> {
        let index_manager = IndexManager::new("/tmp/xavier", true)?;

        let schema = crate::schema::LIBRARY.get("descriptor_v1").unwrap();

        let _index = index_manager.create("structure-search", schema, true)?;

        let _index = index_manager.open("structure-search")?;

        assert!(index_manager.exists("structure-search").unwrap().is_some());

        let index_paths = index_manager.list()?;
        assert_eq!(index_paths.len(), 1);

        let _ = index_manager.delete("structure-search");
        assert!(index_manager.exists("structure-search").unwrap().is_none());

        Ok(())
    }
}
