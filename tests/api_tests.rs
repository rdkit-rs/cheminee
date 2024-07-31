use cheminee::indexing::index_manager::IndexManager;
use cheminee::rest_api::api::{ConvertedSmilesResponse, StandardizeResponse};
use cheminee::rest_api::models::{MolBlock, Smiles};
use cheminee::rest_api::openapi_server::Api;
use poem::{handler, Route};
use poem_openapi::param::Query;
use poem_openapi::payload::Json;

const MOL_BLOCK: &'static str = r#"
  -OEChem-05172223082D

 31 30  0     1  0  0  0  0  0999 V2000
    2.8660    0.7500    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
    2.8660   -2.2500    0.0000 O   0  5  0  0  0  0  0  0  0  0  0  0
    2.0000   -0.7500    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
    3.7320    2.2500    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
    5.4641    0.2500    0.0000 N   0  3  0  0  0  0  0  0  0  0  0  0
    4.5981    0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    3.7320    0.2500    0.0000 C   0  0  3  0  0  0  0  0  0  0  0  0
    6.3301   -0.2500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    5.9641    1.1160    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    4.9641   -0.6160    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    3.7320   -0.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    2.8660   -1.2500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    2.8660    1.7500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    2.0000    2.2500    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    4.9966    1.2250    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    4.1996    1.2250    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    3.7320    0.8700    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    6.0201   -0.7869    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    6.8671   -0.5600    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    6.6401    0.2869    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    6.5010    0.8060    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    6.2741    1.6530    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    5.4272    1.4260    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    4.4272   -0.3060    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    4.6541   -1.1530    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    5.5010   -0.9260    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    3.9441   -1.3326    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    4.3426   -0.6423    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    2.3100    2.7869    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    1.4631    2.5600    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
    1.6900    1.7131    0.0000 H   0  0  0  0  0  0  0  0  0  0  0  0
  1  7  1  0  0  0  0
  1 13  1  0  0  0  0
  2 12  1  0  0  0  0
  3 12  2  0  0  0  0
  4 13  2  0  0  0  0
  5  6  1  0  0  0  0
  5  8  1  0  0  0  0
  5  9  1  0  0  0  0
  5 10  1  0  0  0  0
  6  7  1  0  0  0  0
  6 15  1  0  0  0  0
  6 16  1  0  0  0  0
  7 11  1  0  0  0  0
  7 17  1  0  0  0  0
  8 18  1  0  0  0  0
  8 19  1  0  0  0  0
  8 20  1  0  0  0  0
  9 21  1  0  0  0  0
  9 22  1  0  0  0  0
  9 23  1  0  0  0  0
 10 24  1  0  0  0  0
 10 25  1  0  0  0  0
 10 26  1  0  0  0  0
 11 12  1  0  0  0  0
 11 27  1  0  0  0  0
 11 28  1  0  0  0  0
 13 14  1  0  0  0  0
 14 29  1  0  0  0  0
 14 30  1  0  0  0  0
 14 31  1  0  0  0  0
M  CHG  2   2  -1   5   1
M  END
"#;

#[handler]
async fn no_sanitize_index() -> ConvertedSmilesResponse {
    let sanitize = Query("".to_string());
    let smiles = Json(vec![MolBlock {
        mol_block: MOL_BLOCK.to_string(),
    }]);
    Api {
        index_manager: IndexManager::new("/tmp/blah", false).unwrap(),
    }
    .v1_convert_mol_block_to_smiles(sanitize, smiles)
    .await
}

#[handler]
async fn sanitize_index() -> ConvertedSmilesResponse {
    let sanitize = Query("true".to_string());
    let smiles = Json(vec![MolBlock {
        mol_block: MOL_BLOCK.to_string(),
    }]);
    Api {
        index_manager: IndexManager::new("/tmp/blah", false).unwrap(),
    }
    .v1_convert_mol_block_to_smiles(sanitize, smiles)
    .await
}

#[tokio::test]
async fn test_poem1() {
    let app = Route::new()
        .at("/no_sanitize", poem::post(no_sanitize_index))
        .at("/sanitize", poem::post(sanitize_index));
    let client = poem::test::TestClient::new(app);

    // First test
    let resp = client.post("/no_sanitize").send().await;

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
        .assert_string("[H]C([H])([H])C(=O)OC([H])(C([H])([H])C(=O)[O-])C([H])([H])[N+](C([H])([H])[H])(C([H])([H])[H])C([H])([H])[H]");

    // Second test
    let resp = client.post("/sanitize").send().await;

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
        .assert_string("CC(=O)OC(CC(=O)[O-])C[N+](C)(C)C");
}

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
async fn test_poem2() {
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
}
