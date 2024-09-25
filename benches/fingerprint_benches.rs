#![feature(test)]

use cheminee::search::compound_processing::*;
use rdkit::*;

extern crate test;
use test::Bencher;

#[bench]
fn bench_tanimoto_distance(b: &mut Bencher) {
    let smiles1 = "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12";
    let (_proc_smiles1, fingerprint1, _descriptors1) = process_cpd(smiles1, false).unwrap();

    let smiles2 = "CCc1cccc2c(=O)c3cncc(CN)c3[nH]c12";
    let (_proc_smiles2, fingerprint2, _descriptors2) = process_cpd(smiles2, false).unwrap();

    b.iter(|| fingerprint1.tanimoto_distance(&fingerprint2));
}

// running 1 test
// test bench_tanimoto_distance ... bench:       2,520.20 ns/iter (+/- 151.54)

#[bench]
fn bench_fingerprint_generation(b: &mut Bencher) {
    let smiles =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";
    let romol = ROMol::from_smiles(smiles).unwrap();

    b.iter(|| romol.fingerprint());
}

// running 1 test
// test bench_fingerprint_generation ... bench:     784,123.44 ns/iter (+/- 137,752.50)

#[bench]
fn bench_fingerprint_exact_match(b: &mut Bencher) {
    let mol1 = ROMol::from_smiles(
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O",
    )
    .unwrap();
    let fp1 = mol1.fingerprint();
    let fp2 = fp1.clone();

    b.iter(|| fp1.0 == fp2.0);
}

// running 1 test
// test bench_fingerprint_exact_match ... bench:         372.45 ns/iter (+/- 30.19)
