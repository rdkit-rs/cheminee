use std::path::Path;
use tantivy::directory::MmapDirectory;
use tantivy::schema::*;
use tantivy::{Index, IndexBuilder, TantivyError};

pub use tantivy::doc;

pub fn schema() -> Schema {
    let mut builder = SchemaBuilder::new();
    builder.add_text_field("smile", TEXT | STORED);
    builder.add_json_field("descriptors", TEXT | STORED);
    builder.add_bytes_field("fingerprint", FAST);

    builder.build()
}

pub fn create_or_reset_index(p: impl AsRef<Path>) -> eyre::Result<(Schema, Index)> {
    let schema = schema();

    let builder = IndexBuilder::new().schema(schema.clone());

    let index = match builder.create_in_dir(&p) {
        Ok(index) => index,
        Err(TantivyError::IndexAlreadyExists) => {
            std::fs::remove_dir_all(&p)?;
            std::fs::create_dir(&p)?;
            let builder = IndexBuilder::new().schema(schema.clone());
            builder.create_in_dir(&p)?
        }
        Err(e) => return Err(eyre::eyre!("unhandled error: {:?}", e)),
    };

    Ok((schema, index))
}

pub fn open_index(p: impl AsRef<Path>) -> eyre::Result<Index> {
    let directory = MmapDirectory::open(p)?;

    let index = Index::open(directory)?;

    Ok(index)
}
