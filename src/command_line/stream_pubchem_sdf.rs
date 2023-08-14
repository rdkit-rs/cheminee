use rdkit::MolBlockIter;

use super::prelude::*;

pub const NAME: &str = "stream-pubchem-sdf";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(Arg::new("sdf").required(true).long("sdf").num_args(1))
        .arg(Arg::new("limit").required(false).long("limit").num_args(1))
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let path = matches.get_one::<String>("sdf").unwrap();
    let limit = matches.get_one::<String>("limit");

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
            Ok(m) => {
                let computed = properties.compute_properties(&m.to_ro_mol());
                log::info!("{:?}", computed);
                success_count += 1
            }
            Err(_) => error_count += 1,
        }
    }

    println!("successes: {}, errors: {}", success_count, error_count);

    Ok(())
}
