use std::collections::HashMap;

use tantivy::schema::{JsonObjectOptions, Schema, SchemaBuilder, FAST, STORED, TEXT};

use crate::indexing::KNOWN_DESCRIPTORS;

lazy_static::lazy_static! {
    pub static ref LIBRARY: HashMap<&'static str, tantivy::schema::Schema> = [("descriptor_v1", descriptor_v1_schema()), ("scaffold", scaffold_schema())].into_iter().collect();
}

fn descriptor_v1_schema() -> Schema {
    let mut builder = SchemaBuilder::new();
    builder.add_text_field("smiles", TEXT | STORED);
    // builder.add_json_field("descriptors", TEXT | STORED);
    for field in KNOWN_DESCRIPTORS {
        if field.starts_with("Num") || field.starts_with("lipinski") {
            builder.add_i64_field(field, FAST | STORED);
        } else {
            builder.add_f64_field(field, FAST | STORED);
        }
    }
    builder.add_bytes_field("fingerprint", FAST | STORED);

    let json_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();
    builder.add_json_field("extra_data", json_options);

    builder.build()
}

fn scaffold_schema() -> Schema {
    let mut builder = SchemaBuilder::new();
    builder.add_text_field("smiles", TEXT | STORED);
    builder.add_u64_field("id", FAST | STORED);
    builder.build()
}
