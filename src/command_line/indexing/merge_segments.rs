use segment_manager::SegmentManager;
use tantivy::directory::MmapDirectory;

use crate::command_line::prelude::*;

pub const NAME: &str = "merge-segments";

pub fn command() -> Command {
    Command::new(NAME).arg(
        Arg::new("index")
            .required(true)
            .long("index")
            .short('i')
            .num_args(1),
    )
}

pub fn action(matches: &ArgMatches) -> eyre::Result<()> {
    let index_dir = matches
        .get_one::<String>("index")
        .ok_or(eyre::eyre!("Failed to extract index path"))?;

    log::info!("indexing index_dir={}", index_dir);

    let mmap_index_dir = MmapDirectory::open(index_dir)?;
    let index = tantivy::Index::open(mmap_index_dir)?;

    let segment_manager = SegmentManager {};
    segment_manager.merge(&index)?;

    Ok(())
}
