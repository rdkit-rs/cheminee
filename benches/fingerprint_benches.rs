#![feature(test)]

use cheminee::search::compound_processing::*;
use rdkit::*;

extern crate test;
use test::Bencher;

#[bench]
fn bench_tanimoto_distance(b: &mut Bencher) {
    let smiles1 = "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12";
    let (_proc_smiles1, pattern_fingerprint1, _descriptors1) = process_cpd(smiles1, false).unwrap();

    let smiles2 = "CCc1cccc2c(=O)c3cncc(CN)c3[nH]c12";
    let (_proc_smiles2, pattern_fingerprint2, _descriptors2) = process_cpd(smiles2, false).unwrap();

    b.iter(|| pattern_fingerprint1.tanimoto_distance(&pattern_fingerprint2));
}

// running 1 test
// Before: test bench_tanimoto_similarity ... bench:     132,690 ns/iter (+/- 2,681)
// Now: test bench_tanimoto_distance ... bench:       2,653 ns/iter (+/- 790)

#[bench]
fn bench_pattern_fingerprint_generation(b: &mut Bencher) {
    let smiles = "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12";
    let romol = ROMol::from_smiles(smiles).unwrap();

    b.iter(|| romol.pattern_fingerprint());
}

// running 1 test
// test bench_pattern_fingerprint_generation ... bench:   1,161,160 ns/iter (+/- 274,184)
