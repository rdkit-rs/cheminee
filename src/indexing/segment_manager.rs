use tantivy::{Index, TantivyDocument};

pub struct SegmentManager {}

impl SegmentManager {
    pub fn merge(&self, index: &Index) -> eyre::Result<()> {
        let segments = index.searchable_segment_ids()?;

        let mut writer = index.writer::<TantivyDocument>(64 * 1024 * 1024)?;

        let _merge_operation = writer.merge(&segments).wait()?;

        Ok(())
    }

    // Remember to reopen the index for garbage collection, it what the tantivy-cli does
    pub fn garbage_collect(&self, index: &Index) -> eyre::Result<()> {
        index
            .writer_with_num_threads::<tantivy::TantivyDocument>(1, 64 * 1024 * 1024)?
            .garbage_collect_files()
            .wait()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use tantivy::TantivyDocument;

    use crate::indexing::{index_manager::IndexManager, segment_manager::SegmentManager};

    #[test]
    pub fn test_merge() {
        let schema = crate::schema::LIBRARY.get("descriptor_v1").unwrap();

        let index_manager = IndexManager::new(temp_dir(), true).unwrap();
        let index = index_manager.create("meep", schema, true, None).unwrap();

        let segments = index.searchable_segments().unwrap();
        assert_eq!(segments.len(), 0);

        {
            // create a scope where the writer can be dropped
            let mut writer = index.writer::<TantivyDocument>(16 * 1024 * 1024).unwrap();
            let smiles = schema.get_field("smiles").unwrap();
            let fingerprint = schema.get_field("fingerprint").unwrap();

            writer
                .add_document(tantivy::doc!(
                    smiles => "C",
                    fingerprint => vec![10]
                ))
                .unwrap();
            writer.commit().unwrap();

            writer
                .add_document(tantivy::doc!(
                    smiles => "C",
                    fingerprint => vec![10]
                ))
                .unwrap();
            writer.commit().unwrap();
        }

        let segments = index.searchable_segments().unwrap();
        assert_eq!(segments.len(), 2);

        let segment_manager = SegmentManager {};
        segment_manager.merge(&index).unwrap();

        let segments = index.searchable_segments().unwrap();
        assert_eq!(segments.len(), 1);
    }
}
