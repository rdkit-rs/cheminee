use cheminee::cpd_methods::cpd_processing::*;
use rdkit::*;

#[test]
fn test_mol_stdz() {
    let smiles = "CC.Oc1c(cccc3CC(C(=O)[O-]))c3nc2c(C[NH+])cncc12.[Cl-]";
    let romol = ROMol::from_smile(smiles).unwrap();
    let canon_taut = mol_stdz(&romol);
    println!("{:?}", canon_taut.as_smile());
}

#[test]
fn test_smi_stdz() {
    let smiles = "CC.Oc1c(cccc3CC(C(=O)[O-]))c3nc2c(C[NH+])cncc12.[Cl-]";
    let canon_taut = smi_stdz(&smiles);
    println!("{:?}", canon_taut.as_smile());
}
