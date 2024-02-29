use crate::rest_api::api::{StandardizeResponse, StandardizedSmiles};
use crate::{rest_api::models::Smiles, search::compound_processing::standardize_smiles};
use poem_openapi::payload::Json;
use rayon::prelude::*;

pub async fn v1_standardize(
    mol: Json<Vec<Smiles>>,
    attempt_fix: Option<&str>,
) -> StandardizeResponse {
    let attempt_fix = match attempt_fix {
        Some("true") => true,
        _ => false,
    };

    let standardized_smiles = mol
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

#[cfg(test)]
mod tests {
    use crate::rest_api::api::compound_processing::standardize::*;
    use crate::{indexing::index_manager::IndexManager, rest_api::openapi_server::Api};
    use poem::{handler, Route};
    use poem_openapi::param::Query;

    #[handler]
    async fn index() -> StandardizeResponse {
        let smiles = Json(vec![Smiles {
            smiles: "CC=CO".to_string(), // smiles:  "CCC=O".to_string(), -answer
        }]);
        Api {
            index_manager: IndexManager::new("/tmp/blah", false).unwrap(),
        }
        .v1_standardize(smiles, Query(Some("true".to_string())))
        .await
    }

    #[tokio::test]
    async fn test_poem() {
        let app = Route::new().at("/", poem::post(index));
        let client = poem::test::TestClient::new(app);

        let resp = client.post("/").send().await;

        resp.assert_status_is_ok();

        let json = resp.json().await;
        let json_value = json.value();
        json_value
            .array()
            .iter()
            .map(|value| value.object().get("smiles"))
            .collect::<Vec<_>>()
            .first()
            .expect("first_value")
            .assert_string("CCC=O");
        println!("{:?}", json_value);
    }
}
