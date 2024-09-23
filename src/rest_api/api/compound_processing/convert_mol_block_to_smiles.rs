use crate::rest_api::api::{ConvertedSmiles, ConvertedSmilesResponse};
use crate::rest_api::models::MolBlock;
use poem_openapi::payload::Json;
use rayon::prelude::*;
use rdkit::RWMol;

pub async fn v1_convert_mol_block_to_smiles(
    sanitize: String,
    molblocks: Json<Vec<MolBlock>>,
) -> ConvertedSmilesResponse {
    let sanitize = match sanitize.as_str() {
        "" | "false" | "no" => (false, false, false),
        _ => (true, true, false),
    };

    let smiles_vec = molblocks
        .0
        .into_par_iter()
        .map(|mb| {
            let rw_mol = RWMol::from_mol_block(&mb.mol_block, sanitize.0, sanitize.1, sanitize.2);

            let error = if rw_mol.is_none() {
                Some(format!("could not convert molblock\n{}\n", mb.mol_block))
            } else {
                None
            };

            ConvertedSmiles {
                smiles: rw_mol.map(|rw_mol| rw_mol.as_smiles()),
                error,
            }
        })
        .collect::<Vec<_>>();

    ConvertedSmilesResponse::Ok(Json(smiles_vec))
}
