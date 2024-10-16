use std::collections::HashMap;

use tantivy::schema::{
    JsonObjectOptions, Schema, SchemaBuilder, FAST, INDEXED, STORED, STRING, TEXT,
};

use crate::indexing::KNOWN_DESCRIPTORS;

lazy_static::lazy_static! {
    pub static ref LIBRARY: HashMap<&'static str, Schema> = [("descriptor_v1", descriptor_v1_schema())].into_iter().collect();
}

fn descriptor_v1_schema() -> Schema {
    let mut builder = SchemaBuilder::new();
    builder.add_text_field("smiles", STRING | STORED);
    for field in KNOWN_DESCRIPTORS {
        if field.starts_with("Num") || field.starts_with("lipinski") {
            builder.add_i64_field(field, INDEXED);
        } else if field == "exactmw" {
            builder.add_f64_field(field, INDEXED | FAST);
        } else {
            builder.add_f64_field(field, INDEXED);
        }
    }
    builder.add_bytes_field("pattern_fingerprint", STORED);
    builder.add_bytes_field("morgan_fingerprint", STORED);

    let extra_data_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT | STORED).set_expand_dots_enabled();
    builder.add_json_field("extra_data", extra_data_options);

    let other_descriptors_options: JsonObjectOptions =
        JsonObjectOptions::from(TEXT).set_expand_dots_enabled();
    builder.add_json_field("other_descriptors", other_descriptors_options.clone());

    builder.build()
}
