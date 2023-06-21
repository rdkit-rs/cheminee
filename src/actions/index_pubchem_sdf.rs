use super::prelude::*;
use rdkit::{MolBlockIter, ROMol, RWMol};
use serde_json::{Map, Value};
use std::collections::HashMap;
use tantivy::schema::Field;

pub const NAME: &'static str = "index-pubchem-sdf";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(Arg::new("sdf").required(true).long("sdf").num_args(1))
        .arg(
            Arg::new("index")
                .required(true)
                .long("index")
                .short('i')
                .num_args(1),
        )
        .arg(Arg::new("limit").required(false).long("limit").num_args(1))
}

pub fn action(matches: &ArgMatches) -> eyre::Result<usize> {
    let sdf_path = matches.get_one::<String>("sdf").unwrap();
    let index_dir = matches.get_one::<String>("index").unwrap();
    let limit = matches.get_one::<String>("limit");

    log::info!(
        "indexing path={}, index_dir={}, limit={:?}",
        sdf_path,
        index_dir,
        limit
    );

    let index_dir_metadata = std::fs::metadata(index_dir);
    if let Ok(metadata) = index_dir_metadata {
        if metadata.is_dir() {
            std::fs::remove_dir_all(index_dir)?;
        }
    }
    std::fs::create_dir(index_dir)?;

    let mol_iter = MolBlockIter::from_gz_file(sdf_path, true, false, false)
        .map_err(|e| eyre::eyre!("could not read gz file: {:?}", e))?;

    let mol_iter: Box<dyn Iterator<Item = Result<RWMol, String>>> = if let Some(limit) = limit {
        Box::new(mol_iter.take(limit.parse()?))
    } else {
        Box::new(mol_iter)
    };

    let (schema, index) = create_or_reset_index(index_dir)?;

    let mut index_writer = index.writer_with_num_threads(1, 50 * 1024 * 1024)?;

    let properties = rdkit::Properties::new();

    let mut counter = 0;
    for mol in mol_iter {
        if counter % 100 == 0 {
            log::debug!("wrote 100 docs");
        }
        if mol.is_err() {
            continue;
        }
        let mol = mol.unwrap();
        let mol: ROMol = mol.to_ro_mol();

        let smile = schema.get_field("smile").unwrap();
        let fingerprint = schema.get_field("fingerprint").unwrap();

        let descriptors_fields = KNOWN_DESCRIPTORS
            .iter()
            .map(|kd| (*kd, schema.get_field(kd).unwrap()))
            .collect::<HashMap<&str, Field>>();

        let computed = properties.compute_properties(&mol);
        let json: serde_json::Value = serde_json::to_value(&computed)?;
        let descriptions_map: Map<String, Value> = if let serde_json::Value::Object(map) = json {
            map
        } else {
            panic!("not an object")
        };

        let fp = mol.fingerprint();

        let mut doc = doc!(
            smile => mol.as_smile(),
            fingerprint => fp.0.into_vec()
        );

        for field in KNOWN_DESCRIPTORS {
            if let Some(&serde_json::Value::Number(ref val)) = descriptions_map.get(field) {
                if field.starts_with("Num") || field.starts_with("lipinski") {
                    let int = val.as_f64().unwrap() as i64;
                    doc.add_field_value(descriptors_fields.get(field).unwrap().clone(), int);
                } else {
                    doc.add_field_value(
                        descriptors_fields.get(field).unwrap().clone(),
                        val.as_f64().unwrap(),
                    );
                };
            }
        }

        index_writer.add_document(doc)?;

        if counter % 20_000 == 0 {
            index_writer.commit()?;
        }

        counter += 1;
    }

    log::debug!("committing");
    index_writer.commit()?;

    Ok(counter)
}
