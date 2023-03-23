#![feature(test)]

use rdkit::*;
use cheminee::analysis::compound_processing::*;

extern crate test;
use test::Bencher;

#[bench]
fn bench_tanimoto_distance(b: &mut Bencher) {
    let smiles1 = "[N]Cc1cncc2c(=O)c3cccc(CCC(=O)O)c3[nH]c12";
    let (_proc_smiles1, fingerprint1, _descriptors1) = process_cpd(smiles1).unwrap();

    let smiles2 = "CCc1cccc2c(=O)c3cncc(CN)c3[nH]c12";
    let (_proc_smiles2, fingerprint2, _descriptors2) = process_cpd(smiles2).unwrap();


    b.iter(|| fingerprint1.tanimoto_distance(&fingerprint2));
}

// running 1 test
// Before: test bench_tanimoto_similarity ... bench:     132,690 ns/iter (+/- 2,681)
// Now: test bench_tanimoto_distance ... bench:       2,653 ns/iter (+/- 790)
