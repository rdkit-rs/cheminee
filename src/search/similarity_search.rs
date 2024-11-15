use bitvec::prelude::BitVec;
use cheminee_similarity_model::encoder::{build_encoder_model, EncoderModel};
use std::cmp::min;

lazy_static::lazy_static! {
    pub static ref ENCODER_MODEL: EncoderModel = build_encoder_model().unwrap();
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
