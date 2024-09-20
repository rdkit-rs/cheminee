use crate::command_line::prelude::*;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::{
    identity_search::identity_search, prepare_query_structure, StructureSearchHit,
};
use rayon::prelude::*;

pub const NAME: &str = "identity-search";

pub fn command() -> Command {
    Command::new(NAME)
        .arg(
            Arg::new("index")
                .required(true)
                .long("index")
                .short('i')
                .num_args(1),
        )
        .arg(
            Arg::new("smiles")
                .required(true)
                .long("smiles")
                .short('s')
                .num_args(1),
        )
        .arg(
            Arg::new("use-chirality")
                .required(false)
                .long("use-chirality")
                .short('c')
                .help("Indicates whether chirality should be taken into account for the search")
                .num_args(1),
        )
        .arg(
            Arg::new("extra-query")
                .required(false)
                .long("extra-query")
                .short('e')
                .help("In case of duplicate smiles entries, it may be helpful to add an extra differentiating query (e.g. using data from the 'extra_data' field)")
                .num_args(1),
        )
        .arg(
            Arg::new("use-scaffolds")
                .required(false)
                .long("use-scaffolds")
                .short('u')
                .help("By default scaffolds are computed for the smiles input to enable accelerated searching")
                .num_args(1),
        )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_path = matches
        .get_one::<String>("index")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;
    let query_smiles = matches
        .get_one::<String>("smiles")
        .ok_or(eyre::eyre!("Failed to extract SMILES"))?;
    let use_chirality = matches.get_one::<String>("use-chirality");
    let extra_query = matches.get_one::<String>("extra-query");
    let use_scaffolds = matches.get_one::<String>("use-scaffolds");

    // by default, we will ignore chirality
    let use_chirality = if let Some(use_chirality) = use_chirality {
        !matches!(use_chirality.as_str(), "false")
    } else {
        false
    };

    let extra_query = if let Some(extra_query) = extra_query {
        extra_query.clone()
    } else {
        "".to_string()
    };

    // by default, we will use scaffold-based indexing
    let use_scaffolds = if let Some(use_scaffolds) = use_scaffolds {
        matches!(use_scaffolds.as_str(), "true")
    } else {
        true
    };

    let index = open_index(index_path)?;
    let reader = index.reader()?;
    let searcher = reader.searcher();

    let (query_canon_taut, fingerprint, descriptors) = prepare_query_structure(query_smiles)?;

    let matching_scaffolds = if use_scaffolds {
        Some(scaffold_search(
            &fingerprint.0,
            &query_canon_taut,
            &PARSED_SCAFFOLDS,
        )?)
    } else {
        None
    };

    let results = identity_search(
        &searcher,
        &query_canon_taut,
        &matching_scaffolds,
        fingerprint.0.as_bitslice(),
        &descriptors,
        use_chirality,
        &extra_query,
    )?;

    let final_results = results
        .into_par_iter()
        .map(|(smiles, extra_data)| StructureSearchHit {
            extra_data,
            smiles,
            score: 1.0,
            query: query_smiles.into(),
            used_tautomers: false,
        })
        .collect::<Vec<_>>();

    log::info!("{:#?}", final_results);

    Ok(())
}
