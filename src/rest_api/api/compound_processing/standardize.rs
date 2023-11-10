use crate::rest_api::api::{StandardizeResponse, StandardizedSmile};
use crate::{rest_api::models::Smile, search::compound_processing::standardize_smiles};
use poem_openapi::payload::Json;
use rayon::prelude::*;

pub async fn v1_standardize(mol: Json<Vec<Smile>>) -> StandardizeResponse {
    let standardized_smiles = mol
        .0
        .into_par_iter()
        .map(|s| {
            let standardize = standardize_smiles(&s.smile);

            match standardize {
                Ok(romol) => StandardizedSmile {
                    smile: Some(romol.as_smile()),
                    error: None,
                },
                Err(e) => StandardizedSmile {
                    smile: Some(s.smile),
                    error: Some(e.to_string()),
                },
            }
        })
        .collect::<Vec<_>>();

    StandardizeResponse::Ok(Json(standardized_smiles))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{indexing::index_manager::IndexManager, rest_api::openapi_server::Api};
    use poem::{handler, Route};

    #[handler]
    async fn index() -> StandardizeResponse {
        let smiles = Json(vec![Smile {
            smile: "CC=CO".to_string(), // smile:  "CCC=O".to_string(), -answer
        }]);
        Api {
            index_manager: IndexManager::new("/tmp/blah", false).unwrap(),
        }
        .v1_standardize(smiles)
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
        // json_value.object().get("smile").assert_string("CCC=O");
        json_value
            .array()
            .iter()
            .map(|value| value.object().get("smile"))
            .collect::<Vec<_>>()
            .first()
            .expect("first_value")
            .assert_string("CCC=O");
        println!("{:?}", json_value);
    }
}
