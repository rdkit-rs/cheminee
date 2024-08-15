use cheminee::search::compound_processing::process_cpd;
use cheminee::search::similarity_search::{assign_pca_bins, DESCRIPTOR_STATS, PCA_BINS, PC_MATRIX};
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

    assert_eq!(*nsr_mean_std, vec!["0.659", "1.055"]);
    assert_eq!(*nsa_mean_std, vec!["0.030", "0.212"]);
    assert_eq!(*nah_mean_std, vec!["0.590", "0.883"]);

    let pc_matrix = PC_MATRIX.clone();
    let pc0_vector = pc_matrix
        .row(0)
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();

    assert_eq!(
        pc0_vector,
        vec![
            "0.112", "0.210", "-0.006", "0.088", "0.097", "0.090", "0.042", "0.105", "0.208",
            "0.054", "0.150", "0.092", "0.213", "0.151", "0.094", "0.147", "0.147", "0.073",
            "0.077", "0.029", "0.162", "0.211", "0.212", "0.213", "0.212", "0.211", "0.201",
            "0.184", "0.201", "0.184", "0.191", "0.138", "0.211", "-0.154", "0.208", "0.183",
            "0.001", "0.213", "0.155", "0.084", "0.139"
        ]
    );

    let pca_bins = PCA_BINS.clone();
    let pc0_bins = pca_bins
        .get("pc0")
        .unwrap()
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();

    assert_eq!(
        pc0_bins,
        vec![
            "-inf", "-4.851", "-3.986", "-3.439", "-3.022", "-2.654", "-2.352", "-2.034", "-1.663",
            "-1.265", "-0.869", "-0.416", "0.050", "0.517", "1.019", "1.604", "2.254", "3.095",
            "4.393", "7.196", "inf"
        ]
    );
}

#[test]
fn test_assign_pca_bins() {
    let smiles = "c1ccccc1CCF";
    let (_canon_taut, _fp, descriptors) = process_cpd(smiles, false).unwrap();
    let pca_bins = assign_pca_bins(descriptors).unwrap();

    assert_eq!(*pca_bins.get("pc0").unwrap(), 0);
    assert_eq!(*pca_bins.get("pc1").unwrap(), 0);
    assert_eq!(*pca_bins.get("pc2").unwrap(), 2);
    assert_eq!(*pca_bins.get("pc3").unwrap(), 2);
    assert_eq!(*pca_bins.get("pc4").unwrap(), 1);
    assert_eq!(*pca_bins.get("pc5").unwrap(), 0);

    let pca_bins_json = Value::Object(
        pca_bins
            .into_iter()
            .map(|(pc, bin)| (pc, serde_json::json!(bin)))
            .collect(),
    );

    assert_eq!(format!("{:?}", pca_bins_json), "Object {\"pc0\": Number(0), \"pc1\": Number(0), \"pc2\": Number(2), \"pc3\": Number(2), \"pc4\": Number(1), \"pc5\": Number(0)}");
}
