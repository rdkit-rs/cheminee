use crate::indexing::KNOWN_DESCRIPTORS;
use std::collections::HashMap;
use tantivy::schema::{Schema, SchemaBuilder, FAST, STORED, TEXT};

lazy_static::lazy_static! {
    pub static ref LIBRARY: HashMap<&'static str, tantivy::schema::Schema> = [("descriptor_v1", descriptor_v1_schema())].into_iter().collect();
}

fn descriptor_v1_schema() -> Schema {
    let mut builder = SchemaBuilder::new();
    builder.add_text_field("smile", TEXT | STORED);
    // builder.add_json_field("descriptors", TEXT | STORED);
    for field in KNOWN_DESCRIPTORS {
        if field.starts_with("Num") || field.starts_with("lipinski") {
            builder.add_i64_field(field, FAST | STORED);
        } else {
            builder.add_f64_field(field, FAST | STORED);
        }
    }
    builder.add_bytes_field("fingerprint", FAST | STORED);

    builder.build()
}
