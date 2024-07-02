use std::path::Path;

pub use tantivy::doc;
use tantivy::{directory::MmapDirectory, schema::*, Index, IndexBuilder, TantivyError};

pub mod index_manager;

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

pub fn combine_json_objects(
    obj1: Option<serde_json::Value>,
    obj2: Option<serde_json::Value>,
) -> Option<serde_json::Value> {
    match (obj1, obj2) {
        (Some(obj1), Some(obj2)) => {
            if let (serde_json::Value::Object(mut obj1_map), serde_json::Value::Object(obj2_map)) =
                (obj1.clone(), obj2.clone())
            {
                for (key, value) in obj2_map {
                    obj1_map.insert(key, value);
                }
                return Some(serde_json::Value::Object(obj1_map));
            } else if let serde_json::Value::Object(obj1_map) = obj1 {
                return Some(serde_json::Value::Object(obj1_map));
            }
            Some(obj2)
        }
        (Some(obj1), None) => Some(obj1),
        (None, Some(obj2)) => Some(obj2),
        (None, None) => None,
    }
}
