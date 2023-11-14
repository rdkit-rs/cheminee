use std::collections::HashMap;

use rdkit::*;

pub fn standardize_mol(romol: &ROMol) -> eyre::Result<ROMol> {
    let rwmol = romol.as_rw_mol(false, 1);
    let cleanup_params = CleanupParameters::default();
    let parent_rwmol = fragment_parent(&rwmol, &cleanup_params, false);

    let uncharger = Uncharger::new(false);
    let uncharged_mol = uncharger.uncharge(&parent_rwmol.to_ro_mol());

    let te = TautomerEnumerator::new();
    let canon_taut = te.canonicalize(&uncharged_mol);
    Ok(canon_taut)
}

pub fn standardize_smiles(smi: &str) -> eyre::Result<ROMol> {
    let romol = ROMol::from_smiles(smi)?;
    let canon_taut = standardize_mol(&romol)?;
    Ok(canon_taut)
}

pub fn get_tautomers(romol: &ROMol) -> Vec<ROMol> {
    let te = TautomerEnumerator::new();
    let ter = te.enumerate(romol);
    ter.collect::<Vec<_>>()
}

pub fn get_cpd_properties(romol: &ROMol) -> eyre::Result<(Fingerprint, HashMap<String, f64>)> {
    let properties = Properties::new();
    let computed = properties.compute_properties(romol);
    let rdkit_fp = romol.fingerprint();

    Ok((rdkit_fp, computed))
}

pub fn process_cpd(smi: &str) -> eyre::Result<(ROMol, Fingerprint, HashMap<String, f64>)> {
    let canon_taut = standardize_smiles(smi)?;
    let (rdkit_fp, computed) = get_cpd_properties(&canon_taut)?;

    Ok((canon_taut, rdkit_fp, computed))
}

lazy_static::lazy_static! {
    static ref BRACKETS_RE: regex::Regex = regex::Regex::new(r"\[(?P<e>B|C|N|O|P|S|F|Cl|Br|I)\]").unwrap();
}

pub fn remove_organic_brackets(smi: &str) -> String {
    let new_smi = BRACKETS_RE.replace_all(smi, "$e");
    new_smi.as_ref().to_string()
}
