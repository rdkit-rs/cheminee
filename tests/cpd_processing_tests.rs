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

#[test]
fn test_get_tautomers() {
    let smiles = "Oc1c(cccc3)c3nc2ccncc12";
    let romol = ROMol::from_smile(smiles).unwrap();
    let tauts = get_tautomers(&romol);
    for taut in tauts {
        println!("{:?}", taut.as_smile());
    }
}

#[test]
fn test_process_cpd() {
    let smiles = "Oc1c(cccc3)c3nc2ccncc12";
    let (proc_smiles, fingerprint, descriptors) = process_cpd(smiles);
    println!("{:?}", proc_smiles);
}
