use crate::rest_api::api::{ConvertedMolBlock, ConvertedMolBlockResponse};
use crate::rest_api::models::Smiles;
use poem_openapi::payload::Json;
use rayon::prelude::*;
use rdkit::ROMol;

pub async fn v1_convert_smiles_to_mol_block(
    smiles_vec: Json<Vec<Smiles>>,
) -> ConvertedMolBlockResponse {
    let mol_blocks = smiles_vec
        .0
        .into_par_iter()
        .map(|s| {
            let ro_mol = ROMol::from_smiles(&s.smiles);

            let (mol_block, error) = match ro_mol {
                Ok(ro_mol) => (Some(ro_mol.to_molblock()), None),
                Err(_) => (
                    None,
                    Some(format!("Could not convert smiles\n{}\n", s.smiles)),
                ),
            };

            ConvertedMolBlock { mol_block, error }
        })
        .collect::<Vec<_>>();

    ConvertedMolBlockResponse::Ok(Json(mol_blocks))
}
