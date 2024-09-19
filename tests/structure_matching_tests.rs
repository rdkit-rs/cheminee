use cheminee::search::structure_matching::*;
use rdkit::*;

#[test]
fn test_exact_match() {
    let smiles1 = "OC[C@H]1OC=C[C@@H](O)[C@@H]1O";
    let smiles2 = "OCC1([H])OC=CC([H])(O)C1([H])O";

    let romol1 = ROMol::from_smiles(smiles1).unwrap();
    let romol2 = ROMol::from_smiles(smiles2).unwrap();

    assert!(exact_match(&romol1, &romol1.clone(), true));
    assert!(exact_match(&romol1, &romol2, false));
    assert!(!exact_match(&romol1, &romol2, true));
}

#[test]
fn test_exact_match_fp() {
    let smiles1 = "CC";
    let smiles2 = "CCC";

    let romol1 = ROMol::from_smiles(smiles1).unwrap();
    let romol2 = ROMol::from_smiles(smiles2).unwrap();

    assert!(exact_match_fp(&romol1, &romol1.clone()));
    assert!(!exact_match_fp(&romol1, &romol2));
}

#[test]
fn test_substructure_match_fp() {
    let substructure_smiles = "c1ccccc1";
    let superstructure_smiles = "c1ccccc1CC";

    let sub_romol = ROMol::from_smiles(substructure_smiles).unwrap();
    let super_romol = ROMol::from_smiles(superstructure_smiles).unwrap();

    let sub_fp = sub_romol.fingerprint().0;
    let sub_fp = sub_fp.as_bitslice();
    let super_fp = super_romol.fingerprint().0;
    let super_fp = super_fp.as_bitslice();

    assert!(substructure_match_fp(sub_fp, super_fp));
    assert!(!substructure_match_fp(super_fp, sub_fp));
}
