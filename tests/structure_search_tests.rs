use cheminee::cpd_methods::structure_search::*;
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
    let smiles1 = "CCC";
    let smiles2 = "CCCC";

    let romol1 = ROMol::from_smile(smiles1).unwrap();
    let romol2 = ROMol::from_smile(smiles2).unwrap();

    assert_eq!(exact_match_fp(&romol1, &romol1.clone()), true);
}
