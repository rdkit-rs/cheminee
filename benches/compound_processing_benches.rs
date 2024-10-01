#![feature(test)]

use cheminee::search::compound_processing::*;

extern crate test;
use rdkit::ROMol;
use test::Bencher;

#[bench]
fn bench_process_cpd(b: &mut Bencher) {
    let smiles1 =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";

    b.iter(|| {
        let _ = process_cpd(smiles1, false);
    });
}

// running 1 test
// test bench_process_cpd ... bench:   5,527,616.65 ns/iter (+/- 337,547.33)

#[bench]
fn bench_standardize_mol(b: &mut Bencher) {
    let mol = ROMol::from_smiles(
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O",
    )
    .unwrap();

    b.iter(|| {
        let _ = standardize_mol(&mol);
    });
}

// running 1 test
// test bench_standardize_mol ... bench:   3,906,768.75 ns/iter (+/- 230,949.77)

#[bench]
fn bench_get_cpd_properties(b: &mut Bencher) {
    let smiles1 =
        "C[S+](CC[C@@H](C(=O)[O-])[NH3+])C[C@@H]1[C@H]([C@H]([C@@H](O1)N2C=NC3=C(N=CN=C32)N)O)O";
    let canon_taut = standardize_smiles(smiles1, false).unwrap();

    b.iter(|| {
        let _ = get_cpd_properties(&canon_taut);
    });
}

// running 1 test
// test bench_get_cpd_properties ... bench:   1,329,194.77 ns/iter (+/- 183,436.78)
