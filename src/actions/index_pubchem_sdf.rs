use crate::tantivy::*;
use clap::{Arg, ArgMatches, Command};
use rdkit_sys::molecule::Molecule;
use rdkit_sys::MolBlockIter;

pub const NAME: &'static str = "index-pubchem-sdf";

pub fn command() -> Command<'static> {
    Command::new(NAME)
        .arg(Arg::new("sdf").required(true).long("sdf").takes_value(true))
        .arg(
            Arg::new("limit")
                .required(false)
                .long("limit")
                .takes_value(true),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let path = matches.value_of("sdf").unwrap();
    let limit = matches.value_of("limit");

    let mol_iter = MolBlockIter::from_gz_file(path)
        .map_err(|e| eyre::eyre!("could not read gz file: {:?}", e))?;

    let mol_iter: Box<dyn Iterator<Item = _>> = if let Some(limit) = limit {
        Box::new(mol_iter.take(limit.parse()?))
    } else {
        Box::new(mol_iter)
    };

    let (schema, index) = index()?;

    let mut index_writer = index.writer_with_num_threads(1, 50 * 1024 * 1024)?;

    for mol_block in mol_iter {
        let mol = match Molecule::new(&mol_block, "") {
            Some(m) => m,
            None => continue,
        };

        let smile = schema.get_field("smile").unwrap();
        let description = schema.get_field("description").unwrap();

        let json: serde_json::Value = serde_json::from_str(&mol.get_descriptors())?;

        let doc = doc!(
            smile => mol.get_smiles(""),
            description => json
        );

        index_writer.add_document(doc)?;
    }

    index_writer.commit()?;

    Ok(())
}
