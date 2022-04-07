use super::prelude::*;
use crate::pubchem::down_all_current_sdf;

pub const NAME: &'static str = "fetch-pubchem";

pub fn command() -> Command<'static> {
    Command::new(NAME).arg(
        Arg::new("output-directory")
            .required(true)
            .long("output-directory")
            .short('d')
            .takes_value(true),
    )
}

pub async fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let output_dir = matches.value_of("output-directory").unwrap();

    down_all_current_sdf(output_dir).await
}
