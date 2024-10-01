use cheminee::indexing::index_manager::IndexManager;
use cheminee::rest_api::openapi_server::{api_service, API_PREFIX};
use std::collections::HashMap;

use cheminee::indexing::{combine_json_objects, KNOWN_DESCRIPTORS};
use cheminee::search::compound_processing::process_cpd;
use cheminee::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use poem::test::TestResponse;
use poem::EndpointExt;
use poem::{Endpoint, Route};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Value};
use tantivy::{doc, Index};
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

fn fill_test_index(tantivy_index: Index) -> eyre::Result<()> {
    // Write some docs direct to the index
    let schema = tantivy_index.schema();
    let mut writer = tantivy_index.writer::<tantivy::TantivyDocument>(16 * 1024 * 1024)?;

    let smiles_field = schema.get_field("smiles")?;
    let extra_data_field = schema.get_field("extra_data")?;
    let fingerprint_field = schema.get_field("fingerprint")?;
    let descriptor_fields = KNOWN_DESCRIPTORS
        .iter()
        .map(|kd| (*kd, schema.get_field(kd).unwrap()))
        .collect::<HashMap<&str, Field>>();

    let smiles_and_extra_data = vec![
        ("CC", serde_json::json!({"extra": "data"})),
        ("C1=CC=CC=C1", serde_json::json!({"extra": "data"})),
        (
            "C1=CC=CC=C1CCC2=CC=CC=C2",
            serde_json::json!({"extra": "data"}),
        ),
    ];

    for (smiles, extra_data) in smiles_and_extra_data {
        let (canon_taut, fingerprint, descriptors) = process_cpd(smiles, false)?;

        let mut doc = doc!(
            smiles_field => canon_taut.as_smiles(),
            fingerprint_field => fingerprint.0.as_raw_slice()
        );

        let scaffold_matches = scaffold_search(&fingerprint.0, &canon_taut, &PARSED_SCAFFOLDS)?;

        let scaffold_json = match scaffold_matches.is_empty() {
            true => serde_json::json!({"scaffolds": vec![-1]}),
            false => serde_json::json!({"scaffolds": scaffold_matches}),
        };

        let extra_data_json = combine_json_objects(Some(scaffold_json), Some(extra_data));
        if let Some(extra_data_json) = extra_data_json {
            doc.add_field_value(extra_data_field, extra_data_json);
        }

        for field in KNOWN_DESCRIPTORS {
            if let Some(val) = descriptors.get(field) {
                if field.starts_with("Num") || field.starts_with("lipinski") {
                    let int = *val as i64;
                    doc.add_field_value(*descriptor_fields.get(field).unwrap(), int);
                } else {
                    doc.add_field_value(*descriptor_fields.get(field).unwrap(), *val);
                };
            }
        }

        writer.add_document(doc)?;
    }

    writer.commit()?;

    Ok(())
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
    let schema = cheminee::schema::LIBRARY.get(schema_name).unwrap();
    let smiles_field = schema.get_field("smiles").unwrap();

    let tantivy_index = index_manager.create(index_name, schema, false)?;

    let response = test_client
        .post(format!("/api/v1/indexes/{index_name}/bulk_index"))
        .body_json(&serde_json::json!({
            "docs": [{
                "smiles": "CC",
                "extra_data": {"meow": "mix", "for": "cats"}
            }, {
                "smiles": "C1=CC=CC=C1",
                "extra_data": {"purina": "puppy chow", "for": "dogs"}
            }, {
                "smiles": "C1=CC=CC=C1CCC2=CC=CC=C2",
                "extra_data": {"fish": "food", "for": "fish"}
            }]
        }))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!({
            "statuses": [{"error": null, "opcode": 0}, {"error": null, "opcode": 1}, {"error": null, "opcode": 2}]
        }))
        .await;

    let query_parser = QueryParser::for_index(&tantivy_index, vec![]);
    let query = query_parser.parse_query("*")?;

    let reader = tantivy_index.reader()?;
    let searcher = reader.searcher();
    let results = searcher.search(&query, &TopDocs::with_limit(100))?;
    assert_eq!(results.len(), 3);

    let docs = results
        .into_iter()
        .map(|(_, doc_id)| searcher.doc::<tantivy::TantivyDocument>(doc_id).unwrap())
        .map(|td| {
            td.get_first(smiles_field)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(&docs, &["CC", "c1ccccc1", "c1ccc(CCc2ccccc2)cc1",]);

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

    // Write some docs direct to the index
    fill_test_index(tantivy_index)?;

    let response = test_client
        .get(format!("/api/v1/indexes/{index_name}/search/basic"))
        .query("query", &"NumAtoms:[13 TO 100]")
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!([{
            "extra_data": {"extra": "data", "scaffolds": [0, 126]},
            "query": "NumAtoms:[13 TO 100]",
            "smiles": "c1ccc(CCc2ccccc2)cc1"
        }]))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_identity_search() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, index_manager) = build_test_client()?;

    let tantivy_index = index_manager.create(
        index_name,
        cheminee::schema::LIBRARY.get(schema_name).unwrap(),
        false,
    )?;

    fill_test_index(tantivy_index)?;

    let response = test_client
        .get(format!("/api/v1/indexes/{index_name}/search/identity"))
        .query("smiles", &"C1=CC=CC=C1CCC2=CC=CC=C2")
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!([{
            "extra_data": {"extra": "data", "scaffolds": [0, 126]},
            "query": "C1=CC=CC=C1CCC2=CC=CC=C2",
            "score": 1.0,
            "smiles": "c1ccc(CCc2ccccc2)cc1",
            "used_tautomers": false
        }]))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_substructure_search() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, index_manager) = build_test_client()?;

    let tantivy_index = index_manager.create(
        index_name,
        cheminee::schema::LIBRARY.get(schema_name).unwrap(),
        false,
    )?;

    fill_test_index(tantivy_index)?;

    let response = test_client
        .get(format!("/api/v1/indexes/{index_name}/search/substructure"))
        .query("smiles", &"C1=CC=CC=C1")
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!([{
            "extra_data": {"extra": "data", "scaffolds": [0, 126]},
            "query": "C1=CC=CC=C1",
            "score": 1.0,
            "smiles": "c1ccc(CCc2ccccc2)cc1",
            "used_tautomers": false
        }]))
        .await;
    Ok(())
}

#[tokio::test]
async fn test_superstructure_search() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, index_manager) = build_test_client()?;

    let tantivy_index = index_manager.create(
        index_name,
        cheminee::schema::LIBRARY.get(schema_name).unwrap(),
        false,
    )?;

    fill_test_index(tantivy_index)?;

    let response = test_client
        .get(format!(
            "/api/v1/indexes/{index_name}/search/superstructure"
        ))
        .query("smiles", &"C1=CC=CC=C1CCC2=CC=CC=C2")
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!([
            {
                "extra_data": {"extra": "data", "scaffolds": [-1]},
                "query": "C1=CC=CC=C1CCC2=CC=CC=C2",
                "score": 1.0,
                "smiles": "CC",
                "used_tautomers": false
            },
            {
                "extra_data": {"extra": "data", "scaffolds": [0]},
                "query": "C1=CC=CC=C1CCC2=CC=CC=C2",
                "score": 1.0,
                "smiles": "c1ccccc1",
                "used_tautomers": false
            }
        ]))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_list_indices() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, index_manager) = build_test_client()?;

    index_manager.create(
        index_name,
        cheminee::schema::LIBRARY.get(schema_name).unwrap(),
        false,
    )?;

    let response = test_client.get(format!("/api/v1/indexes")).send().await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!([{"name": index_name, "schema": schema_name}]))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_list_schemas() -> eyre::Result<()> {
    let (test_client, _) = build_test_client()?;
    let expected_schema = cheminee::schema::LIBRARY.get("descriptor_v1").unwrap();

    let response = test_client.get(format!("/api/v1/schemas")).send().await;
    response.assert_status_is_ok();

    response
        .assert_json(&serde_json::json!([{"name": "descriptor_v1", "schema": expected_schema}]))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_get_index() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, index_manager) = build_test_client()?;
    let expected_schema = cheminee::schema::LIBRARY.get("descriptor_v1").unwrap();

    index_manager.create(
        index_name,
        cheminee::schema::LIBRARY.get(schema_name).unwrap(),
        false,
    )?;

    let response = test_client
        .get(format!("/api/v1/indexes/{index_name}"))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!({
            "index": index_name,
            "schema": expected_schema
        }))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_bulk_delete() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, index_manager) = build_test_client()?;

    let tantivy_index = index_manager.create(
        index_name,
        cheminee::schema::LIBRARY.get(schema_name).unwrap(),
        false,
    )?;

    fill_test_index(tantivy_index)?;

    let response = test_client
        .delete(format!("/api/v1/indexes/{index_name}/bulk_delete"))
        .body_json(&serde_json::json!({
            "docs": [
                {"smiles": "c1ccc(CCc2ccccc2)cc1"}
            ]
        }))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!({
            "statuses": [{"error": null, "opcode": 4}],
        }))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_delete_index() -> eyre::Result<()> {
    let index_name = "test-api-index";
    let schema_name = "descriptor_v1";
    let (test_client, index_manager) = build_test_client()?;

    index_manager.create(
        index_name,
        cheminee::schema::LIBRARY.get(schema_name).unwrap(),
        false,
    )?;
    assert_eq!(index_manager.list().unwrap().len(), 1);

    let response = test_client
        .delete(format!("/api/v1/indexes/{index_name}"))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!({
            "name": "test-api-index",
            "schema": "descriptor_v1",
        }))
        .await;
    assert_eq!(index_manager.list().unwrap().len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_mol_block_to_smiles_with_sanitiz() -> eyre::Result<()> {
    let (test_client, _) = build_test_client()?;

    let response = test_client
        .post("/api/v1/convert/mol_block_to_smiles")
        .body_json(&serde_json::json!([{"mol_block": MOL_BLOCK}]))
        .query("sanitize", &"true")
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!([{"smiles": "CC(=O)OC(CC(=O)[O-])C[N+](C)(C)C"}]))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_mol_block_to_smiles_without_sanitize() -> eyre::Result<()> {
    let (test_client, _) = build_test_client()?;

    let response = test_client
        .post("/api/v1/convert/mol_block_to_smiles")
        .body_json(&serde_json::json!([{"mol_block": MOL_BLOCK}]))
        .query("sanitize", &"false")
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!([{"smiles": "[H]C([H])([H])C(=O)OC([H])(C([H])([H])C(=O)[O-])C([H])([H])[N+](C([H])([H])[H])(C([H])([H])[H])C([H])([H])[H]"}]))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_smile_to_mol_block() -> eyre::Result<()> {
    let (test_client, _) = build_test_client()?;

    let mut response = test_client
        .post("/api/v1/convert/smiles_to_mol_block")
        .body_json(&serde_json::json!([{"smiles": "CC(=O)OC(CC(=O)[O-])C[N+](C)(C)C"}]))
        .query("sanitize", &"false")
        .send()
        .await;
    response.assert_status_is_ok();

    let body = String::from_utf8(response.0.take_body().into_bytes().await?.to_vec()).unwrap();
    assert!(body.contains("RDKit          2D"));
    assert!(body.contains("END"));

    Ok(())
}

#[tokio::test]
async fn test_standardization_without_attempt_fix() -> eyre::Result<()> {
    let (test_client, _) = build_test_client()?;

    let response = test_client
        .post("/api/v1/standardize")
        .body_json(&serde_json::json!([{"smiles": "CC(=O)OC(CC(=O)[O-])CN(C)(C)C"}]))
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!([{"error": "could not convert smiles to romol (exception)", "smiles": "CC(=O)OC(CC(=O)[O-])CN(C)(C)C"}]))
        .await;

    Ok(())
}

#[tokio::test]
async fn test_standardization_with_attempt_fix() -> eyre::Result<()> {
    let (test_client, _) = build_test_client()?;

    let response = test_client
        .post("/api/v1/standardize")
        .body_json(&serde_json::json!([{"smiles": "CC(=O)OC(CC(=O)[O-])CN(C)(C)C"}]))
        .query("attempt_fix", &"true")
        .send()
        .await;
    response.assert_status_is_ok();
    response
        .assert_json(&serde_json::json!([{"smiles": "CC(=O)OC(CC(=O)O)C[N+](C)(C)C"}]))
        .await;

    Ok(())
}
