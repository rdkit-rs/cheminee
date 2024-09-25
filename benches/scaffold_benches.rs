#![feature(test)]

extern crate test;
use cheminee::search::compound_processing::process_cpd;
use cheminee::search::scaffold_search::{scaffold_search, PARSED_SCAFFOLDS};
use test::Bencher;

#[bench]
fn bench_scaffold_search(b: &mut Bencher) {
    let smiles =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";
    let (mol, fp, _descriptors) = process_cpd(smiles, false).unwrap();

    b.iter(|| scaffold_search(&fp.0, &mol, &PARSED_SCAFFOLDS));
}

// running 1 test
// test bench_scaffold_search ... bench:     756,170.85 ns/iter (+/- 27,301.91)
