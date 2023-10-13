use poem_openapi::{payload::Json, ApiResponse, Object};
use rayon::prelude::*;

use crate::{rest_api::models::Smile, search::compound_processing::standardize_smiles};

#[cfg(test)]
mod tests {
    use poem::{handler, Route};

    use super::*;
    use crate::rest_api::Api;

    #[handler]
    async fn index() -> StandardizeResponse {
        let smiles = Json(vec![Smile {
            smile: "CC=CO".to_string(), // smile:  "CCC=O".to_string(), -answer
        }]);
        Api.v1_standardize(smiles).await
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
        // TestJsonValue(Array([Object({"smile": String("CCC=O")})]))
        //     resp.assert_text("CCC=O").await;

        println!("lllla")
    }
}

#[derive(ApiResponse)]
pub enum StandardizeResponse {
    #[oai(status = "200")]
    Ok(Json<Vec<StandardizedSmile>>),
}

#[derive(Object, Debug)]
pub struct StandardizedSmile {
    #[oai(skip_serializing_if_is_none)]
    pub smile: Option<String>,
    #[oai(skip_serializing_if_is_none)]
    pub error: Option<String>,
}

pub async fn standardize(mol: Json<Vec<Smile>>) -> StandardizeResponse {
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
