use tantivy::schema::*;
use tantivy::{Index, IndexBuilder};

pub use tantivy::doc;

pub fn schema() -> Schema {
    let mut builder = SchemaBuilder::new();
    builder.add_text_field("smile", TEXT | STORED);
    builder.add_json_field("description", TEXT | STORED);

    builder.build()
}

pub fn index() -> eyre::Result<(Schema, Index)> {
    let schema = schema();

    let builder = IndexBuilder::new().schema(schema.clone());

    let index = builder.create_in_dir("tmp/index/")?;

    Ok((schema, index))
}
