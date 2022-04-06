use rdkit_sys::molecule::Molecule;
use rdkit_sys::read_sdfile_gz;
use std::path::Path;

pub async fn read_sdf_from_tmp(path: impl AsRef<Path>) -> eyre::Result<Vec<Option<Molecule>>> {
    let path = path.as_ref().to_owned();

    println!("reading {:?}", path);

    let sdf = read_sdfile_gz(path.to_str().unwrap());

    Ok(sdf)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_read_sdf() {
        read_sdf_from_tmp("tmp/Compound_000000001_000500000.sdf.gz")
            .await
            .unwrap();
    }
}
