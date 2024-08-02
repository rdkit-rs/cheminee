use crate::rest_api::models::MolBlock;
use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Object};
use rayon::prelude::*;
use rdkit::RWMol;

#[derive(ApiResponse, Debug)]
pub enum ConvertedSmilesResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<ConvertedSmiles>>),
}

#[derive(Object, Debug)]
pub struct ConvertedSmiles {
    #[oai(skip_serializing_if_is_none)]
    pub smiles: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}

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
