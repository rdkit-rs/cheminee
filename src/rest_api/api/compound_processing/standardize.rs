use crate::rest_api::api::{StandardizeResponse, StandardizedSmiles};
use crate::{rest_api::models::Smiles, search::compound_processing::standardize_smiles};
use poem_openapi::payload::Json;
use rayon::prelude::*;

pub async fn v1_standardize(
    smiles_vec: Json<Vec<Smiles>>,
    attempt_fix: Option<&str>,
) -> StandardizeResponse {
    let attempt_fix = attempt_fix.is_some();

    let standardized_smiles = smiles_vec
        .0
        .into_par_iter()
        .map(|s| {
            let standardize = standardize_smiles(&s.smiles, attempt_fix);

            match standardize {
                Ok(romol) => StandardizedSmiles {
                    smiles: Some(romol.as_smiles()),
                    error: None,
                },
                Err(e) => StandardizedSmiles {
                    smiles: Some(s.smiles),
                    error: Some(e.to_string()),
                },
            }
        })
        .collect::<Vec<_>>();

    StandardizeResponse::Ok(Json(standardized_smiles))
}
