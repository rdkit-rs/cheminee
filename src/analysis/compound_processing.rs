use rdkit::*;
use std::collections::HashMap;
use bitvec::prelude::*;

pub fn standardize_mol(romol: &ROMol) -> ROMol {
    let rwmol = romol.as_rw_mol(false, 1);
    let cleanup_params = CleanupParameters::default();
    let parent_rwmol = fragment_parent(&rwmol, &cleanup_params, true);

    let uncharger = Uncharger::new(false);
    let uncharged_mol = uncharger.uncharge(&parent_rwmol.to_ro_mol());

    let te = TautomerEnumerator::new();
    let canon_taut = te.canonicalize(&uncharged_mol);
    canon_taut
}

pub fn standardize_smiles(smi: &str) -> ROMol {
    let romol = ROMol::from_smile(smi).unwrap();
    let canon_taut = standardize_mol(&romol);
    canon_taut
}

pub fn get_tautomers(romol: &ROMol) -> Vec<ROMol> {
    let te = TautomerEnumerator::new();
    let ter = te.enumerate(&romol);
    let ts = ter.collect::<Vec<_>>();
    ts
}

pub fn process_cpd(smi: &str) -> (&str, BitVec<u8>, HashMap<String, f64>) {
    let canon_taut = standardize_smiles(smi);
    let properties = Properties::new();
    let computed = properties.compute_properties(&canon_taut);
    let rdkit_fp = canon_taut.fingerprint().0;
    (smi, rdkit_fp, computed)
}

lazy_static::lazy_static! {
    static ref BRACKETS_RE: regex::Regex = regex::Regex::new(r"\[(?P<e>B|C|N|O|P|S|F|Cl|Br|I)\]").unwrap();
}

pub fn remove_organic_brackets(smi: &str) -> String {
    let new_smi = BRACKETS_RE.replace_all(smi, "$e");
    new_smi.as_ref().to_string()
}
