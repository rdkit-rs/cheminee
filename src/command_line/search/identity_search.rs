use crate::command_line::prelude::*;
use crate::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use crate::search::{
    get_smiles_and_extra_data, identity_search::identity_search, prepare_query_structure,
    StructureSearchHit,
};

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
    let extra_query = matches.get_one::<String>("extra-query");
    let use_scaffolds = matches.get_one::<String>("use-scaffolds");

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

    let scaffolds = if use_scaffolds {
        Some(&PARSED_SCAFFOLDS)
    } else {
        None
    };

    let matching_scaffolds = match scaffolds {
        Some(scaffolds) => Some(scaffold_search(&query_canon_taut, scaffolds)?),
        None => None,
    };

    let result = identity_search(
        &searcher,
        &query_canon_taut,
        &matching_scaffolds,
        fingerprint.0.as_bitslice(),
        &descriptors,
        &extra_query,
    )?;

    if let Some(result) = result {
        let schema = searcher.schema();
        let smiles_field = schema.get_field("smiles")?;
        let extra_data_field = schema.get_field("extra_data")?;

        let (smiles, extra_data) =
            get_smiles_and_extra_data(result, &searcher, smiles_field, extra_data_field)?;

        println!(
            "{:#?}",
            &[StructureSearchHit {
                extra_data,
                smiles,
                score: 1.0,
                query: query_smiles.into(),
                used_tautomers: false,
            }]
        );
    } else {
        println!("No exact match result for {:?}", query_smiles);
    }

    Ok(())
}
