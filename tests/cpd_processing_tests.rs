use bitvec::prelude::*;
use cheminee::analysis::compound_processing::*;
use rdkit::*;

#[test]
fn test_standardize_mol() {
    let smiles = "CC.Oc1c(cccc3CC(C(=O)[O-]))c3nc2c(C[NH+])cncc12.[Cl-]";
    let romol = ROMol::from_smile(smiles).unwrap();
    let canon_taut = standardize_mol(&romol);
    assert_eq!(
        canon_taut.as_smile(),
        "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12"
    );
}

#[test]
fn test_standardize_smiles() {
    let smiles1 = "CC.Oc1c(cccc3CC(C(=O)[O-]))c3nc2c(C[NH+])cncc12.[Cl-]";
    let canon_taut1 = standardize_smiles(&smiles1);
    assert_eq!(
        canon_taut1.as_smile(),
        "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12"
    );

    let smiles2 = "[Mg](OCC)OCC";
    let canon_taut2 = standardize_smiles(&smiles2);
    assert_eq!(
        canon_taut2.as_smile(),
        "CCO"
    );
}

#[test]
fn test_get_tautomers() {
    let smiles = "Oc1c(cccc3)c3nc2ccncc12";
    let romol = ROMol::from_smile(smiles).unwrap();
    let tauts = get_tautomers(&romol);
    assert_eq!(tauts.len(), 3);
}

#[test]
fn test_process_cpd() {
    let smiles = "Oc1c(cccc3)c3nc2ccncc12";
    let (proc_smiles, fingerprint, descriptors) = process_cpd(smiles);

    let expected_fp = bitvec![
        1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
        1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0,
        0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
        0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0,
        0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
        0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1
    ];

    assert_eq!(proc_smiles, "Oc1c(cccc3)c3nc2ccncc12");
    assert_eq!(fingerprint, expected_fp);
    assert_eq!(*descriptors.get("exactmw").unwrap(), 196.063662876);
}

#[test]
fn bad_mol_test() {
    let smiles = "F(C)(C)(C)(C)(C)";
    let romol = ROMol::from_smile(smiles);
    assert!(romol.is_err());
    // let stdz_mol = mol_stdz(&romol);
}

#[test]
fn test_remove_organic_brackets() {
    let smiles = "CCCC([F])([Br])([Na])";
    let new_smiles = remove_organic_brackets(smiles);
    assert_eq!(&new_smiles, "CCCC(F)(Br)([Na])");
}

#[test]
fn test_fix_repeating_smiles() {
    let smiles = "CC(C)(C)OC(=O)NC(CC1=CSC=N1)C(=O)OCC(C)(C)OC(=O)NC(CC1=CSC=N1)C(=O)O.[Na+]";
    let fixed_smiles = fix_repeating_smiles(smiles);
    assert_eq!(&fixed_smiles, "CC(C)(C)OC(=O)NC(CC1=CSC=N1)C(=O)O");
}
