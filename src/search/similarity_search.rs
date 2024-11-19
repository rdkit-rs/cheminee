use crate::search::basic_search::basic_search;
use crate::search::StructureSearchHit;
use bitvec::order::Lsb0;
use bitvec::prelude::{BitSlice, BitVec};
use cheminee_similarity_model::encoder::{build_encoder_model, EncoderModel};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::cmp::min;
use std::collections::HashSet;
use tantivy::schema::{Field, OwnedValue, Value};
use tantivy::{DocAddress, Searcher};

lazy_static::lazy_static! {
    pub static ref ENCODER_MODEL: EncoderModel = build_encoder_model().unwrap();
}

pub fn similarity_search(
    searcher: &Searcher,
    results: HashSet<DocAddress>,
    taut_morgan_fingerprints: &[BitVec<u8>],
    tanimoto_minimum: f32,
    query_smiles: &str,
) -> eyre::Result<Vec<StructureSearchHit>> {
    let schema = searcher.schema();
    let smiles_field = schema.get_field("smiles")?;
    let morgan_fingerprint_field = schema.get_field("morgan_fingerprint")?;
    let extra_data_field = schema.get_field("extra_data")?;

    let used_tautomers = taut_morgan_fingerprints.len() > 1;

    let mut final_results = results
        .into_par_iter()
        .filter_map(|docaddr| {
            let result = get_best_similarity(
                searcher,
                &docaddr,
                smiles_field,
                morgan_fingerprint_field,
                extra_data_field,
                taut_morgan_fingerprints,
            );

            match result {
                Ok(result) => {
                    if result.2 < tanimoto_minimum {
                        None
                    } else {
                        Some(StructureSearchHit {
                            smiles: result.0,
                            extra_data: result.1,
                            score: result.2,
                            query: query_smiles.into(),
                            used_tautomers,
                        })
                    }
                }
                Err(e) => {
                    log::warn!("Encountered exception in Tanimoto calculation: {e}");
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    final_results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(final_results)
}

pub fn neighbor_search(
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

pub fn get_tanimoto_similarity(fp1: &BitSlice<u8>, fp2: &BitSlice<u8>) -> f32 {
    let and = fp1.to_bitvec() & fp2;
    let or = fp1.to_bitvec() | fp2;

    let and_ones = and.count_ones();
    let or_ones = or.count_ones();

    and_ones as f32 / or_ones as f32
}

pub fn encode_fingerprint(fingerprint: &[u8], only_best_cluster: bool) -> eyre::Result<Vec<i32>> {
    let bit_vec = BitVec::<u8, Lsb0>::from_slice(fingerprint);
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
