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
