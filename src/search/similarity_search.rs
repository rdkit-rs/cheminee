use crate::search::basic_search::basic_search;
use bitvec::order::Lsb0;
use bitvec::prelude::{BitSlice, BitVec};
use cheminee_similarity_model::encoder::{build_encoder_model, EncoderModel};
use std::cmp::min;
use std::collections::HashSet;
use tantivy::schema::{Field, OwnedValue, Value};
use tantivy::{DocAddress, DocId, Searcher, SegmentOrdinal};

lazy_static::lazy_static! {
    pub static ref ENCODER_MODEL: EncoderModel = build_encoder_model().unwrap();
}

pub fn similarity_search(
    searcher: &Searcher,
    query_morgan_fingerprint: &BitVec<u8>,
    extra_query: &str,
    search_perc: f32,
) -> eyre::Result<HashSet<DocAddress>> {
    let query_fp_slice = query_morgan_fingerprint.as_raw_slice();
    let ranked_clusters = encode_fingerprint(query_fp_slice, false)?;
    let query = build_similarity_query(&ranked_clusters, extra_query, search_perc);

    let docs = basic_search(searcher, &query, 1_000_000)?;
    let results: HashSet<DocAddress> = docs.into_iter().collect();

    Ok(results)
}

pub fn get_best_similarity(
    searcher: &Searcher,
    docaddr: &DocAddress,
    smiles_field: Field,
    morgan_fingerprint_field: Field,
    extra_data_field: Field,
    taut_fingerprints: &[BitVec<u8>],
) -> eyre::Result<(String, serde_json::Value, f32)> {
    let doc = searcher.doc::<tantivy::TantivyDocument>(*docaddr)?;

    let fingerprint = doc
        .get_first(morgan_fingerprint_field)
        .ok_or(eyre::eyre!("Tantivy fingerprint retrieval failed"))?
        .as_bytes()
        .ok_or(eyre::eyre!("Failed to read fingerprint as bytes"))?;

    let fingerprint = BitSlice::<u8, Lsb0>::from_slice(fingerprint);

    let smiles = doc
        .get_first(smiles_field)
        .ok_or(eyre::eyre!("Tantivy smiles retrieval failed"))?;

    let smiles = match smiles {
        OwnedValue::Str(s) => s,
        other => return Err(eyre::eyre!("expected string, got {:?}", other)),
    };

    let extra_data = match doc.get_first(extra_data_field) {
        Some(extra_data) => serde_json::from_str(&serde_json::to_string(extra_data)?)?,
        None => serde_json::Value::Object(Default::default()),
    };

    let score = taut_fingerprints
        .iter()
        .map(|fp| get_tanimoto_similarity(fp, fingerprint))
        .fold(f32::MIN, |max, x| x.max(max));

    Ok((smiles.to_string(), extra_data, score))
}

pub fn score_similarity(
    docaddr: DocAddress,
    smiles_field: Field,
    morgan_fingerprint_field: Field,
    extra_data_field: Field,
    searcher: &Searcher,
    query_morgan_fingerprint: &BitSlice<u8>,
) -> eyre::Result<(String, serde_json::Value, f32, SegmentOrdinal, DocId)> {
    let doc = searcher.doc::<tantivy::TantivyDocument>(docaddr)?;

    let smiles = doc
        .get_first(smiles_field)
        .ok_or(eyre::eyre!("Tantivy smiles retrieval failed"))?;

    let smiles = match smiles {
        OwnedValue::Str(s) => s,
        other => return Err(eyre::eyre!("could not fetch smile, got {:?}", other)),
    };

    let morgan_fingerprint = doc
        .get_first(morgan_fingerprint_field)
        .ok_or(eyre::eyre!("Tantivy Morgan fingerprint retrieval failed"))?;

    let morgan_fingerprint = match morgan_fingerprint {
        OwnedValue::Bytes(f) => f,
        other => {
            return Err(eyre::eyre!(
                "could not fetch pattern_fingerprint, got {:?}",
                other
            ))
        }
    };

    let morgan_fingerprint = BitSlice::from_slice(morgan_fingerprint);
    let tanimoto_score = get_tanimoto_similarity(query_morgan_fingerprint, morgan_fingerprint);

    let extra_data = match doc.get_first(extra_data_field) {
        Some(extra_data) => serde_json::from_str(&serde_json::to_string(extra_data)?)?,
        None => serde_json::Value::Object(Default::default()),
    };

    Ok((
        smiles.to_string(),
        extra_data,
        tanimoto_score,
        docaddr.segment_ord,
        docaddr.doc_id,
    ))
}

pub fn get_tanimoto_similarity(fp1: &BitSlice<u8>, fp2: &BitSlice<u8>) -> f32 {
    let and = fp1.to_bitvec() & fp2;
    let or = fp1.to_bitvec() | fp2;

    let and_ones = and.count_ones();
    let or_ones = or.count_ones();

    and_ones as f32 / or_ones as f32
}

pub fn encode_fingerprint(fingerprint: &[u8], only_best_cluster: bool) -> eyre::Result<Vec<i32>> {
    let bit_vec = BitVec::<u8>::from_slice(fingerprint);
    let fp_vec = bit_vec
        .iter()
        .map(|b| if *b { 1 } else { 0 })
        .collect::<Vec<u8>>();

    let ranked_clusters = ENCODER_MODEL.transform(&fp_vec)?;

    if only_best_cluster {
        Ok(vec![ranked_clusters[0]])
    } else {
        Ok(ranked_clusters)
    }
}

pub fn build_similarity_query(
    ranked_clusters: &[i32],
    extra_query: &str,
    search_perc: f32,
) -> String {
    let num_search_clusters = min(
        (ENCODER_MODEL.num_centroids as f32 * search_perc / 100f32).ceil() as usize,
        ranked_clusters.len(),
    );

    let cluster_parts = (0..num_search_clusters)
        .map(|idx| format!("other_descriptors.scaffolds:{}", ranked_clusters[idx]))
        .collect::<Vec<String>>();

    let cluster_query = cluster_parts.join(" OR ");

    let mut query_parts = vec![format!("({cluster_query})")];

    if !extra_query.is_empty() {
        for subquery in extra_query.split(" AND ") {
            query_parts.push(subquery.to_string());
        }
    }

    query_parts.join(" AND ")
}