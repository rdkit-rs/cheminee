use cheminee::search::compound_processing::process_cpd;
use cheminee::search::similarity_search::{
    assign_pca_bins, get_ordered_bins, DESCRIPTOR_STATS, PCA_BIN_EDGES, PC_MATRIX,
};
use ndarray::{Array1, Array2};
use serde_json::Value;

#[test]
fn test_dot_product() {
    let matrix =
        Array2::<f64>::from_shape_vec((2, 4), vec![1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0])
            .unwrap();
    assert_eq!(matrix.shape(), vec![2, 4]);

    let vector = Array1::<f64>::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
    let dot_result = matrix.dot(&vector);
    assert_eq!(dot_result, Array1::<f64>::from_vec(vec![10.0, 20.0]));
}

#[test]
fn test_pca_params_parse() {
    let descriptor_stats = DESCRIPTOR_STATS.clone();
    let nsr_mean_std = descriptor_stats
        .get("NumSaturatedRings")
        .unwrap()
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();
    let nsa_mean_std = descriptor_stats
        .get("NumSpiroAtoms")
        .unwrap()
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();
    let nah_mean_std = descriptor_stats
        .get("NumAliphaticHeterocycles")
        .unwrap()
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();

    assert_eq!(*nsr_mean_std, vec!["0.642", "0.971"]);
    assert_eq!(*nsa_mean_std, vec!["0.027", "0.174"]);
    assert_eq!(*nah_mean_std, vec!["0.577", "0.809"]);

    let pc_matrix = PC_MATRIX.clone();
    let pc0_vector = pc_matrix
        .row(0)
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();

    assert_eq!(
        pc0_vector,
        vec![
            "0.113", "0.208", "-0.005", "0.076", "0.088", "0.100", "0.048", "0.105", "0.205",
            "0.048", "0.149", "0.099", "0.210", "0.153", "0.089", "0.140", "0.153", "0.058",
            "0.067", "0.031", "0.161", "0.208", "0.209", "0.210", "0.209", "0.208", "0.200",
            "0.191", "0.200", "0.191", "0.191", "0.174", "0.208", "-0.153", "0.205", "0.181",
            "0.003", "0.210", "0.155", "0.093", "0.141"
        ]
    );

    let pca_bin_edges = PCA_BIN_EDGES.clone();
    let pc0_bin_edges = pca_bin_edges
        .get("pc0")
        .unwrap()
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();

    assert_eq!(
        pc0_bin_edges,
        vec!["-inf", "-3.250", "-2.119", "-0.909", "0.543", "2.613", "inf"]
    );
}

#[test]
fn test_assign_pca_bins() {
    let smiles = "c1ccccc1CCF";
    let (_canon_taut, _fp, descriptors) = process_cpd(smiles, false).unwrap();
    let pca_bins = assign_pca_bins(&descriptors);

    assert_eq!(pca_bins, vec![0, 2, 2, 4, 1, 1]);

    let pca_bins_json = Value::Object(
        pca_bins
            .iter()
            .enumerate()
            .map(|(idx, bin)| (format!("pc{idx}"), serde_json::json!(bin)))
            .collect(),
    );

    assert_eq!(format!("{:?}", pca_bins_json), "Object {\"pc0\": Number(0), \"pc1\": Number(2), \"pc2\": Number(2), \"pc3\": Number(4), \"pc4\": Number(1), \"pc5\": Number(1)}");
}

#[test]
fn test_get_ordered_bins() {
    let ordered_bins = get_ordered_bins(&vec![0, 3, 0, 0, 0, 1]).collect::<Vec<_>>();

    assert_eq!(ordered_bins[0], vec![0, 3, 0, 0, 0, 1]);
    assert_eq!(ordered_bins[1], vec![0, 3, 0, 0, 0, 0]);
    assert_eq!(ordered_bins[2], vec![0, 3, 0, 0, 0, 2]);
    assert_eq!(ordered_bins[3], vec![0, 3, 0, 0, 0, 3]);
}
