use cheminee::indexing::index_manager::IndexManager;
use cheminee::rest_api::api::{BulkRequest, BulkRequestDoc, StandardizeResponse};
use cheminee::rest_api::models::{MolBlock, Smiles};
use cheminee::rest_api::openapi_server::{api_service, ApiV1, API_PREFIX};

use cheminee::search::compound_processing::process_cpd;
use poem::test::TestResponse;
use poem::{handler, Endpoint, Response, Route};
use poem::{EndpointExt, IntoEndpoint};
use poem_openapi::param::{Path, Query};
use poem_openapi::payload::Json;
use tempdir::TempDir;

const MOL_BLOCK: &str = r#"
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
async fn index() -> StandardizeResponse {
    let smiles = Json(vec![Smiles {
        smiles: "CC=CO".to_string(),
    }]);
    ApiV1::default().v1_standardize(smiles, Query(None)).await
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
}

fn build_test_client() -> eyre::Result<(poem::test::TestClient<impl Endpoint>, IndexManager)> {
    let tempdir = TempDir::new("cheminee-api-tests-")?;
    let index_manager = IndexManager::new(tempdir.into_path(), true)?;
    let test_api = api_service("https://does-not-matter.com", API_PREFIX)?;
    let route = Route::new()
        .nest(API_PREFIX, test_api)
        .data(index_manager.clone());
    let test_client = poem::test::TestClient::new(route);

    Ok((test_client, index_manager))
}

#[allow(dead_code)]
async fn dump_body(response: &mut TestResponse) {
    let body = response.0.take_body();
    let bytes = body.into_bytes().await.unwrap();
    panic!("{:?}", bytes);
}

#[tokio::test]
async fn test_create_index() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, _) = build_test_client()?;

    // Test index creation
    let response = test_client
        .post(format!("/api/v1/indexes/{index_name}"))
        .query("schema", &schema_name)
        .send()
        .await;

    response.assert_status_is_ok();
    response
        .assert_json(&cheminee::rest_api::api::IndexMeta {
            name: index_name.into(),
            schema: schema_name.into(),
        })
        .await;

    // and for good measure, make sure we get an error if called a second time
    // Test index creation
    let response = test_client
        .post(format!("/api/v1/indexes/{index_name}"))
        .query("schema", &schema_name)
        .send()
        .await;
    response.assert_status("400".parse()?);

    Ok(())
}

#[tokio::test]
async fn test_bulk_indexing() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, index_manager) = build_test_client()?;

    let _tantivy_index = index_manager.create(
        index_name,
        cheminee::schema::LIBRARY.get(schema_name).unwrap(),
        false,
    )?;

    // and for good measure, make sure we get an error if called a second time
    // Test index creation
    let response = test_client
        .post(format!("/api/v1/indexes/{index_name}/bulk_index"))
        .body_json(&serde_json::json!({
            "docs": [{
                "smiles": "CCC",
                "extra_data": {"meow": "mix", "for": "cats"}
            }]
        }))
        .send()
        .await;
    response.assert_status("200".parse()?);
    response
        .assert_json(&serde_json::json!({
            "statuses": [{"error": null, "opcode": 0}]
        }))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_basic_search() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, index_manager) = build_test_client()?;

    let tantivy_index = index_manager.create(
        index_name,
        cheminee::schema::LIBRARY.get(schema_name).unwrap(),
        false,
    )?;

    // Skip the API to write some docs
    {
        let schema = tantivy_index.schema();
        let smiles = schema.get_field("smiles").unwrap();
        let num_atoms = schema.get_field("NumAtoms").unwrap();

        let mut writer = tantivy_index.writer::<tantivy::TantivyDocument>(16 * 1024 * 1024)?;

        let smiles_and_descriptors = vec![
            ("CCC", 8),
            ("C1=CC=CC=C1", 8),
            ("C1=CC=CC=C1CCC2=CC=CC=C2", 28),
        ];

        for (smiles_string, smiles_num_atoms) in smiles_and_descriptors {
            writer.add_document(tantivy::doc!(
                smiles => smiles_string,
                num_atoms => smiles_num_atoms as i64
            ))?;
        }

        writer.commit()?;
    }

    let response = test_client
        .get(format!("/api/v1/indexes/{index_name}/search/basic"))
        .query("query", &"NumAtoms:[13 TO 100]")
        .send()
        .await;
    response.assert_status("200".parse()?);
    response
        .assert_json(&serde_json::json!([{
            "extra_data": "",
            "query": "NumAtoms:[13 TO 100]",
            "smiles": "C1=CC=CC=C1CCC2=CC=CC=C2"
        }]))
        .await;

    Ok(())
}

// // Test basic search
// let basic_resp = test_api
//     .v1_index_search_basic(
//         Path(index_name.to_string()),
//         Query("NumAtoms:[13 TO 100]".to_string()),
//         Query(None),
//     )
//     .await;

// assert_eq!(
//     format!("{:?}", basic_resp),
//     "Ok(Json([QuerySearchHit { extra_data: \"{\\\"scaffolds\\\":[0,126]}\", smiles: \"c1ccc(CCc2ccccc2)cc1\", query: \"NumAtoms:[13 TO 100]\" }]))"
// );

// // Test identity search
// let identity_resp = test_api
//     .v1_index_search_identity(
//         Path(index_name.to_string()),
//         Query(smi3.to_string()),
//         Query(None),
//         Query(None),
//         Query(None),
//     )
//     .await;

// assert_eq!(
//     format!("{:?}", identity_resp),
//     "Ok(Json([StructureSearchHit { extra_data: \"{\\\"scaffolds\\\":[0,126]}\", smiles: \"c1ccc(CCc2ccccc2)cc1\", score: 1.0, query: \"C1=CC=CC=C1CCC2=CC=CC=C2\", used_tautomers: false }]))"
// );

// // Test substructure search
// let substructure_resp = test_api
//     .v1_index_search_substructure(
//         Path(index_name.to_string()),
//         Query(smi2.to_string()),
//         Query(None),
//         Query(None),
//         Query(None),
//         Query(None),
//         Query(None),
//     )
//     .await;

// let substructure_resp_str = format!("{:?}", substructure_resp);

// assert!(substructure_resp_str.contains("StructureSearchHit { extra_data: \"{\\\"scaffolds\\\":[0,126]}\", smiles: \"c1ccc(CCc2ccccc2)cc1\", score: 1.0, query: \"C1=CC=CC=C1\", used_tautomers: false }"));

// // Test superstructure search
// let superstructure_resp = test_api
//     .v1_index_search_superstructure(
//         Path(index_name.to_string()),
//         Query("C1=CC=CC=C1CCC2=CC=CC=C2".to_string()),
//         Query(None),
//         Query(None),
//         Query(None),
//         Query(None),
//         Query(None),
//     )
//     .await;

// let superstructure_resp_str = format!("{:?}", superstructure_resp);

// assert!(superstructure_resp_str.contains("StructureSearchHit { extra_data: \"{\\\"scaffolds\\\":[0]}\", smiles: \"c1ccccc1\", score: 1.0, query: \"C1=CC=CC=C1CCC2=CC=CC=C2\", used_tautomers: false }"));
// assert!(superstructure_resp_str.contains("StructureSearchHit { extra_data: \"{\\\"scaffolds\\\":[-1]}\", smiles: \"CC\", score: 1.0, query: \"C1=CC=CC=C1CCC2=CC=CC=C2\", used_tautomers: false }"));

// // Test list indexes
// let list_indexes_resp = test_api.v1_list_indexes().await;
// assert_eq!(
//     format!("{:?}", list_indexes_resp),
//     "Ok(Json([IndexMeta { name: \"test-api-index\", schema: \"descriptor_v1\" }]))"
// );

// // Test list schemas
// let list_schemas_resp = test_api.v1_list_schemas().await;
// assert!(format!("{:?}", list_schemas_resp).contains("Ok(Json([Schema {"));

// // Test get index
// let get_index_resp = test_api.v1_get_index(Path(index_name.to_string())).await;
// assert!(format!("{:?}", get_index_resp).contains("Ok(Json(IndexSchema {"));

// // Test bulk delete
// let bulk_delete_request_docs = smiles_vec
//     .into_iter()
//     .map(|s| BulkRequestDoc {
//         smiles: s.into(),
//         extra_data: None,
//     })
//     .collect::<Vec<_>>();

// let bulk_delete_request = Json(BulkRequest {
//     docs: bulk_delete_request_docs,
// });

// let bulk_delete_resp = test_api
//     .v1_delete_indexes_bulk_delete(Path(index_name.to_string()), bulk_delete_request)
//     .await;

// assert_eq!(
//     format!("{:?}", bulk_delete_resp),
//     "Ok(Json(DeleteIndexBulkResponseOk { statuses: [DeleteIndexBulkResponseOkStatus { opcode: Some(4), error: None }, DeleteIndexBulkResponseOkStatus { opcode: Some(5), error: None }, DeleteIndexBulkResponseOkStatus { opcode: Some(6), error: None }] }))"
// );

// // Test delete index
// let delete_index_resp = test_api.v1_delete_index(Path(index_name.to_string())).await;
// assert_eq!(
//     format!("{:?}", delete_index_resp),
//     "Ok(Json(IndexMeta { name: \"test-api-index\", schema: \"descriptor_v1\" }))"
// );

// #[tokio::test]
// async fn test_compound_processing_endpoints() {
//     let test_api = ApiV1::default();

//     // Test molblock-to-smiles conversion with sanitization
//     let mol_block = Json(vec![MolBlock {
//         mol_block: MOL_BLOCK.to_string(),
//     }]);

//     let mol_block_resp = test_api
//         .v1_convert_mol_block_to_smiles(Query("true".to_string()), mol_block)
//         .await;

//     assert_eq!(
//         format!("{:?}", mol_block_resp),
//         "Ok(Json([ConvertedSmiles { smiles: Some(\"CC(=O)OC(CC(=O)[O-])C[N+](C)(C)C\"), error: None }]))"
//     );

//     // Test molblock-to-smiles conversion without sanitization
//     let mol_block = Json(vec![MolBlock {
//         mol_block: MOL_BLOCK.to_string(),
//     }]);

//     let mol_block_no_sanitize_resp = test_api
//         .v1_convert_mol_block_to_smiles(Query("false".to_string()), mol_block)
//         .await;

//     assert_eq!(
//         format!("{:?}", mol_block_no_sanitize_resp),
//         "Ok(Json([ConvertedSmiles { smiles: Some(\"[H]C([H])([H])C(=O)OC([H])(C([H])([H])C(=O)[O-])C([H])([H])[N+](C([H])([H])[H])(C([H])([H])[H])C([H])([H])[H]\"), error: None }]))"
//     );

//     // Test smiles-to-molblock conversion
//     let smiles = Json(vec![Smiles {
//         smiles: "CC(=O)OC(CC(=O)[O-])C[N+](C)(C)C".to_string(),
//     }]);

//     let smiles_resp = test_api.v1_convert_smiles_to_mol_block(smiles).await;
//     assert!(format!("{:?}", smiles_resp).contains("Ok(Json([ConvertedMolBlock { mol_block: Some("));

//     // Test standardization with no attempted fix
//     let smiles = Json(vec![Smiles {
//         smiles: "CC(=O)OC(CC(=O)[O-])CN(C)(C)C".to_string(),
//     }]);

//     let stdz_no_fix_resp = test_api.v1_standardize(smiles, Query(None)).await;
//     assert!(format!("{:?}", stdz_no_fix_resp).contains("could not convert smiles to romol"));

//     // Test standardization with attempted fix
//     let smiles = Json(vec![Smiles {
//         smiles: "CC(=O)OC(CC(=O)[O-])CN(C)(C)C".to_string(),
//     }]);

//     let stdz_yes_fix_resp = test_api
//         .v1_standardize(smiles, Query(Some("true".to_string())))
//         .await;
//     assert_eq!(
//         format!("{:?}", stdz_yes_fix_resp),
//         "Ok(Json([StandardizedSmiles { smiles: Some(\"CC(=O)OC(CC(=O)O)C[N+](C)(C)C\"), error: None }]))"
//     );
// }
