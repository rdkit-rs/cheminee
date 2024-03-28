use bitvec::prelude::*;
use cheminee::search::compound_processing::*;
use rdkit::*;

#[test]
fn test_update_atom_hcount() {
    let smiles = "[H]N([H])([H])";
    let mut romol = ROMol::from_smiles(smiles).unwrap();
    let nitrogen = &mut romol.atom_with_idx(0);
    update_atom_hcount(nitrogen, 1, 4);
    set_hybridization(&mut romol);

    assert_eq!(romol.as_smiles(), "[NH4+]");
}

#[test]
fn test_neutralize_atoms() {
    let charged_phe = "C1=CC=C(C=C1)C[C@@H](C(=O)[O-])[NH3+]";
    let romol = ROMol::from_smiles(charged_phe).unwrap();
    let neutralized_romol = neutralize_atoms(&romol);
    assert_eq!(neutralized_romol.as_smiles(), "N[C@@H](Cc1ccccc1)C(=O)O");
}

#[test]
fn test_remove_hypervalent_silicon() {
    let smiles = "[Si-2].CCC";
    let fixed_smiles = remove_hypervalent_silicon(smiles);
    assert_eq!(fixed_smiles, "CCC");
}

#[test]
fn test_add_formal_charge() {
    let smiles = "CN([C])([C])([C])";
    let mut parser_params = SmilesParserParams::default();
    parser_params.set_sanitize(false);
    let mut romol = ROMol::from_smiles_with_params(smiles, &parser_params).unwrap();

    add_formal_charge(&mut romol, 1);
    assert_eq!(romol.as_smiles(), "C[N+](C)(C)C");
}

#[test]
fn test_fix_chemistry_problems() {
    let smiles1 = "F[Si-2](F)(F)(F)(F)F.CC";
    let romol1 = fix_chemistry_problems(smiles1).unwrap();
    assert_eq!(romol1.as_smiles(), "CC");

    let smiles2 = "C[N](C)(C)C";
    let romol2 = fix_chemistry_problems(smiles2).unwrap();
    assert_eq!(romol2.as_smiles(), "C[N+](C)(C)C");
}

#[test]
fn test_build_romol_from_really_bad_smiles() {
    let smiles = "smiles";
    let romol = ROMol::from_smiles(smiles);
    assert!(romol.is_err());

    let mut parser_params = SmilesParserParams::default();
    parser_params.set_sanitize(false);

    let romol = ROMol::from_smiles_with_params(smiles, &parser_params);
    assert!(romol.is_err());
}

#[test]
fn test_standardize_mol() {
    let smiles = "CC.Oc1c(cccc3CC(C(=O)[O-]))c3nc2c(C[NH+])cncc12.[Cl-]";
    let romol = ROMol::from_smiles(smiles).unwrap();
    let canon_taut = standardize_mol(&romol).unwrap();
    assert_eq!(
        canon_taut.as_smiles(),
        "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12"
    );
}

#[test]
fn test_standardize_smiles() {
    let smiles1 = "CC.Oc1c(cccc3CC(C(=O)[O-]))c3nc2c(C[NH+])cncc12.[Cl-]";
    let canon_taut1 = standardize_smiles(smiles1, false).unwrap();
    assert_eq!(
        canon_taut1.as_smiles(),
        "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12"
    );

    let smiles2 = "[Mg](OCC)OCC";
    let canon_taut2 = standardize_smiles(smiles2, false).unwrap();
    assert_eq!(canon_taut2.as_smiles(), "CCO");
}

#[test]
fn test_standardize_bad_smiles() {
    tracing_subscriber::fmt().with_env_filter("trace").init();

    let smiles = "smiles";
    assert!(standardize_smiles(smiles, false).is_err());
}

#[test]
fn test_get_tautomers() {
    let smiles = "Oc1c(cccc3)c3nc2ccncc12";
    let romol = ROMol::from_smiles(smiles).unwrap();
    let tauts = get_tautomers(&romol);
    assert_eq!(tauts.len(), 3);
}

#[test]
fn test_process_cpd() {
    let smiles = "Oc1c(cccc3)c3nc2ccncc12";
    let (canon_taut, fingerprint, descriptors) = process_cpd(smiles, false).unwrap();

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

    assert_eq!(&(canon_taut.as_smiles())[..], "O=c1c2ccccc2[nH]c2ccncc12");
    assert_eq!(fingerprint.0, expected_fp);
    assert_eq!(*descriptors.get("exactmw").unwrap(), 196.063662876);
}

#[test]
fn bad_mol_test() {
    let smiles = "F(C)(C)(C)(C)(C)";
    let romol = ROMol::from_smiles(smiles);
    assert!(romol.is_err());
}

#[test]
fn test_remove_organic_brackets() {
    let smiles = "CCCC([F])([Br])([Na])";
    let new_smiles = remove_organic_brackets(smiles);
    assert_eq!(&new_smiles, "CCCC(F)(Br)([Na])");
}
