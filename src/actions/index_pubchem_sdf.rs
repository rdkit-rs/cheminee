use super::prelude::*;
use rdkit::{MolBlockIter, ROMol, RWMol};

pub const NAME: &'static str = "index-pubchem-sdf";

pub fn command() -> Command<'static> {
    Command::new(NAME)
        .arg(Arg::new("sdf").required(true).long("sdf").takes_value(true))
        .arg(
            Arg::new("index")
                .required(true)
                .long("index")
                .short('i')
                .takes_value(true),
        )
        .arg(
            Arg::new("limit")
                .required(false)
                .long("limit")
                .takes_value(true),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<usize> {
    let sdf_path = matches.value_of("sdf").unwrap();
    let index_dir = matches.value_of("index").unwrap();
    let limit = matches.value_of("limit");

    log::info!(
        "indexing path={}, index_dir={}, limit={:?}",
        sdf_path,
        index_dir,
        limit
    );

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
        if mol.is_err() {
            continue;
        }
        let mol = mol.unwrap();
        let mol: ROMol = mol.to_ro_mol();

        let smile = schema.get_field("smile").unwrap();
        let descriptors = schema.get_field("descriptors").unwrap();
        let fingerprint = schema.get_field("fingerprint").unwrap();

        let computed = properties.compute_properties(&mol);
        let json: serde_json::Value = serde_json::to_value(&computed)?;

        let fp = mol.fingerprint();

        // todo!("we gotta map 64bits to 8 8bits");
        let doc = doc!(
            smile => mol.as_smile(),
            descriptors => json,
            fingerprint => vec![] // fp.0.into_vec()
        );

        index_writer.add_document(doc)?;
        counter += 1;
    }

    index_writer.commit()?;

    Ok(counter)
}
