use rdkit::*;
use std::collections::HashMap;

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
    let romol = ROMol::from_smile(smi)?;
    log::info!("gonna standardize");
    let canon_taut = standardize_mol(&romol)?;
    log::info!("done standardizing");
    Ok(canon_taut)
}

pub fn get_tautomers(romol: &ROMol) -> Vec<ROMol> {
    let te = TautomerEnumerator::new();
    let ter = te.enumerate(&romol);
    let ts = ter.collect::<Vec<_>>();
    ts
}

pub fn process_cpd(smi: &str) -> eyre::Result<(String, Fingerprint, HashMap<String, f64>)> {
    let canon_taut = standardize_smiles(smi)?;
    let proc_smi = canon_taut.as_smile();
    let properties = Properties::new();
    let computed = properties.compute_properties(&canon_taut);
    let rdkit_fp = canon_taut.fingerprint();

    Ok((proc_smi, rdkit_fp, computed))
}

lazy_static::lazy_static! {
    static ref BRACKETS_RE: regex::Regex = regex::Regex::new(r"\[(?P<e>B|C|N|O|P|S|F|Cl|Br|I)\]").unwrap();
}

pub fn remove_organic_brackets(smi: &str) -> String {
    let new_smi = BRACKETS_RE.replace_all(smi, "$e");
    new_smi.as_ref().to_string()
}

pub fn fix_repeating_smiles(smiles: &str) -> String {
    let sep = ".";
    let min_length = 10;

    let mut trunc_smiles = &(smiles.to_string())[..];
    if trunc_smiles.contains(sep) {
        let smiles_vec = trunc_smiles.split(sep).collect::<Vec<&str>>();
        let lengths = smiles_vec.iter().map(|s| s.len()).collect::<Vec<usize>>();
        let max_length = lengths.iter().max().unwrap();
        let max_idx = lengths.iter().position(|v| v == max_length).unwrap();
        trunc_smiles = &smiles_vec[max_idx];
    }

    let mut pattern_substrings: Vec<&str> = Vec::new();
    let mut pattern_counts: Vec<usize> = Vec::new();
    let mut pattern_lengths: Vec<usize> = Vec::new();
    for idx in 0..trunc_smiles.len() {
        let mut pattern_substring = "";
        let mut pattern_count: usize = 1;
        let mut pattern_length: usize = 0;
        let remaining_str = &trunc_smiles[idx..trunc_smiles.len()];
        for jdx in 0..remaining_str.len() {
            let substring = &trunc_smiles[idx..idx + jdx + 1];
            let substring_count = trunc_smiles.matches(substring).count();
            let substring_length = substring.len();

            if substring_length > pattern_substring.len() {
                pattern_substring = substring;
                pattern_count = substring_count;
                pattern_length = substring_length;
            }
        }
        let mut unq_elements = pattern_substring.split("").collect::<Vec<&str>>();
        unq_elements.dedup();
        let unq_element_num = unq_elements.len();

        if pattern_count < 2 || pattern_length < min_length || unq_element_num == 1 {
            continue;
        }

        let pattern_frac = pattern_length as f32 / trunc_smiles.len() as f32;
        if pattern_frac < 0.99 / pattern_count as f32 {
            continue;
        }

        pattern_substrings.push(pattern_substring);
        pattern_counts.push(pattern_count);
        pattern_lengths.push(pattern_length);
    }

    if pattern_counts.len() == 0 || *pattern_counts.iter().max().unwrap() as u32 == 1 {
        return trunc_smiles.to_string();
    } else {
        let max_length = pattern_lengths.iter().max().unwrap();
        let max_idx = pattern_lengths
            .iter()
            .position(|v| v == max_length)
            .unwrap();

        let output_smiles = pattern_substrings[max_idx];
        return output_smiles.to_string();
    };
}
