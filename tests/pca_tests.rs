use cheminee::search::similarity_search::PCA_PARAMS;
use cheminee::search::SIMILARITY_DESCRIPTORS;
use ndarray::{Array1, Array2};
use serde_json::Value;
use std::collections::HashMap;

#[test]
fn test_dot_product() {
    let matrix =
        Array2::<f64>::from_shape_vec((2, 4), vec![1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0])
            .unwrap();
    let vector = Array1::<f64>::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
    let result = matrix.dot(&vector);
    assert_eq!(result, Array1::<f64>::from_vec(vec![10.0, 20.0]));
}

#[test]
fn test_parse_pca_params() {
    let num_pcs = 6; // empirically determined number of PCs; captures ~85% of descriptor variance
    let num_descriptors = SIMILARITY_DESCRIPTORS.len();
    let mut descriptor_means: Vec<f64> = Vec::with_capacity(num_descriptors);
    let mut descriptor_stds: Vec<f64> = Vec::with_capacity(num_descriptors);
    let mut flat_pc_matrix: Vec<f64> = Vec::with_capacity(num_pcs * num_descriptors);
    let mut pc_bin_hash: HashMap<String, Vec<f64>> = HashMap::new();
    let _ = PCA_PARAMS
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
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

                        descriptor_means.push(
                            dstats
                                .get("mean")
                                .expect("Failed to retrieve descriptor mean")
                                .as_f64()
                                .unwrap(),
                        );
                        descriptor_stds.push(
                            dstats
                                .get("std")
                                .expect("Failed to retrieve descriptor std")
                                .as_f64()
                                .unwrap(),
                        );
                    })
                    .collect::<Vec<_>>();
            } else {
                panic!("Failed to parse descriptor json!");
            }

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

            let pc_bins_value = v
                .get("PC_BIN_EDGES")
                .expect("Failed to extract PC bins from static data");
            if let Value::Object(pc_bins_map) = pc_bins_value {
                let _ = pc_bins_map
                    .iter()
                    .map(|(pc, bin_edges)| {
                        if let Value::Array(bin_edges) = bin_edges {
                            let bin_vector = bin_edges
                                .iter()
                                .map(|v| v.as_f64().unwrap())
                                .collect::<Vec<_>>();
                            pc_bin_hash.insert(pc.into(), bin_vector);
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

    let pc_matrix =
        Array2::<f64>::from_shape_vec((num_pcs, num_descriptors), flat_pc_matrix).unwrap();

    println!("{:?}", descriptor_means);
    println!("{:?}", descriptor_stds);
    println!("{:?}", pc_matrix);
    println!("{:?}", pc_bin_hash);
}
