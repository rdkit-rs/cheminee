use super::prelude::*;
use rdkit::MolBlockIter;

pub const NAME: &'static str = "stream-pubchem-sdf";

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

    let mol_iter = MolBlockIter::from_gz_file(path, true, false, false)
        .map_err(|e| eyre::eyre!("could not read gz file: {:?}", e))?;

    let mol_iter: Box<dyn Iterator<Item = _>> = if let Some(limit) = limit {
        Box::new(mol_iter.take(limit.parse()?))
    } else {
        Box::new(mol_iter)
    };

    let mut error_count = 0;
    let mut success_count = 0;

    let properties = rdkit::Properties::new();

    for mol in mol_iter {
        match mol {
            Some(m) => {
                let computed = properties.compute_properties(&m.to_ro_mol());
                log::info!("{:?}", computed);
                success_count += 1
            }
            None => error_count += 1,
        }
    }

    println!("successes: {}, errors: {}", success_count, error_count);

    Ok(())
}
