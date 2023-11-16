use crate::command_line::prelude::*;
use crate::pubchem::down_all_current_sdf;

pub const NAME: &str = "fetch-pubchem";

pub fn command() -> Command {
    Command::new(NAME).arg(
        Arg::new("output-directory")
            .required(true)
            .long("output-directory")
            .short('d')
            .num_args(1),
    )
}

pub async fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let output_dir = matches
        .get_one::<String>("output-directory")
        .ok_or(eyre::eyre!("Failed to extract output directory"))?;
    down_all_current_sdf(output_dir).await
}
