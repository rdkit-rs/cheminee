use std::collections::HashMap;
use std::path::Path;
use rdkit::{Fingerprint, ROMol};
pub use tantivy::doc;
use tantivy::{directory::MmapDirectory, schema::*, Index, IndexBuilder, TantivyError};
use crate::search::scaffold_search::{PARSED_SCAFFOLDS, scaffold_search};

pub mod index_manager;
pub mod segment_manager;

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

pub fn create_tantivy_doc(
    canon_taut: &ROMol,
    extra_data: &Option<serde_json::Value>,
    pattern_fp: &Fingerprint,
    morgan_fp: &Fingerprint,
    descriptors: &HashMap<String, f64>,
    similarity_cluster: i32,
    smiles_field: Field,
    pattern_fingerprint_field: Field,
    morgan_fingerprint_field: Field,
    descriptor_fields: &HashMap<&str, Field>,
    extra_data_field: Field,
    other_descriptors_field: Field,
) -> eyre::Result<impl Document> {
    let mut doc = doc!(
        smiles_field => canon_taut.as_smiles(),
        pattern_fingerprint_field => pattern_fp.0.as_raw_slice(),
        morgan_fingerprint_field => morgan_fp.0.as_raw_slice(),
    );

    let scaffold_matches = scaffold_search(&pattern_fp.0, &canon_taut, &PARSED_SCAFFOLDS)?;
    let scaffold_json = match scaffold_matches.is_empty() {
        true => serde_json::json!({"scaffolds": vec![-1]}),
        false => serde_json::json!({"scaffolds": scaffold_matches}),
    };

    let cluster_json = serde_json::json!({"similarity_cluster": similarity_cluster});

    let other_descriptors_json = combine_json_objects(Some(scaffold_json), Some(cluster_json));

    if let Some(other_descriptors_json) = other_descriptors_json {
        doc.add_field_value(other_descriptors_field, other_descriptors_json);
    }

    if let Some(extra_data) = extra_data {
        doc.add_field_value(extra_data_field, extra_data.clone());
    }

    for field in KNOWN_DESCRIPTORS {
        if let Some(val) = descriptors.get(field) {
            if field.starts_with("Num") || field.starts_with("lipinski") {
                let int = *val as i64;
                doc.add_field_value(*descriptor_fields.get(field).unwrap(), int);
            } else {
                doc.add_field_value(*descriptor_fields.get(field).unwrap(), *val);
            };
        }
    }

    Ok(doc)
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
