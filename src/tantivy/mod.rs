use std::path::Path;
use std::collections::HashMap;
use tantivy::directory::MmapDirectory;
use tantivy::schema::*;
use tantivy::{Index, IndexBuilder, Searcher, TantivyError, DocAddress};
use tantivy::collector::TopDocs;

pub use tantivy::doc;
use tantivy::query::QueryParser;

// pub const KNOWN_DESCRIPTORS: [&str; 2] = ["CrippenClogP", "CrippenMR"];
pub const KNOWN_DESCRIPTORS: [&str; 43] = [
    "CrippenClogP",
    "CrippenMR",
    "FractionCSP3",
    "NumAliphaticHeterocycles",
    "NumAliphaticRings",
    "NumAmideBonds",
    "NumAromaticHeterocycles",
    "NumAromaticRings",
    "NumAtomStereoCenters",
    "NumAtoms",
    "NumBridgeheadAtoms",
    "NumHBA",
    "NumHBD",
    "NumHeavyAtoms",
    "NumHeteroatoms",
    "NumHeterocycles",
    "NumRings",
    "NumRotatableBonds",
    "NumSaturatedHeterocycles",
    "NumSaturatedRings",
    "NumSpiroAtoms",
    "NumUnspecifiedAtomStereoCenters",
    "Phi",
    "amw",
    "chi0n",
    "chi0v",
    "chi1n",
    "chi1v",
    "chi2n",
    "chi2v",
    "chi3n",
    "chi3v",
    "chi4n",
    "chi4v",
    "exactmw",
    "hallKierAlpha",
    "kappa1",
    "kappa2",
    "kappa3",
    "labuteASA",
    "lipinskiHBA",
    "lipinskiHBD",
    "tpsa",
];

pub fn schema() -> Schema {
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

// pub fn basic_search(query_parser: &QueryParser, searcher: &Searcher, schema: &Schema, query: &String) -> eyre::Result<Vec<(DocAddress, HashMap<str, Value>)>> {
pub fn basic_search(query_parser: &QueryParser, searcher: &Searcher, schema: &Schema, query: &String) -> eyre::Result<()> {
    let query = query_parser.parse_query(query)?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    let docs = top_docs
        .into_iter()
        .map(|(_score, doc_addr)| {
            let doc = searcher.doc(doc_addr).unwrap();
            let field_values = doc.field_values();
            let reconstituted_doc = field_values
                .iter()
                .map(|field_value| {
                    let field_name = schema.get_field_name(field_value.field);
                    (field_name, field_value.value.clone())
                })
                .collect::<HashMap<_, _>>();

            (doc_addr, reconstituted_doc)
        })
        .collect::<Vec<_>>();

    for doc in docs {
        println!("{:?}", doc);
    }

    Ok(())
}

