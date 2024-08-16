use cheminee::search::compound_processing::process_cpd;
use cheminee::search::similarity_search::{
    assign_pca_bins, DESCRIPTOR_STATS, PCA_BIN_EDGES, PC_MATRIX,
};
use ndarray::{Array1, Array2};
use serde_json::Value;

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

    assert_eq!(*nsr_mean_std, vec!["0.647", "0.991"]);
    assert_eq!(*nsa_mean_std, vec!["0.025", "0.168"]);
    assert_eq!(*nah_mean_std, vec!["0.583", "0.857"]);

    let pc_matrix = PC_MATRIX.clone();
    let pc0_vector = pc_matrix
        .row(0)
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();

    assert_eq!(
        pc0_vector,
        vec![
            "0.113", "0.206", "-0.001", "0.087", "0.094", "0.093", "0.040", "0.102", "0.204",
            "0.053", "0.148", "0.093", "0.209", "0.149", "0.092", "0.141", "0.152", "0.076",
            "0.076", "0.022", "0.162", "0.207", "0.208", "0.209", "0.208", "0.207", "0.200",
            "0.191", "0.200", "0.191", "0.192", "0.179", "0.207", "-0.149", "0.205", "0.181",
            "0.081", "0.209", "0.152", "0.085", "0.137"
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
        vec!["-inf", "-2.648", "-0.868", "1.463", "inf"]
    );
}

#[test]
fn test_assign_pca_bins() {
    let smiles = "c1ccccc1CCF";
    let (_canon_taut, _fp, descriptors) = process_cpd(smiles, false).unwrap();
    let pca_bins = assign_pca_bins(descriptors).unwrap();

    assert_eq!(*pca_bins.get("pc0").unwrap(), 0);
    assert_eq!(*pca_bins.get("pc1").unwrap(), 0);
    assert_eq!(*pca_bins.get("pc2").unwrap(), 1);
    assert_eq!(*pca_bins.get("pc3").unwrap(), 2);
    assert_eq!(*pca_bins.get("pc4").unwrap(), 1);
    assert_eq!(*pca_bins.get("pc5").unwrap(), 1);

    let pca_bins_json = Value::Object(
        pca_bins
            .into_iter()
            .map(|(pc, bin)| (pc, serde_json::json!(bin)))
            .collect(),
    );

    assert_eq!(format!("{:?}", pca_bins_json), "Object {\"pc0\": Number(0), \"pc1\": Number(0), \"pc2\": Number(1), \"pc3\": Number(2), \"pc4\": Number(1), \"pc5\": Number(1)}");
}
