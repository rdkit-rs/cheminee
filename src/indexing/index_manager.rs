use std::path::PathBuf;
use std::sync::Arc;

use tantivy::{directory::MmapDirectory, schema::Schema, IndexBuilder, TantivyError};
use tokio::sync::RwLock;

#[derive(Clone)]
#[allow(dead_code)]
pub struct IndexManager {
    storage_dir: PathBuf,
    lock: Arc<RwLock<()>>,
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
            storage_dir,
            lock: Arc::new(RwLock::new(())),
        })
    }

    pub fn create(&self, name: &str, schema: &Schema, force: bool) -> eyre::Result<tantivy::Index> {
        let builder = Self::build_builder(schema)?;
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

                    let builder = Self::build_builder(schema)?;
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

    pub fn build_builder(schema: &Schema) -> eyre::Result<tantivy::IndexBuilder> {
        let builder = IndexBuilder::new().schema(schema.clone());
        Ok(builder)
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

    pub fn list(&self) -> eyre::Result<Vec<String>> {
        let storage_dir = format!("{}{}", self.storage_dir.display(), "/");

        let paths = std::fs::read_dir(&self.storage_dir)?;

        let paths: Vec<_> = paths
            .into_iter()
            .map(|p| format!("{}", p.unwrap().path().display()).replace(&storage_dir, ""))
            .collect();

        Ok(paths)
    }
}
