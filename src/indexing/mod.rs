use crate::search::compound_processing::process_cpd;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::similarity_search::encode_fingerprints;
use bitvec::prelude::BitVec;
use rayon::prelude::*;
use rdkit::Fingerprint;
use std::collections::HashMap;
use std::path::Path;
pub use tantivy::doc;
use tantivy::{directory::MmapDirectory, schema::*, Index, IndexBuilder, TantivyError};

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

#[derive(Clone)]
pub struct CompoundDocAttributes {
    pub smiles: String,
    pub extra_data: Option<serde_json::Value>,
    pub pattern_fingerprint: Fingerprint,
    pub morgan_fingerprint: Fingerprint,
    pub descriptors: HashMap<String, f64>,
    pub scaffold_ids: Vec<i64>,
    pub status: String,
}

pub struct CompoundDocFields {
    pub smiles: Field,
    pub pattern_fingerprint: Field,
    pub morgan_fingerprint: Field,
    pub descriptors: HashMap<String, Field>,
    pub extra_data: Field,
    pub other_descriptors: Field,
}

pub fn batch_doc_creation(
    compounds: &[(String, Option<serde_json::Value>)],
    schema: &Schema,
) -> eyre::Result<Vec<eyre::Result<impl Document>>> {
    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (kd.to_string(), schema.get_field(kd).unwrap()))
        .collect::<HashMap<String, Field>>();

    let compound_doc_fields = CompoundDocFields {
        smiles: schema.get_field("smiles")?,
        extra_data: schema.get_field("extra_data")?,
        pattern_fingerprint: schema.get_field("pattern_fingerprint")?,
        morgan_fingerprint: schema.get_field("morgan_fingerprint")?,
        descriptors: descriptor_fields,
        other_descriptors: schema.get_field("other_descriptors")?,
    };

    let placeholder_attributes = get_compound_doc_attributes("c1ccccc1", &None)?;

    let mol_attributes = compounds
        .into_par_iter()
        .map(|(smiles, extra_data)| {
            let attributes_result = get_compound_doc_attributes(smiles, extra_data);
            match attributes_result {
                Ok(attributes) => attributes,
                Err(e) => {
                    let mut attributes = placeholder_attributes.clone();
                    attributes.status = format!("{e}");
                    attributes
                }
            }
        })
        .collect::<Vec<_>>();

    let batch_size = 200;
    let num_compounds = mol_attributes.len();
    let num_batches = (num_compounds as f32 / batch_size as f32).ceil() as usize;
    let mut similarity_clusters: Vec<Vec<Vec<i32>>> = Vec::with_capacity(num_batches);
    let mut morgan_bitvecs: Vec<BitVec<u8>> = Vec::with_capacity(batch_size);

    for attributes in &mol_attributes {
        let morgan_fp = attributes.morgan_fingerprint.clone();
        morgan_bitvecs.push(morgan_fp.0);

        if morgan_bitvecs.len() == batch_size {
            let similarity_cluster_batch = encode_fingerprints(&morgan_bitvecs, true)
                .map_err(|e| eyre::eyre!("Failed batched similarity cluster assignment: {e}"))?;

            similarity_clusters.push(similarity_cluster_batch);
            morgan_bitvecs.clear();
        }
    }

    if !morgan_bitvecs.is_empty() {
        let similarity_cluster_batch = encode_fingerprints(&morgan_bitvecs, true)
            .map_err(|e| eyre::eyre!("Failed batched similarity cluster assignment: {e}"))?;

        similarity_clusters.push(similarity_cluster_batch);
    }

    let similarity_clusters = similarity_clusters
        .into_iter()
        .flatten()
        .collect::<Vec<Vec<i32>>>();

    let docs = (0..num_compounds)
        .into_par_iter()
        .map(|i| {
            let attributes = mol_attributes[i].clone();
            if attributes.status == "Passed" {
                create_tantivy_doc(attributes, similarity_clusters[i][0], &compound_doc_fields)
            } else {
                Err(eyre::eyre!("{}", attributes.status))
            }
        })
        .collect::<Vec<_>>();

    Ok(docs)
}

pub fn get_compound_doc_attributes(
    raw_smiles: &str,
    extra_data: &Option<serde_json::Value>,
) -> eyre::Result<CompoundDocAttributes> {
    let initial_attributes = process_cpd(raw_smiles, false)?;
    let mut scaffold_ids = scaffold_search(
        &initial_attributes.1 .0,
        &initial_attributes.0,
        &PARSED_SCAFFOLDS,
    )?;

    if scaffold_ids.is_empty() {
        scaffold_ids.push(-1);
    }

    Ok(CompoundDocAttributes {
        smiles: initial_attributes.0.as_smiles(),
        pattern_fingerprint: initial_attributes.1,
        morgan_fingerprint: initial_attributes.0.morgan_fingerprint(),
        descriptors: initial_attributes.2,
        extra_data: extra_data.clone(),
        scaffold_ids,
        status: "Passed".to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn create_tantivy_doc(
    compound_doc_attributes: CompoundDocAttributes,
    similarity_cluster: i32,
    compound_doc_fields: &CompoundDocFields,
) -> eyre::Result<impl Document> {
    let mut doc = doc!(
        compound_doc_fields.smiles => compound_doc_attributes.smiles,
        compound_doc_fields.pattern_fingerprint => compound_doc_attributes.pattern_fingerprint.0.as_raw_slice(),
        compound_doc_fields.morgan_fingerprint => compound_doc_attributes.morgan_fingerprint.0.as_raw_slice(),
    );

    let scaffold_json = serde_json::json!({"scaffolds": compound_doc_attributes.scaffold_ids});
    let cluster_json = serde_json::json!({"similarity_cluster": similarity_cluster});
    let other_descriptors_json = combine_json_objects(Some(scaffold_json), Some(cluster_json));

    if let Some(other_descriptors_json) = other_descriptors_json {
        doc.add_field_value(
            compound_doc_fields.other_descriptors,
            other_descriptors_json,
        );
    }

    if let Some(extra_data) = compound_doc_attributes.extra_data {
        doc.add_field_value(compound_doc_fields.extra_data, extra_data);
    }

    for field in KNOWN_DESCRIPTORS {
        if let Some(val) = compound_doc_attributes.descriptors.get(field) {
            if field.starts_with("Num") || field.starts_with("lipinski") {
                let int = *val as i64;
                doc.add_field_value(*compound_doc_fields.descriptors.get(field).unwrap(), int);
            } else {
                doc.add_field_value(*compound_doc_fields.descriptors.get(field).unwrap(), *val);
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
