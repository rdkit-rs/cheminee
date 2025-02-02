use crate::search::structure_matching::substructure_match_fp;
use bitvec::prelude::{BitSlice, BitVec};
use rdkit::{substruct_match, ROMol, SubstructMatchParameters};
use std::iter::Iterator;
use std::sync::Arc;
use std::sync::Mutex;

const SCAFFOLDS: &str = include_str!("../../assets/standardized_scaffolds_20240405.json");

pub struct Scaffold {
    pub fp: BitVec<u8>,
    pub mol: Arc<Mutex<ROMol>>,
    pub idx: i64,
}

lazy_static::lazy_static! {
    pub static ref PARSED_SCAFFOLDS: Vec<Scaffold> = SCAFFOLDS
    .lines()
    .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
    .map(|v| {
        let smiles = v.get("smiles")
                .expect("failed to get smiles from static data")
                .as_str()
                .unwrap();
        let romol = ROMol::from_smiles(smiles).expect("failed to create ROMol from static smiles");

        Scaffold {
            fp: romol.pattern_fingerprint().0,
            mol: Arc::new(Mutex::new(romol)),
            idx: v.get("scaffold_id")
                .expect("failed to get scaffold_id from static data")
                .as_i64()
                .unwrap(),
        }
    })
    .collect();
}

pub fn scaffold_search(
    query_pattern_fingerprint: &BitSlice<u8>,
    query_mol: &ROMol,
    scaffolds: &Vec<Scaffold>,
) -> eyre::Result<Vec<i64>> {
    let mut matching_scaffolds: Vec<i64> = Vec::with_capacity(scaffolds.len());
    let params = SubstructMatchParameters::default();

    for scaffold in scaffolds {
        let fp_substruct_match =
            substructure_match_fp(scaffold.fp.as_bitslice(), query_pattern_fingerprint);

        if fp_substruct_match {
            let mol_substruct_match =
                substruct_match(query_mol, &scaffold.mol.lock().unwrap(), &params);
            if !mol_substruct_match.is_empty() {
                matching_scaffolds.push(scaffold.idx);
            }
        }
    }

    Ok(matching_scaffolds)
}
