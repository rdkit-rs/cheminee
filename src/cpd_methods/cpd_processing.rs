use rdkit::*;
use std::collections::HashMap;

pub fn mol_stdz(romol: &ROMol) -> ROMol {
    let rwmol = romol.as_rw_mol(false, 1);
    let cleanup_params = CleanupParameters::default();
    let parent_rwmol = fragment_parent(&rwmol, &cleanup_params, true);

    let uncharger = Uncharger::new(false);
    let uncharged_mol = uncharger.uncharge(&parent_rwmol.to_ro_mol());

    let te = TautomerEnumerator::new();
    let canon_taut = te.canonicalize(&uncharged_mol);
    canon_taut
}

pub fn smi_stdz(smi: &str) -> ROMol {
    let romol = ROMol::from_smile(smi).unwrap();
    let canon_taut = mol_stdz(&romol);
    canon_taut
}

pub fn get_tautomers(romol: &ROMol) -> Vec<ROMol> {
    let te = TautomerEnumerator::new();
    let ter = te.enumerate(&romol);
    let ts = ter.collect::<Vec<_>>();
    ts
}

pub fn process_cpd(smi: &str) -> (&str, Fingerprint, HashMap<std::string::String, f64>) {
    let canon_taut = smi_stdz(smi);
    let properties = Properties::new();
    let computed = properties.compute_properties(&canon_taut);
    let rdkit_fp = canon_taut.fingerprint();
    (smi, rdkit_fp, computed)
}
