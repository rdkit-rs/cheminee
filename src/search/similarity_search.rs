use crate::search::basic_search::basic_search;
use crate::search::SIMILARITY_DESCRIPTORS;
use bitvec::prelude::*;
use itertools::Itertools;
use ndarray::{Array1, Array2, Axis};
use rdkit::Fingerprint;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tantivy::schema::Field;
use tantivy::{DocAddress, Searcher};

const PCA_PARAMS: &str = include_str!("../../assets/cheminee_pca_params_20240816.json");

lazy_static::lazy_static! {
    pub static ref DESCRIPTOR_STATS: Arc<HashMap<String, Vec<f64>>> = get_descriptor_stats();
    pub static ref PC_MATRIX: Arc<Array2<f64>> = get_pc_matrix();
    pub static ref PCA_BIN_EDGES: Arc<HashMap<String, Vec<f64>>> = get_pca_bin_edges();
}

pub fn similarity_search(
    searcher: &Searcher,
    descriptors: &HashMap<String, f64>,
    result_limit: usize,
    extra_query: &str,
) -> eyre::Result<HashSet<DocAddress>> {
    let query = build_similarity_query(descriptors, extra_query);
    basic_search(searcher, &query, result_limit)
}

pub fn build_similarity_query(descriptors: &HashMap<String, f64>, extra_query: &str) -> String {
    let pca_bin_vec = assign_pca_bins(descriptors);
    let mut query_parts = Vec::with_capacity(pca_bin_vec.len());

    if !extra_query.is_empty() {
        for subquery in extra_query.split(" AND ") {
            query_parts.push(subquery.to_string());
        }
    }

    query_parts.extend(
        pca_bin_vec
            .iter()
            .enumerate()
            .map(|(idx, bin)| format!("extra_data.pc{idx}:{bin}"))
            .collect::<Vec<_>>(),
    );

    query_parts.join(" AND ")
}

pub fn get_best_similarity(
    searcher: &Searcher,
    docaddr: &DocAddress,
    fingerprint_field: Field,
    taut_fingerprints: &[Fingerprint],
) -> eyre::Result<f32> {
    let doc = searcher.doc(*docaddr)?;

    let fingerprint = doc
        .get_first(fingerprint_field)
        .ok_or(eyre::eyre!("Tantivy fingerprint retrieval failed"))?
        .as_bytes()
        .ok_or(eyre::eyre!("Failed to read fingerprint as bytes"))?;

    let fingerprint = BitSlice::<u8, Lsb0>::from_slice(fingerprint);

    let similarity = taut_fingerprints
        .iter()
        .map(|fp| get_tanimoto_similarity(&fp.0, fingerprint))
        .fold(f32::MIN, |max, x| x.max(max));

    Ok(similarity)
}

pub fn get_tanimoto_similarity(fp1: &BitVec<u8>, fp2: &BitSlice<u8>) -> f32 {
    let and = fp1.to_bitvec() & fp2;
    let or = fp1.to_bitvec() | fp2;

    let and_ones = and.count_ones();
    let or_ones = or.count_ones();

    and_ones as f32 / or_ones as f32
}

fn get_descriptor_stats() -> Arc<HashMap<String, Vec<f64>>> {
    let mut descriptor_stats: HashMap<String, Vec<f64>> = HashMap::new();

    let _ = PCA_PARAMS
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).expect("Failed to parse json lines"))
        .map(|v| {
            let descriptor_value = v
                .get("DESCRIPTORS")
                .expect("Failed to extract descriptor statistics from static data");

            if let Value::Object(descriptor_map) = descriptor_value {
                let _ = descriptor_map
                    .iter()
                    .enumerate()
                    .map(|(idx, (d, dstats))| {
                        if d != SIMILARITY_DESCRIPTORS[idx] {
                            panic!("Similarity descriptor order does not match!");
                        }

                        let dmean = dstats
                            .get("mean")
                            .expect("Failed to retrieve descriptor mean")
                            .as_f64()
                            .unwrap();

                        let dstd = dstats
                            .get("std")
                            .expect("Failed to retrieve descriptor std")
                            .as_f64()
                            .unwrap();

                        descriptor_stats.insert(d.to_string(), vec![dmean, dstd]);
                    })
                    .collect::<Vec<_>>();
            } else {
                panic!("Failed to parse descriptor json!");
            }
        })
        .collect::<Vec<_>>();

    Arc::new(descriptor_stats)
}

fn get_pc_matrix() -> Arc<Array2<f64>> {
    let num_pcs = 6; // empirically determined number of PCs; captures ~85% of descriptor variance
    let num_descriptors = SIMILARITY_DESCRIPTORS.len();
    let mut flat_pc_matrix: Vec<f64> = Vec::with_capacity(num_pcs * num_descriptors);

    let _ = PCA_PARAMS
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).expect("Failed to parse json lines"))
        .map(|v| {
            let pc_matrix_value = v
                .get("PC_VECTORS")
                .expect("Failed to extract PC vectors from static data");
            if let Value::Array(rows) = pc_matrix_value {
                let _ = rows
                    .iter()
                    .map(|row| {
                        if let Value::Array(row_vector) = row {
                            let _ = row_vector
                                .iter()
                                .map(|v| flat_pc_matrix.push(v.as_f64().unwrap()))
                                .collect::<Vec<_>>();
                        } else {
                            panic!("Failed to parse PC vector!");
                        }
                    })
                    .collect::<Vec<_>>();
            } else {
                panic!("Failed to parse PCA vector json!");
            };
        })
        .collect::<Vec<_>>();

    Arc::new(Array2::<f64>::from_shape_vec((num_pcs, num_descriptors), flat_pc_matrix).unwrap())
}

fn get_pca_bin_edges() -> Arc<HashMap<String, Vec<f64>>> {
    let mut pc_bins: HashMap<String, Vec<f64>> = HashMap::new();

    let _ = PCA_PARAMS
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .map(|v| {
            let pc_bins_value = v
                .get("PC_BIN_EDGES")
                .expect("Failed to extract PC bins from static data");
            if let Value::Object(pc_bins_map) = pc_bins_value {
                let _ = pc_bins_map
                    .iter()
                    .map(|(pc, bin_edges)| {
                        if let Value::Array(bin_edges) = bin_edges {
                            let mut bin_vector = bin_edges
                                .iter()
                                .map(|v| v.as_f64().unwrap())
                                .collect::<Vec<_>>();

                            bin_vector.insert(0, f64::NEG_INFINITY);
                            bin_vector.push(f64::INFINITY);

                            pc_bins.insert(pc.into(), bin_vector);
                        } else {
                            panic!("Failed to parse PC bin edges");
                        }
                    })
                    .collect::<Vec<_>>();
            } else {
                panic!("Failed to parse PCA bins");
            }
        })
        .collect::<Vec<_>>();

    Arc::new(pc_bins)
}

pub fn assign_pca_bins(descriptors: &HashMap<String, f64>) -> Vec<u64> {
    let stdz_descriptors = SIMILARITY_DESCRIPTORS
        .iter()
        .map(|d| {
            let d_val = *descriptors.get(*d).unwrap();
            let d_stats = DESCRIPTOR_STATS.get(*d).unwrap();
            let d_mean = d_stats[0];
            let d_std = d_stats[1];
            (d_val - d_mean) / d_std
        })
        .collect::<Vec<_>>();

    let descriptor_array = Array1::<f64>::from_vec(stdz_descriptors);

    let pca_proj = PC_MATRIX.dot(&descriptor_array);

    let final_bins = pca_proj
        .iter()
        .enumerate()
        .map(|(idx, val)| {
            let current_pc = format!("pc{}", idx);
            let pc_bin_edges = PCA_BIN_EDGES.get(&current_pc).unwrap();

            // left inclusive
            let rank_search = pc_bin_edges.binary_search_by(|x| x.partial_cmp(val).unwrap());
            rank_search.unwrap_or_else(|right_bin_edge| right_bin_edge - 1) as u64
        })
        .collect::<Vec<_>>();

    final_bins
}

pub fn get_ordered_bins(bins: Vec<u64>) -> impl Iterator<Item = Vec<u64>> {
    let bins = bins.iter().map(|v| *v as i64).collect::<Vec<_>>();
    let num_pcs = PC_MATRIX.shape()[0];
    let bins_per_pc = PCA_BIN_EDGES.get("pc0").unwrap().len() - 1;
    let all_bins_flattened = (0..num_pcs)
        .map(|_| 0..bins_per_pc)
        .multi_cartesian_product()
        .flat_map(|v| v.iter().map(|e| *e as i64).collect::<Vec<_>>())
        .collect::<Vec<i64>>();

    let num_possible_bins = all_bins_flattened.len() / num_pcs;

    let all_bins =
        Array2::<i64>::from_shape_vec((num_possible_bins, num_pcs), all_bins_flattened).unwrap();

    let bin_vec = Array1::<i64>::from_vec(bins);
    let bin_diffs = &all_bins - &bin_vec;

    // Need to scale differences so that we sort bin vecs by increasing PC variance
    let scale_array = (0..num_pcs)
        .rev()
        .map(|i| bins_per_pc.pow(i as u32) as i64)
        .collect::<Vec<_>>();
    let scale_array = Array1::<i64>::from_vec(scale_array);
    let bin_diffs_scaled = &bin_diffs * &scale_array;
    let vec_diffs = bin_diffs_scaled
        .mapv(|x| x.abs())
        .sum_axis(Axis(1))
        .to_vec();

    let mut argsort = (0..vec_diffs.len()).collect::<Vec<usize>>();
    argsort.sort_by_key(|idx| vec_diffs[*idx]);

    argsort.into_iter().map(move |idx| {
        all_bins
            .row(idx)
            .iter()
            .map(|v| *v as u64)
            .collect::<Vec<_>>()
    })
}
