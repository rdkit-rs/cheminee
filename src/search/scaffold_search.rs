use rdkit::{substruct_match, ROMol, SubstructMatchParameters};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn get_scaffolds() -> eyre::Result<Vec<(ROMol, u64)>> {
    let file = File::open("src/indexing/standardized_scaffolds_20240405.json")?;
    let reader = BufReader::new(file);
    let mut scaffold_vec = Vec::with_capacity(1000);

    for result_line in reader.lines() {
        let line = result_line?;
        let record: serde_json::Value = serde_json::from_str(&line)?;
        let smiles = record
            .get("smiles")
            .ok_or(eyre::eyre!("Failed to extract smiles"))?
            .as_str()
            .ok_or(eyre::eyre!("Failed to convert smiles to str"))?;
        let scaffold_id = record
            .get("scaffold_id")
            .ok_or(eyre::eyre!("Failed to extract scaffold id"))?
            .as_u64()
            .ok_or(eyre::eyre!("Failed to convert scaffold id to integer"))?;

        let romol = ROMol::from_smiles(smiles)?;

        scaffold_vec.push((romol, scaffold_id));
    }
    Ok(scaffold_vec)
}

pub fn scaffold_search(query_mol: &ROMol, scaffolds: &Vec<(ROMol, u64)>) -> eyre::Result<Vec<u64>> {
    let mut matching_scaffolds: Vec<u64> = Vec::with_capacity(scaffolds.len());
    for scaffold in scaffolds {
        let params = SubstructMatchParameters::default();
        let mol_substruct_match = substruct_match(&query_mol, &scaffold.0, &params);
        if !mol_substruct_match.is_empty() {
            matching_scaffolds.push(scaffold.1);
        }
    }

    Ok(matching_scaffolds)
}
