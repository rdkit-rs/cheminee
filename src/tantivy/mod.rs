use std::path::Path;
use tantivy::directory::MmapDirectory;
use tantivy::schema::*;
use tantivy::{Index, IndexBuilder};

pub use tantivy::doc;

pub fn schema() -> Schema {
    let mut builder = SchemaBuilder::new();
    builder.add_text_field("smile", TEXT | STORED);
    builder.add_json_field("descriptors", TEXT | STORED);

    builder.build()
}

pub fn create_index(p: impl AsRef<Path>) -> eyre::Result<(Schema, Index)> {
    let schema = schema();

    let builder = IndexBuilder::new().schema(schema.clone());

    let index = builder.create_in_dir(p)?;

    Ok((schema, index))
}

pub fn open_index(p: impl AsRef<Path>) -> eyre::Result<Index> {
    let directory = MmapDirectory::open(p)?;

    let index = Index::open(directory)?;

    Ok(index)
}
