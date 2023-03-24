use bitvec::prelude::*;
use cheminee::analysis::compound_processing::*;
use rdkit::*;

#[test]
fn test_standardize_mol() {
    let smiles = "CC.Oc1c(cccc3CC(C(=O)[O-]))c3nc2c(C[NH+])cncc12.[Cl-]";
    let romol = ROMol::from_smile(smiles).unwrap();
    let canon_taut = standardize_mol(&romol).unwrap();
    assert_eq!(
        canon_taut.as_smile(),
        "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12"
    );
}

#[test]
fn test_standardize_smiles() {
    let smiles1 = "CC.Oc1c(cccc3CC(C(=O)[O-]))c3nc2c(C[NH+])cncc12.[Cl-]";
    let canon_taut1 = standardize_smiles(&smiles1).unwrap();
    assert_eq!(
        canon_taut1.as_smile(),
        "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12"
    );

    let smiles2 = "[Mg](OCC)OCC";
    let canon_taut2 = standardize_smiles(&smiles2).unwrap();
    assert_eq!(canon_taut2.as_smile(), "CCO");
}

#[test]
fn test_standardize_bad_smiles() {
    env_logger::init();

    let smiles = "smiles";
    assert!(standardize_smiles(&smiles).is_err());
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
    let (proc_smiles, fingerprint, descriptors) = process_cpd(smiles).unwrap();

    // Sorry this is ugly but it's faster than the bitvec! macro
    let expected_fp: BitVec<u8, Lsb0> = bitvec::vec::BitVec::from_slice(&[
        33, 8, 128, 6, 36, 16, 129, 34, 2, 128, 8, 1, 161, 1, 64, 16, 8, 97, 8, 0, 2, 8, 0, 144,
        76, 4, 0, 0, 200, 4, 32, 16, 24, 80, 64, 34, 81, 4, 66, 0, 0, 140, 0, 1, 0, 4, 1, 20, 4, 2,
        193, 32, 32, 8, 0, 0, 4, 32, 0, 1, 0, 1, 36, 32, 24, 4, 68, 64, 0, 0, 8, 100, 16, 0, 0, 8,
        0, 4, 20, 83, 129, 0, 0, 144, 36, 52, 0, 2, 0, 209, 128, 13, 100, 0, 10, 130, 0, 144, 8,
        137, 8, 36, 0, 146, 64, 167, 48, 140, 69, 16, 40, 0, 128, 16, 64, 128, 98, 16, 0, 0, 13,
        128, 98, 4, 16, 64, 0, 1, 14, 32, 1, 72, 16, 0, 10, 0, 17, 66, 132, 32, 8, 38, 0, 10, 9, 1,
        0, 0, 3, 0, 2, 0, 128, 1, 32, 0, 160, 0, 137, 128, 128, 1, 8, 6, 2, 128, 72, 97, 66, 8, 0,
        132, 2, 64, 1, 65, 64, 0, 64, 1, 24, 0, 1, 0, 0, 44, 80, 18, 128, 8, 5, 33, 0, 32, 0, 0, 1,
        12, 133, 32, 208, 84, 2, 74, 128, 0, 4, 1, 16, 16, 39, 68, 0, 4, 0, 128, 80, 2, 6, 1, 4, 0,
        4, 107, 1, 0, 148, 144, 0, 1, 64, 18, 32, 72, 0, 4, 50, 64, 24, 0, 34, 12, 44, 19, 65, 0,
        212, 65, 136, 9, 8, 0, 0, 67, 1, 130,
    ]);

    assert_eq!(proc_smiles, "Oc1c(cccc3)c3nc2ccncc12");
    assert_eq!(fingerprint.0, expected_fp);
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
