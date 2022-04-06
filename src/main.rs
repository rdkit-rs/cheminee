use rdkit_sys::molecule::Molecule;
use rdkit_sys::MolBlockIter;

#[tokio::main]
async fn main() {
    let mol_iter = MolBlockIter::from_gz_file("tmp/Compound_000000001_000500000.sdf.gz").unwrap();

    let mut error_count = 0;
    let mut success_count = 0;

    for mol_block in mol_iter {
        match Molecule::new(&mol_block, "") {
            Some(_) => success_count += 1,
            None => error_count += 1,
        }
    }

    println!("successes: {}, errors: {}", success_count, error_count)
}
