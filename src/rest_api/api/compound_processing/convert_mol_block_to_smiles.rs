use crate::rest_api::models::MolBlock;
use poem_openapi::payload::Json;
use poem_openapi_derive::{ApiResponse, Object};
use rdkit::RWMol;

#[derive(ApiResponse)]
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
    mol: Json<Vec<MolBlock>>,
) -> ConvertedSmilesResponse {
    let sanitize = match sanitize.as_str() {
        "" | "false" | "no" => (false, false, false),
        _ => (true, true, false),
    };

    let smiles = mol
        .0
        .into_iter()
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

    ConvertedSmilesResponse::Ok(Json(smiles))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexing::index_manager::IndexManager;
    use crate::rest_api::openapi_server::Api;
    use poem::{handler, Route};
    use poem_openapi::param::Query;

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
> <PUBCHEM_COMPOUND_CID>
1

> <PUBCHEM_COMPOUND_CANONICALIZED>
1

> <PUBCHEM_CACTVS_COMPLEXITY>
214

> <PUBCHEM_CACTVS_HBOND_ACCEPTOR>
4

> <PUBCHEM_CACTVS_HBOND_DONOR>
0

> <PUBCHEM_CACTVS_ROTATABLE_BOND>
5

> <PUBCHEM_CACTVS_SUBSKEYS>
AAADceByOAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHgAAAAAACBThgAYCCAMABAAIAACQCAAAAAAAAAAAAAEIAAACABQAgAAHAAAFIAAQAAAkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==

> <PUBCHEM_IUPAC_OPENEYE_NAME>
3-acetoxy-4-(trimethylammonio)butanoate

> <PUBCHEM_IUPAC_CAS_NAME>
3-acetyloxy-4-(trimethylammonio)butanoate

> <PUBCHEM_IUPAC_NAME_MARKUP>
3-acetyloxy-4-(trimethylazaniumyl)butanoate

> <PUBCHEM_IUPAC_NAME>
3-acetyloxy-4-(trimethylazaniumyl)butanoate

> <PUBCHEM_IUPAC_SYSTEMATIC_NAME>
3-acetyloxy-4-(trimethylazaniumyl)butanoate

> <PUBCHEM_IUPAC_TRADITIONAL_NAME>
3-acetoxy-4-(trimethylammonio)butyrate

> <PUBCHEM_IUPAC_INCHI>
InChI=1S/C9H17NO4/c1-7(11)14-8(5-9(12)13)6-10(2,3)4/h8H,5-6H2,1-4H3

> <PUBCHEM_IUPAC_INCHIKEY>
RDHQFKQIGNGIED-UHFFFAOYSA-N

> <PUBCHEM_XLOGP3_AA>
0.4

> <PUBCHEM_EXACT_MASS>
203.11575802

> <PUBCHEM_MOLECULAR_FORMULA>
C9H17NO4

> <PUBCHEM_MOLECULAR_WEIGHT>
203.24

> <PUBCHEM_OPENEYE_CAN_SMILES>
CC(=O)OC(CC(=O)[O-])C[N+](C)(C)C

> <PUBCHEM_OPENEYE_ISO_SMILES>
CC(=O)OC(CC(=O)[O-])C[N+](C)(C)C

> <PUBCHEM_CACTVS_TPSA>
66.4

> <PUBCHEM_MONOISOTOPIC_WEIGHT>
203.11575802

> <PUBCHEM_TOTAL_CHARGE>
0

> <PUBCHEM_HEAVY_ATOM_COUNT>
14

> <PUBCHEM_ATOM_DEF_STEREO_COUNT>
0

> <PUBCHEM_ATOM_UDEF_STEREO_COUNT>
1

> <PUBCHEM_BOND_DEF_STEREO_COUNT>
0

> <PUBCHEM_BOND_UDEF_STEREO_COUNT>
0

> <PUBCHEM_ISOTOPIC_ATOM_COUNT>
0

> <PUBCHEM_COMPONENT_COUNT>
1

> <PUBCHEM_CACTVS_TAUTO_COUNT>
1

> <PUBCHEM_COORDINATE_TYPE>
1
5
255

> <PUBCHEM_BONDANNOTATIONS>
7  11  3
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
    async fn test_poem() {
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
}
