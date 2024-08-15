use cheminee::search::compound_processing::process_cpd;
use cheminee::search::similarity_search::{assign_pca_bins, DESCRIPTOR_STATS, PCA_BINS, PC_MATRIX};
use ndarray::{Array1, Array2};

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

    assert_eq!(*nsr_mean_std, vec!["0.676", "1.149"]);
    assert_eq!(*nsa_mean_std, vec!["0.029", "0.215"]);
    assert_eq!(*nah_mean_std, vec!["0.597", "0.998"]);

    let pc_matrix = PC_MATRIX.clone();
    let pc0_vector = pc_matrix
        .row(0)
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();

    assert_eq!(
        pc0_vector,
        vec![
            "0.101", "0.206", "-0.005", "0.091", "0.097", "0.096", "0.039", "0.100", "0.204",
            "0.061", "0.149", "0.100", "0.209", "0.152", "0.096", "0.145", "0.147", "0.080",
            "0.081", "0.024", "0.161", "0.208", "0.208", "0.209", "0.207", "0.208", "0.198",
            "0.193", "0.198", "0.193", "0.188", "0.183", "0.208", "-0.146", "0.205", "0.180",
            "0.025", "0.209", "0.155", "0.091", "0.141"
        ]
    );

    let pca_bins = PCA_BINS.clone();
    let pc5_bins = pca_bins
        .get("pc5")
        .unwrap()
        .iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>();

    assert_eq!(
        pc5_bins,
        vec![
            "-inf", "-0.432", "-0.336", "-0.284", "-0.244", "-0.208", "-0.177", "-0.150", "-0.122",
            "-0.095", "-0.072", "-0.052", "-0.031", "-0.009", "0.013", "0.034", "0.056", "0.079",
            "0.105", "0.133", "0.170", "0.210", "0.276", "0.452", "inf"
        ]
    );
}

#[test]
fn test_assign_pca_bins() {
    let smiles = "c1ccccc1CCF";
    let (_canon_taut, _fp, descriptors) = process_cpd(smiles, false).unwrap();
    let pca_bins = assign_pca_bins(descriptors).unwrap();

    assert_eq!(*pca_bins.get("pc0").unwrap(), 3);
    assert_eq!(*pca_bins.get("pc1").unwrap(), 34);
    assert_eq!(*pca_bins.get("pc2").unwrap(), 51);
    assert_eq!(*pca_bins.get("pc3").unwrap(), 48);
    assert_eq!(*pca_bins.get("pc4").unwrap(), 19);
    assert_eq!(*pca_bins.get("pc5").unwrap(), 13);
}
