use std::path::Path;

pub use tantivy::doc;
use tantivy::{directory::MmapDirectory, schema::*, Index, IndexBuilder, TantivyError};

pub mod index_manager;

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

pub fn create_or_reset_index(p: impl AsRef<Path>, schema: &Schema) -> eyre::Result<Index> {
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

    Ok(index)
}

pub fn open_index(p: impl AsRef<Path>) -> eyre::Result<Index> {
    let directory = MmapDirectory::open(p)?;

    let index = Index::open(directory)?;

    Ok(index)
}

// pub fn basic_search(searcher: &Searcher, query: &String) -> eyre::Result<()> {
//
//     let schema = searcher.schema();
//     let index = searcher.index();
//     let query_parser = QueryParser::for_index(index, vec![]);
//
//     let query = query_parser.parse_query(query)?;
//
//     let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;
//
//     let docs = top_docs
//         .into_iter()
//         .map(|(_score, doc_addr)| {
//             let doc = searcher.doc(doc_addr).unwrap();
//             let field_values = doc.field_values();
//             let reconstituted_doc = field_values
//                 .iter()
//                 .map(|field_value| {
//                     let field_name = schema.get_field_name(field_value.field);
//                     (field_name, field_value.value.clone())
//                 })
//                 .collect::<HashMap<_, _>>();
//
//             (doc_addr, reconstituted_doc)
//         })
//         .collect::<Vec<_>>();
//
//     for doc in docs {
//         println!("{:?}", doc);
//     }
//
//     Ok(())
// }
