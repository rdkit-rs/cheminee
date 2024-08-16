use crate::search::SIMILARITY_DESCRIPTORS;
use ndarray::{Array1, Array2};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

const PCA_PARAMS: &str = include_str!("../../assets/cheminee_pca_params_20240816.json");

lazy_static::lazy_static! {
    pub static ref DESCRIPTOR_STATS: Arc<HashMap<String, Vec<f64>>> = get_descriptor_stats();
    pub static ref PC_MATRIX: Arc<Array2<f64>> = get_pc_matrix();
    pub static ref PCA_BIN_EDGES: Arc<HashMap<String, Vec<f64>>> = get_pca_bins();
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

fn get_pca_bins() -> Arc<HashMap<String, Vec<f64>>> {
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

pub fn assign_pca_bins(descriptors: HashMap<String, f64>) -> eyre::Result<HashMap<String, u64>> {
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

    let mut final_bins = HashMap::new();
    let _ = pca_proj
        .iter()
        .enumerate()
        .map(|(idx, val)| {
            let current_pc = format!("pc{}", idx);
            let pc_bin_edges = PCA_BIN_EDGES.get(&current_pc).unwrap();

            // left inclusive
            let rank_search = pc_bin_edges.binary_search_by(|x| x.partial_cmp(val).unwrap());
            let pc_bin = rank_search.unwrap_or_else(|right_bin_edge| right_bin_edge - 1);
            final_bins.insert(current_pc, pc_bin as u64);
        })
        .collect::<Vec<_>>();

    Ok(final_bins)
}
