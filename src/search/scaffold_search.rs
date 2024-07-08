use rdkit::{substruct_match, ROMol, SubstructMatchParameters};
use std::iter::Iterator;
use std::sync::Arc;
use std::sync::Mutex;

const SCAFFOLDS: &str = include_str!("../../assets/standardized_scaffolds_20240405.json");

lazy_static::lazy_static! {
    pub static ref PARSED_SCAFFOLDS: Vec<(Arc<Mutex<ROMol>>, u64)> = SCAFFOLDS
    .lines()
    .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
    .map(|v| {
        let smiles = v.get("smiles")
                .expect("failed to get smiles from static data")
                .as_str()
                .unwrap();
        let romol = ROMol::from_smiles(smiles).expect("failed to create ROMol from static smiles");

        (
            Arc::new(Mutex::new(romol)),
            v.get("scaffold_id")
                .expect("failed to get scaffold_id from static data")
                .as_u64()
                .unwrap(),
        )
    })
    .collect();
}

// pub fn get_scaffolds() -> Vec<(ROMol, u64)> {
//     let mut scaffold_vec = Vec::with_capacity(1000);
//
//     for line in SCAFFOLDS.lines() {
//         let record: serde_json::Value = serde_json::from_str(line)?;
//         let smiles = record
//             .get("smiles")
//             .ok_or(eyre::eyre!("Failed to extract smiles"))?
//             .as_str()
//             .ok_or(eyre::eyre!("Failed to convert smiles to str"))?;
//         let scaffold_id = record
//             .get("scaffold_id")
//             .ok_or(eyre::eyre!("Failed to extract scaffold id"))?
//             .as_u64()
//             .ok_or(eyre::eyre!("Failed to convert scaffold id to integer"))?;
//
//         let romol = ROMol::from_smiles(smiles)?;
//
//         scaffold_vec.push((romol, scaffold_id));
//     }
//     Ok(scaffold_vec)
// }

pub fn scaffold_search(
    query_mol: &ROMol,
    scaffolds: &Vec<(Arc<Mutex<ROMol>>, u64)>,
) -> eyre::Result<Vec<u64>> {
    let mut matching_scaffolds: Vec<u64> = Vec::with_capacity(scaffolds.len());
    for scaffold in scaffolds {
        let params = SubstructMatchParameters::default();
        let mol_substruct_match = substruct_match(query_mol, &scaffold.0.lock().unwrap(), &params);
        if !mol_substruct_match.is_empty() {
            matching_scaffolds.push(scaffold.1);
        }
    }

    Ok(matching_scaffolds)
}
