use cheminee::analysis::structure_matching::*;
use rdkit::*;

#[test]
fn test_exact_match() {
    let smiles1 = "CCC";
    let smiles2 = "CCCC";

    let romol1 = ROMol::from_smile(smiles1).unwrap();
    let romol2 = ROMol::from_smile(smiles2).unwrap();

    assert_eq!(exact_match(&romol1, &romol1.clone()), true);
    assert_eq!(exact_match(&romol1, &romol2), false);
}

#[test]
fn test_exact_match_fp() {
    let smiles1 = "CC";
    let smiles2 = "CCC";

    let romol1 = ROMol::from_smile(smiles1).unwrap();
    let romol2 = ROMol::from_smile(smiles2).unwrap();

    assert_eq!(exact_match_fp(&romol1, &romol1.clone()), true);
    assert_eq!(exact_match_fp(&romol1, &romol2), false);
}

#[test]
fn test_substructure_match_fp() {
    let substructure_smiles = "c1ccccc1";
    let superstructure_smiles = "c1ccccc1CC";

    let sub_romol = ROMol::from_smile(substructure_smiles).unwrap();
    let super_romol = ROMol::from_smile(superstructure_smiles).unwrap();

    let sub_fp = sub_romol.fingerprint().0;
    let sub_fp = sub_fp.as_bitslice();
    let super_fp = super_romol.fingerprint().0;
    let super_fp= super_fp.as_bitslice();

    assert_eq!(substructure_match_fp(sub_fp, super_fp), true);
    assert_eq!(substructure_match_fp(super_fp, sub_fp), false);
}
