use crate::search::SIMILARITY_DESCRIPTORS;
use ndarray::Array2;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

const PCA_PARAMS: &str = include_str!("../../assets/cheminee_pca_params_20240814.json");

lazy_static::lazy_static! {
    pub static ref DESCRIPTOR_STATS: Arc<HashMap<String, Vec<f64>>> = get_descriptor_stats();
    pub static ref PC_MATRIX: Arc<Array2<f64>> = get_pc_matrix();
    pub static ref PCA_BINS: Arc<HashMap<String, Vec<f64>>> = get_pca_bins();
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
