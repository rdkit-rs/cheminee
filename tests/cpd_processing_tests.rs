use bitvec::prelude::*;
use cheminee::search::compound_processing::*;
use rdkit::*;

#[test]
fn test_update_atom_hcount() {
    let smiles = "[H]N([H])([H])";
    let mut romol = ROMol::from_smiles(smiles).unwrap();
    let nitrogen = &mut romol.atom_with_idx(0);
    let _ = update_atom_hcount(nitrogen, 1, 4);
    set_hybridization(&mut romol);

    assert_eq!(romol.as_smiles(), "[NH4+]");
}

#[test]
fn test_neutralize_atoms() {
    let charged_phe = "C1=CC=C(C=C1)C[C@@H](C(=O)[O-])[NH3+]";
    let romol = ROMol::from_smiles(charged_phe).unwrap();
    let neutralized_romol = neutralize_atoms(&romol).unwrap();
    assert_eq!(neutralized_romol.as_smiles(), "N[C@@H](Cc1ccccc1)C(=O)O");
}

#[test]
fn test_neutralize_atoms_with_boron() {
    let smiles = "OC(=O)C(N)CCCC[B-](O)(O)O";
    let romol = ROMol::from_smiles(smiles).unwrap();
    let _ = neutralize_atoms(&romol).unwrap();
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
}

#[test]
fn test_standardize_bad_smiles() {
    tracing_subscriber::fmt().with_env_filter("trace").init();

    let smiles = "smiles";
    assert!(standardize_smiles(smiles, false).is_err());
}

#[test]
fn test_bad_standardization() {
    // The smiles below is technically a perfectly valid smiles
    // but at the moment our standardization procedure can mess up isotopic hydrogens.
    // This is a rare occurrence so it's maybe not worth fixing these cases specifically just yet.
    // That said, we should at least force an error in these cases.
    let smiles = "O=C(O[2H])C(F)(F)F";
    let result = standardize_smiles(smiles, false);
    assert!(result.is_err());
}

#[test]
fn test_get_tautomers() {
    let smiles = "Oc1c(cccc3)c3nc2ccncc12";
    let romol = ROMol::from_smiles(smiles).unwrap();
    let tauts = get_tautomers(&romol);
    assert_eq!(tauts.len(), 2);
}

#[test]
fn test_process_cpd() {
    let smiles = "Oc1c(cccc3)c3nc2ccncc12";
    let (canon_taut, pattern_fingerprint, descriptors) = process_cpd(smiles, false).unwrap();

    // Sorry this is ugly but it's faster than the bitvec! macro
    let expected_fp: BitVec<u8, Lsb0> = BitVec::from_slice(&[
        2, 8, 1, 0, 128, 64, 32, 2, 1, 0, 16, 35, 128, 16, 4, 0, 4, 0, 32, 64, 128, 160, 4, 0, 0,
        172, 6, 66, 64, 32, 0, 49, 50, 68, 0, 8, 72, 1, 0, 0, 0, 0, 130, 16, 64, 1, 0, 0, 32, 4, 0,
        20, 2, 177, 76, 128, 0, 100, 34, 12, 0, 5, 2, 8, 33, 136, 48, 16, 64, 0, 40, 0, 0, 9, 8, 0,
        0, 70, 192, 0, 0, 128, 128, 169, 138, 0, 8, 76, 2, 0, 160, 68, 8, 4, 26, 0, 48, 16, 32, 96,
        0, 66, 64, 0, 0, 0, 33, 128, 34, 40, 0, 16, 4, 96, 1, 12, 1, 128, 11, 32, 136, 144, 20, 16,
        64, 138, 128, 72, 0, 34, 0, 16, 50, 26, 65, 16, 16, 0, 64, 0, 81, 48, 0, 16, 140, 184, 1,
        72, 34, 0, 40, 208, 0, 4, 48, 0, 4, 3, 0, 1, 0, 195, 0, 0, 1, 64, 3, 0, 64, 0, 144, 18,
        128, 60, 148, 8, 64, 3, 0, 0, 145, 130, 16, 18, 72, 5, 66, 4, 16, 3, 66, 8, 0, 0, 0, 32,
        32, 0, 64, 200, 128, 17, 1, 0, 32, 128, 64, 3, 32, 0, 0, 0, 64, 0, 10, 0, 176, 64, 64, 0,
        80, 0, 128, 0, 136, 40, 0, 8, 0, 0, 64, 0, 0, 0, 32, 136, 130, 192, 4, 0, 16, 8, 64, 40, 9,
        2, 0, 64, 2, 0, 8, 0, 10, 0, 64, 0,
    ]);

    assert_eq!(&(canon_taut.as_smiles())[..], "O=c1c2ccccc2[nH]c2ccncc12");
    assert_eq!(pattern_fingerprint.0, expected_fp);
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
